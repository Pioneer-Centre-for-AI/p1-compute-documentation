#!/bin/bash
set -u

plain() {
	case $1 in
	hex:*) printf '%b' "$(printf %s "${1#hex:}" | sed 's/../\\x&/g')" ;;
	*) printf %s "$1" ;;
	esac
}
app=$1
target=$(plain "$2")
server=$3
data=$(plain "$4")
base=$(plain "$5")
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
chmod 700 "$data" "$data/logs" || die "cannot make $data private"
find "$data/logs" -type f -mtime +14 -delete 2>/dev/null
find "$(dirname "$0")" -maxdepth 1 -name 'session-*.sh' ! -name "$(basename "$0")" -mtime +14 -delete 2>/dev/null
export PATH=$data/bin:$HOME/.local/bin:$PATH
export TERM=${TERM:-xterm-256color}
export UV_CACHE_DIR=$data/uv/cache UV_PYTHON_INSTALL_DIR=$data/uv/python UV_TOOL_DIR=$data/uv/tools UV_TOOL_BIN_DIR=$data/bin

gpu=
if line=$(nvidia-smi --query-gpu=name,memory.used,memory.total --format=csv,noheader,nounits 2>/dev/null | head -n 1) && [ -n "$line" ]; then
	IFS=, read -r name used total <<<"$line"
	gpu="${name# }, $(((total - used) / 1024)) of $((total / 1024)) GiB free, shared with other jobs"
fi

