#!/bin/sh
# curl -fsSL https://hpc.aicentre.dk/install | sh

set -eu

REPO="https://github.com/Pioneer-Centre-for-AI/p1-compute-documentation"
REPO_FLAKE="github:Pioneer-Centre-for-AI/p1-compute-documentation"

SIGNER="release@hpc.aicentre.dk"
SIGNERS='release@hpc.aicentre.dk namespaces="aicentre-release" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIK+uPLLcLaHWeOrDw9e03Y4LAPJmRKMkLxPb/6rB++mh'

err() {
	printf 'aicentre install: %s\n' "$1" >&2
	exit 1
}

has() {
	command -v "$1" >/dev/null 2>&1
}

fetch() {
	case $1 in
	file://*) cp "${1#file://}" "$2" ;;
	/*) cp "$1" "$2" ;;
	*)
		if has curl; then
			curl -fsSL --proto '=https' --tlsv1.2 --retry 3 -o "$2" "$1"
		elif has wget; then
			wget -q --https-only -O "$2" "$1"
		else
			err "needs curl or wget to download the release"
		fi
		;;
	esac
}

sha256() {
	if has sha256sum; then
		sha256sum "$1" | awk '{ print $1 }'
	elif has shasum; then
		shasum -a 256 "$1" | awk '{ print $1 }'
	else
		err "needs sha256sum or shasum to check the download"
	fi
}

platform() {
	case $(uname -s) in
	Linux) os=linux ;;
	Darwin) os=darwin ;;
	*) err "runs on Linux and macOS only; on Windows, run it inside WSL" ;;
	esac

	case $(uname -m) in
	x86_64 | amd64) arch=x64 ;;
	aarch64 | arm64) arch=arm64 ;;
	*) err "has no build for $(uname -m)" ;;
	esac

	if [ "$os" = darwin ] && [ "$arch" = x64 ] && [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = 1 ]; then
		arch=arm64
	fi

	case ${AICENTRE_VERSION:-} in
	0.1.*) needs_glibc=1 ;;
	*) needs_glibc=0 ;;
	esac
	if [ "$os" = linux ] && [ "$needs_glibc" -eq 1 ]; then
		for loader in /lib/ld-musl-*; do
			if [ -e "$loader" ]; then
				err "needs a glibc Linux; this one uses musl (Alpine?)"
			fi
		done
		if [ -e /etc/NIXOS ] && [ -z "${NIX_LD:-}" ]; then
			err "on NixOS, use the flake instead: nix run $REPO_FLAKE#aicentre (see https://hpc.aicentre.dk/cli/#with-nix)"
		fi
	fi
}

main() {
	allow_root=0
	for arg in "$@"; do
		case $arg in
		--allow-root) allow_root=1 ;;
		*) err "unknown option $arg" ;;
		esac
	done

	if [ "$(id -u)" -eq 0 ] && [ "$allow_root" -eq 0 ]; then
		err "run this as yourself, not root; aicentre installs into your home directory"
	fi

	platform
	asset="aicentre-$os-$arch.tar.gz"

	if [ -n "${AICENTRE_DOWNLOAD_BASE:-}" ]; then
		base=${AICENTRE_DOWNLOAD_BASE%/}
	elif [ -n "${AICENTRE_VERSION:-}" ]; then
		base="$REPO/releases/download/cli-v$AICENTRE_VERSION"
	else
		base="$REPO/releases/latest/download"
	fi

	tmp=$(mktemp -d 2>/dev/null || mktemp -d -t aicentre)
	trap 'rm -rf "$tmp"' EXIT
	trap 'exit 130' INT TERM

	printf 'Downloading %s...\n' "$asset"
	fetch "$base/$asset" "$tmp/$asset" || err "cannot download $base/$asset"
	fetch "$base/SHA256SUMS" "$tmp/SHA256SUMS" || err "cannot download $base/SHA256SUMS"

	fetch "$base/SHA256SUMS.sig" "$tmp/SHA256SUMS.sig" || err "this release has no signature ($base/SHA256SUMS.sig), so it was not installed"
	has ssh-keygen || err "needs ssh-keygen to check the release signature"
	signers=$SIGNERS
	if [ -n "${AICENTRE_DOWNLOAD_BASE:-}" ] && [ -n "${AICENTRE_RELEASE_SIGNERS:-}" ]; then
		signers=$AICENTRE_RELEASE_SIGNERS
	fi
	printf '%s\n' "$signers" >"$tmp/signers"
	ssh-keygen -Y verify -f "$tmp/signers" -I "$SIGNER" -n aicentre-release -s "$tmp/SHA256SUMS.sig" <"$tmp/SHA256SUMS" >/dev/null 2>&1 ||
		err "SHA256SUMS is not signed with aicentre's release key, so nothing was installed"

	expected=$(awk -v f="$asset" '$2 == f || $2 == "*" f { print $1 }' "$tmp/SHA256SUMS")
	[ -n "$expected" ] || err "SHA256SUMS has no line for $asset"
	actual=$(sha256 "$tmp/$asset")
	[ "$expected" = "$actual" ] || err "$asset does not match its checksum (expected $expected, got $actual)"

	tar -xzf "$tmp/$asset" -C "$tmp" || err "cannot unpack $asset"
	[ -x "$tmp/aicentre" ] || err "$asset has no aicentre binary in it"

	status=0
	"$tmp/aicentre" self-install </dev/null || status=$?
	exit "$status"
}

main "$@"
