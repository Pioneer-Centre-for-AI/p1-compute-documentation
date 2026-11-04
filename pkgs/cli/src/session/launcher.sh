#!/bin/bash
set -u

app=$1
target=$2
server=$3
data=$4
base=$5
bind=$6

tag() { printf '@@%s %s\n' "$1" "$2"; }
die() {
	tag ERROR "$1"
	exit 1
}

# shellcheck disable=SC2088
case $target in
/*) ;;
"~") target=$HOME ;;
"~/"*) target=$HOME/${target#"~/"} ;;
*) target=$base/$target ;;
esac
[ -e "$target" ] || die "$target does not exist on the cluster"
if [ -d "$target" ]; then
	dir=$target
	file=
else
	dir=$(dirname "$target")
	file=$(basename "$target")
fi
cd "$dir" || die "cannot open $dir"

mkdir -p "$data/bin" "$data/logs" || die "cannot create $data"
export PATH=$data/bin:$HOME/.local/bin:$PATH
export TERM=${TERM:-xterm-256color}
export UV_CACHE_DIR=$data/uv/cache UV_PYTHON_INSTALL_DIR=$data/uv/python UV_TOOL_DIR=$data/uv/tools UV_TOOL_BIN_DIR=$data/bin

gpu=
if line=$(nvidia-smi --query-gpu=name,memory.used,memory.total --format=csv,noheader,nounits 2>/dev/null | head -n 1) && [ -n "$line" ]; then
	IFS=, read -r name used total <<<"$line"
	gpu="${name# }, $(((total - used) / 1024)) of $((total / 1024)) GiB free, shared with other jobs"
fi

need_uv() {
	command -v uv >/dev/null 2>&1 && return
	if [ "$app" = shell ]; then echo "Installing uv into $data/bin..."; else tag STATUS "installing uv into $data/bin"; fi
	curl -LsSf https://astral.sh/uv/install.sh | env UV_NO_MODIFY_PATH=1 UV_INSTALL_DIR="$data/bin" sh >/dev/null 2>&1 ||
		die "cannot install uv"
}

if [ "$app" = shell ]; then
	need_uv
	if [ -n "${LSB_JOBID:-}${SLURM_JOB_ID:-}" ]; then
		echo "GPU: ${gpu:-none visible}"
		echo "Shell on $(hostname), job ${LSB_JOBID:-$SLURM_JOB_ID}, in $dir. 'exit' ends the job."
	else
		echo "Shell on $(hostname), a login node, in $dir. Keep heavy work for --gpu or batch jobs."
	fi
	exec bash -l
fi

tag PID $$
[ -n "$gpu" ] && tag GPU "$gpu"

port=
for _ in $(seq 1 50); do
	p=$((RANDOM % 30000 + 20000))
	if ! (exec 3<>"/dev/tcp/127.0.0.1/$p") 2>/dev/null; then
		port=$p
		break
	fi
done
[ -n "$port" ] || die "no free port on $(hostname)"
token=$(head -c 24 /dev/urandom | od -An -tx1 | tr -d ' \n')
token_file=$(mktemp)
chmod 600 "$token_file"
printf %s "$token" >"$token_file"
log=$data/logs/${LSB_JOBID:-${SLURM_JOB_ID:-login-$$}}-$app.log

pid=
# shellcheck disable=SC2317,SC2329 # called by the EXIT trap
cleanup() {
	[ -n "$pid" ] && kill "$pid" 2>/dev/null
	rm -f "$token_file"
}
trap cleanup EXIT
trap 'exit 143' TERM HUP INT

case $app in
marimo)
	need_uv
	tag STATUS "starting marimo (the first start installs it, allow a minute)"
	if [ -f pyproject.toml ]; then
		marimo=(uv run --with marimo marimo)
	else
		marimo=(uvx marimo)
	fi
	"${marimo[@]}" edit ${file:+"$file"} --headless --skip-update-check --host "$bind" --port "$port" \
		--token-password-file "$token_file" >"$log" 2>&1 &
	pid=$!
	path="/?access_token=$token"
	;;
code)
	if [ "$server" = openvscode ]; then
		bin=
		for b in "$data"/openvscode/*/bin/openvscode-server; do
			[ -x "$b" ] && bin=$b
		done
		if [ -z "$bin" ]; then
			tag STATUS "fetching openvscode-server"
			latest=$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/gitpod-io/openvscode-server/releases/latest) ||
				die "cannot reach github.com for openvscode-server"
			release=${latest##*/}
			mkdir -p "$data/openvscode"
			curl -fsSL "https://github.com/gitpod-io/openvscode-server/releases/download/$release/$release-linux-x64.tar.gz" |
				tar -xz -C "$data/openvscode" || die "cannot download $release"
			bin=$data/openvscode/$release-linux-x64/bin/openvscode-server
		fi
		tag STATUS "starting openvscode-server"
		"$bin" --host "$bind" --port "$port" --connection-token-file "$token_file" \
			--server-data-dir "$data/openvscode/data" --telemetry-level off --default-folder "$dir" >"$log" 2>&1 &
	else
		code=$data/vscode/bin/code
		if [ ! -x "$code" ]; then
			tag STATUS "fetching the VS Code CLI"
			mkdir -p "$data/vscode/bin"
			curl -fsSL "https://code.visualstudio.com/sha/download?build=stable&os=cli-alpine-x64" |
				tar -xz -C "$data/vscode/bin" code || die "cannot download the VS Code CLI"
		fi
		tag STATUS "starting VS Code (the first start downloads its server, allow a minute)"
		"$code" --cli-data-dir "$data/vscode/cli" serve-web --accept-server-license-terms --disable-telemetry \
			--host "$bind" --port "$port" --connection-token-file "$token_file" \
			--server-data-dir "$data/vscode/server" --default-folder "$dir" >"$log" 2>&1 &
	fi
	pid=$!
	path="/?tkn=$token"
	;;
*) die "unknown app $app" ;;
esac

for _ in $(seq 1 600); do
	if ! kill -0 "$pid" 2>/dev/null; then
		while IFS= read -r l; do tag LOG "$l"; done < <(tail -n 8 "$log")
		pid=
		die "the server stopped before it was ready; its log is $log"
	fi
	if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
		tag HOST "$(hostname)"
		tag PORT "$port"
		tag PATH "$path"
		wait "$pid"
		status=$?
		pid=
		exit "$status"
	fi
	sleep 1
done
die "the server did not answer within 10 minutes; its log is $log"