urlencode() {
	local LC_ALL=C s=$1 out='' c i
	for ((i = 0; i < ${#s}; i++)); do
		c=${s:i:1}
		case $c in
		[A-Za-z0-9._~/-]) out+=$c ;;
		*) printf -v c '%%%02X' "'$c" && out+=$c ;;
		esac
	done
	printf %s "$out"
}

connected() {
	[ "$(cat "$keys/busy" 2>/dev/null || echo 0)" -gt 0 ]
}

in_job=${LSB_JOBID:-}${SLURM_JOB_ID:-}

serve() {
	local run=(env)
	command -v setsid >/dev/null 2>&1 && run+=(setsid)
	[ -z "$in_job" ] && run+=(nice -n 10)
	exec "${run[@]}" "$@"
}

login_watch() {
	local limit=${AICENTRE_LOGIN_SECONDS:-43200} ran=0 step
	step=$((limit < 60 ? 1 : 30))
	while kill -0 "$pid" 2>/dev/null; do
		sleep "$step" &
		wait $!
		ran=$((ran + step))
		tag BEAT "$ran" || exit 143
		if [ "$ran" -ge "$limit" ]; then
			tag ERROR "a login-node session runs for $((limit / 3600)) h at most, so it ended; start it again"
			exit 0
		fi
	done
}

idle_watch() {
	local limit=${AICENTRE_IDLE_SECONDS:-600} quiet=0 step span what=browser
	[ "$app" = sshd ] && what=ssh
	step=$((limit < 60 ? 1 : 30))
	if [ "$limit" -ge 60 ]; then span="$((limit / 60)) min"; else span="$limit s"; fi
	while kill -0 "$pid" 2>/dev/null; do
		tag BEAT "$quiet" || exit 143
		if connected; then quiet=0; else quiet=$((quiet + step)); fi
		if [ "$quiet" -ge "$limit" ]; then
			tag ERROR "no $what connection for $span, so the job ended to free the GPU"
			exit 0
		fi
		sleep "$step" &
		wait $!
	done
}

free_port() {
	local p
	for _ in $(seq 1 50); do
		p=$((RANDOM % 30000 + 20000))
		if ! (exec 3<>"/dev/tcp/127.0.0.1/$p") 2>/dev/null; then
			echo "$p"
			return
		fi
	done
	return 1
}

# 0: this user's own process listens on the port. 1: nothing does. 2: someone else's does.
listener() {
	local lines
	(exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null || return 1
	if command -v ss >/dev/null 2>&1; then
		lines=$(ss -Hltnp "sport = :$1" 2>/dev/null)
		[ -n "$lines" ] && ! grep -qv 'pid=' <<<"$lines" && return 0
	elif command -v lsof >/dev/null 2>&1; then
		lsof -nP -a -u "$(id -u)" -iTCP:"$1" -sTCP:LISTEN >/dev/null 2>&1 && return 0
	fi
	return 2
}

UV=0.13.9
UV_SHA_x86_64=58ccf5f1792b8a5a88d01bc910ea131a8b3db4a319a17ad5fad75e0845a3e271
UV_SHA_aarch64=37fbb7c5bc66cf99c12aa881949d216181e201dec449772c4329e56487d61179
OPENVSCODE=openvscode-server-v1.109.5
OPENVSCODE_SHA=b433bf4f0227321a7014d8460d10a8f958adc0f45aa79bd889e84e65e8f88363
VSCODE=69047191fc427f2f92828279aae1d90d8eeda7ab
VSCODE_SHA=a03f96e434317abadcf4ba959057a5a05a971df863b50714adf461d4954f0438

sha256() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | cut -d' ' -f1
	else
		shasum -a 256 "$1" | cut -d' ' -f1
	fi
}

fetch() {
	local what=$1 url=$2 want=$3 to=$4 got proto='=https'
	if [ -n "${AICENTRE_FETCH_FROM:-}" ]; then
		url=$AICENTRE_FETCH_FROM/${url##*/}
		proto='=https,file'
	fi
	rm -f "$to"
	curl -fsSL --proto "$proto" -o "$to" "$url" || die "cannot download $what from ${url%/*}"
	got=$(sha256 "$to")
	[ "$got" = "$want" ] && return
	rm -f "$to"
	die "$what from ${url%/*} is not the file this aicentre was built for (sha256 $got), so it was deleted and nothing ran"
}

x64_only() {
	[ "$(uname -m)" = x86_64 ] || die "$1 is only set up for x86_64 nodes, and this one is $(uname -m)"
}

need_uv() {
	command -v uv >/dev/null 2>&1 && return
	if [ "$app" = shell ]; then echo "Installing uv into $data/bin..."; else tag STATUS "installing uv into $data/bin"; fi
	local arch sha tmp
	arch=$(uname -m)
	case $arch in
	x86_64) sha=$UV_SHA_x86_64 ;;
	aarch64 | arm64) arch=aarch64 sha=$UV_SHA_aarch64 ;;
	*) die "no uv is set up for $arch nodes" ;;
	esac
	tmp=$data/bin/.uv-$$.tar.gz
	fetch "uv $UV" "https://github.com/astral-sh/uv/releases/download/$UV/uv-$arch-unknown-linux-musl.tar.gz" "$sha" "$tmp"
	tar -xzf "$tmp" -C "$data/bin" --strip-components=1 || die "cannot unpack uv"
	rm -f "$tmp"
}

if [ "$app" = shell ]; then
	need_uv
	if [ -n "${LSB_JOBID:-}${SLURM_JOB_ID:-}" ]; then
		echo "GPU: ${gpu:-none visible}"
		idle=${AICENTRE_IDLE_SECONDS:-600}
		echo "Shell on $(hostname), job ${LSB_JOBID:-$SLURM_JOB_ID}, in $dir. 'exit' ends the job, and so does $((idle / 60)) min at an idle prompt."
		export TMOUT=$idle
	else
		echo "Shell on $(hostname), a login node, in $dir. Keep heavy work for --gpu or batch jobs."
	fi
	exec bash -l
fi

tag PID $$
tag DIR "$dir"
[ -n "$gpu" ] && tag GPU "$gpu"

port=$(free_port) || die "no free port on $(hostname)"
token=$(head -c 24 /dev/urandom | od -An -tx1 | tr -d ' \n')
token_file=$(mktemp)
chmod 600 "$token_file"
printf %s "$token" >"$token_file"
log=$data/logs/${LSB_JOBID:-${SLURM_JOB_ID:-login-$$}}-$app.log
gate_log=$data/logs/${LSB_JOBID:-${SLURM_JOB_ID:-login-$$}}-gate.log
(umask 077 && : >"$log") || die "cannot create $log"

pid=
keys=
gate_pid=
# shellcheck disable=SC2317,SC2329 # called by the EXIT trap
cleanup() {
	[ -n "$pid" ] && { kill -- -"$pid" 2>/dev/null || kill "$pid" 2>/dev/null; }
	[ -n "$gate_pid" ] && kill "$gate_pid" 2>/dev/null
	rm -f "$token_file"
	[ -n "$keys" ] && rm -rf "$keys"
}
trap cleanup EXIT
trap 'exit 143' TERM HUP INT

start_gate() {
	local mode=$1 gate_port=$2 target=$3 python ips allow='' ip setenv='' config=''
	python=$(command -v python3 2>/dev/null || echo /usr/libexec/platform-python)
	[ -x "$python" ] || die "$(hostname) has no python3 for the connection gate"
	ips=${AICENTRE_GATE_ALLOW:-}
	if [ -z "$ips" ]; then
		for ip in $(getent ahostsv4 "${LSB_SUB_HOST:-}" 2>/dev/null | awk '$1 !~ /^127\./ { print $1 }' | sort -u); do
			ips="${ips:+$ips,}$ip"
		done
	fi
	for ip in ${ips//,/ }; do
		allow="${allow:+$allow }$USER@$ip"
	done
	[ -n "$ips" ] || die "cannot resolve ${LSB_SUB_HOST:-the submitting login node}, so the gate cannot limit who connects"
	keys=$(mktemp -d) || die "cannot create a private directory"
	(umask 077 && head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n' >"$keys/secret") || die "cannot make a secret"
	if [ "$mode" = sshd ]; then
		target=$(command -v sshd 2>/dev/null || echo /usr/sbin/sshd)
		[ -x "$target" ] || die "$(hostname) has no sshd for your editor to connect to"
		# shellcheck disable=SC2088 # a literal ~ reads better in the message
		[ -s "${AICENTRE_AUTHORIZED_KEYS:-$HOME/.ssh/authorized_keys}" ] || die "~/.ssh/authorized_keys on the cluster is empty, so your editor cannot log in"
		ssh-keygen -q -t ed25519 -N '' -C aicentre-gpu -f "$keys/host" || die "cannot make a host key"
		[ -n "${CUDA_VISIBLE_DEVICES+set}" ] && setenv="$setenv CUDA_VISIBLE_DEVICES=$CUDA_VISIBLE_DEVICES"
		[ -n "${LSB_JOBID+set}" ] && setenv="$setenv LSB_JOBID=$LSB_JOBID"
		config=$keys/sshd_config
		cat >"$config" <<-EOF
			HostKey $keys/host
			PasswordAuthentication no
			AllowUsers $allow
			AllowTcpForwarding local
			AllowAgentForwarding no
			X11Forwarding no
			PermitTunnel no
			LoginGraceTime 20
			Subsystem sftp internal-sftp
		EOF
		[ -n "$setenv" ] && echo "SetEnv$setenv" >>"$config"
		[ -n "${AICENTRE_AUTHORIZED_KEYS:-}" ] && printf 'AuthorizedKeysFile %s\nStrictModes no\n' "$AICENTRE_AUTHORIZED_KEYS" >>"$config"
		"$target" -t -f "$config" >"$keys/check" 2>&1 || die "this node's sshd rejects the session's config: $(head -n 1 "$keys/check")"
	fi
	cat >"$keys/gate.py" <<-'EOF'
		import hmac, os, select, socket, struct, sys, time
		bind, port, secret_file, allow, mode, target, config, busy = sys.argv[1:9]
		with open(secret_file, "rb") as f:
		    secret = f.read()
		allowed = set(filter(None, allow.split(",")))
		live, authed = set(), set()
		r, w = os.pipe()
		os.set_blocking(r, False)
		def note(count):
		    with open(busy + ".new", "w") as f:
		        f.write(str(count))
		    os.replace(busy + ".new", busy)
		def reap():
		    while True:
		        try:
		            pid, _ = os.waitpid(-1, os.WNOHANG)
		        except ChildProcessError:
		            return
		        if pid == 0:
		            return
		        live.discard(pid)
		        authed.discard(pid)
		noted = 0
		note(noted)
		server = socket.socket()
		server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
		server.bind((bind, int(port)))
		server.listen(8)
		while True:
		    ready = select.select([server, r], [], [], 1)[0]
		    if r in ready:
		        said = os.read(r, 4096)
		        for i in range(0, len(said) - 3, 4):
		            authed.add(struct.unpack("I", said[i:i + 4])[0])
		    reap()
		    authed &= live
		    if len(authed) != noted:
		        noted = len(authed)
		        note(noted)
		    if server not in ready:
		        continue
		    conn, peer = server.accept()
		    waiting = len(live) - len(authed)
		    if len(live) >= 16 or waiting >= 4 or (allowed and peer[0] not in allowed):
		        conn.close()
		        continue
		    pid = os.fork()
		    if pid:
		        live.add(pid)
		        conn.close()
		        continue
		    server.close()
		    os.close(r)
		    deadline = time.monotonic() + 5
		    got = b""
		    try:
		        while len(got) < len(secret):
		            conn.settimeout(max(deadline - time.monotonic(), 0.01))
		            chunk = conn.recv(len(secret) - len(got))
		            if not chunk:
		                break
		            got += chunk
		    except OSError:
		        pass
		    if not hmac.compare_digest(got, secret):
		        os._exit(0)
		    os.write(w, struct.pack("I", os.getpid()))
		    os.close(w)
		    conn.settimeout(None)
		    if mode == "sshd":
		        os.dup2(conn.fileno(), 0)
		        os.dup2(conn.fileno(), 1)
		        os.execv(target, [target, "-i", "-e", "-f", config])
		    upstream = socket.create_connection(("127.0.0.1", int(target)))
		    pair = {conn: upstream, upstream: conn}
		    while True:
		        for s in select.select(list(pair), [], [])[0]:
		            try:
		                data = s.recv(65536)
		            except OSError:
		                data = b""
		            if not data:
		                os._exit(0)
		            pair[s].sendall(data)
	EOF
	(umask 077 && : >"$gate_log") || die "cannot create $gate_log"
	"$python" "$keys/gate.py" "$bind" "$gate_port" "$keys/secret" "$ips" "$mode" "$target" "$config" "$keys/busy" >>"$gate_log" 2>&1 &
	gate_pid=$!
	[ "$mode" = sshd ] && tag HOSTKEY "$(cat "$keys/host.pub")"
	tag SECRET "$(cat "$keys/secret")"
	tag GATE "$gate_port"
}

if [ -n "${LSB_JOBID:-}${SLURM_JOB_ID:-}" ] && [ "$app" != sshd ]; then
	tag STATUS "starting the gate into the job"
	gate_port=$port
	while [ "$gate_port" = "$port" ]; do
		gate_port=$(free_port) || die "no free port on $(hostname)"
	done
	start_gate forward "$gate_port" "$port"
fi

case $app in
marimo)
	need_uv
	tag STATUS "starting marimo (the first start installs it, allow a minute)"
	if [ -f pyproject.toml ]; then
		marimo=(uv run --with marimo marimo)
	else
		marimo=(uvx marimo)
	fi
	serve "${marimo[@]}" edit ${file:+"$file"} --headless --skip-update-check --host 127.0.0.1 --port "$port" \
		--token-password-file "$token_file" >"$log" 2>&1 &
	pid=$!
	path="/?access_token=$token"
	;;
jupyter)
	need_uv
	tag STATUS "starting JupyterLab (the first start installs it, allow a minute)"
	if [ -f pyproject.toml ]; then
		jupyter=(uv run --with jupyterlab jupyter-lab)
	else
		jupyter=(uvx --from jupyterlab jupyter-lab)
	fi
	cull=()
	[ -z "$in_job" ] && cull=(--MappingKernelManager.cull_idle_timeout=3600 --ServerApp.shutdown_no_activity_timeout=3600)
	JUPYTER_TOKEN=$token serve "${jupyter[@]}" --no-browser --ip 127.0.0.1 --port "$port" \
		--ServerApp.port_retries=0 ${cull[@]+"${cull[@]}"} --LabApp.check_for_updates_class=jupyterlab.NeverCheckForUpdate >"$log" 2>&1 &
	pid=$!
	path="/lab${file:+/tree/$(urlencode "$file")}?token=$token"
	;;
code)
	if [ "$server" = openvscode ]; then
		bin=$data/openvscode/$OPENVSCODE-linux-x64/bin/openvscode-server
		if [ ! -x "$bin" ]; then
			tag STATUS "fetching openvscode-server"
			x64_only openvscode-server
			mkdir -p "$data/openvscode"
			tmp=$data/openvscode/.fetch-$$.tar.gz
			fetch "$OPENVSCODE" "https://github.com/gitpod-io/openvscode-server/releases/download/$OPENVSCODE/$OPENVSCODE-linux-x64.tar.gz" "$OPENVSCODE_SHA" "$tmp"
			tar -xzf "$tmp" -C "$data/openvscode" || die "cannot unpack $OPENVSCODE"
			rm -f "$tmp"
		fi
		tag STATUS "starting openvscode-server"
		serve "$bin" --host 127.0.0.1 --port "$port" --connection-token-file "$token_file" \
			--server-data-dir "$data/openvscode/data" --telemetry-level off --default-folder "$dir" >"$log" 2>&1 &
	else
		code=$data/vscode/$VSCODE/code
		if [ ! -x "$code" ]; then
			tag STATUS "fetching the VS Code CLI"
			x64_only "the VS Code CLI"
			mkdir -p "$data/vscode/$VSCODE"
			tmp=$data/vscode/.fetch-$$.tar.gz
			fetch "the VS Code CLI" "https://update.code.visualstudio.com/commit:$VSCODE/cli-alpine-x64/stable" "$VSCODE_SHA" "$tmp"
			tar -xzf "$tmp" -C "$data/vscode/$VSCODE" code || die "cannot unpack the VS Code CLI"
			rm -rf "$tmp" "$data/vscode/bin"
		fi
		tag STATUS "starting VS Code (the first start downloads its server, allow a minute)"
		serve "$code" --cli-data-dir "$data/vscode/cli" serve-web --commit-id "$VSCODE" --accept-server-license-terms --disable-telemetry \
			--host 127.0.0.1 --port "$port" --connection-token-file "$token_file" \
			--server-data-dir "$data/vscode/server" --default-folder "$dir" >"$log" 2>&1 &
	fi
	pid=$!
	path="/?tkn=$token"
	;;
sshd)
	tag STATUS "starting the gate for your editor's ssh"
	start_gate sshd "$port" ''
	pid=$gate_pid
	path=
	;;
*) die "unknown app $app" ;;
esac

for _ in $(seq 1 600); do
	if [ -n "$gate_pid" ] && ! kill -0 "$gate_pid" 2>/dev/null; then
		while IFS= read -r l; do tag LOG "$l"; done < <(tail -n 8 "$gate_log")
		gate_pid=
		die "the gate into the job stopped; its log is $gate_log"
	fi
	if ! kill -0 "$pid" 2>/dev/null; then
		while IFS= read -r l; do tag LOG "$l"; done < <(tail -n 8 "$log")
		pid=
		die "the server stopped before it was ready; its log is $log"
	fi
	listener "$port"
	owner=$?
	[ "$owner" = 2 ] && die "port $port on $(hostname) is not this session's own, so nothing was opened; start it again"
	if [ "$owner" = 0 ]; then
		tag HOST "$(hostname)"
		tag PORT "$port"
		tag PATH "$path"
		if [ -n "$gate_pid" ]; then idle_watch; else login_watch; fi
		wait "$pid"
		status=$?
		pid=
		exit "$status"
	fi
	sleep 1
done
die "the server did not answer within 10 minutes; its log is $log"
