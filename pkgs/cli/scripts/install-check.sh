#!/usr/bin/env bash
# usage: install-check.sh <new-version> [<previous-version>]
set -euo pipefail

new=$1
old=${2:-}
here=$(cd "$(dirname "$0")/.." && pwd)
bin=$HOME/.local/bin/aicentre
data=${XDG_DATA_HOME:-$HOME/.local/share}/aicentre
state=${XDG_STATE_HOME:-$HOME/.local/state}/aicentre
cache=${XDG_CACHE_HOME:-$HOME/.cache}/aicentre
config=${XDG_CONFIG_HOME:-$HOME/.config}/aicentre

step() { printf '\n== %s\n' "$*"; }
fail() {
	printf 'install-check: %s\n' "$*" >&2
	exit 1
}
install() { AICENTRE_VERSION=$1 sh "$here/install.sh" </dev/null; }
want_version() {
	local got
	got=$("$bin" --version)
	[ "$got" = "$1" ] || fail "expected version $1, got $got"
}
gone() {
	local p
	for p in "$@"; do
		if [ -e "$p" ] || [ -L "$p" ]; then fail "$p is left after uninstall"; fi
	done
}

[ ! -e "$data" ] || fail "$data exists: run this as a user without aicentre"
[ ! -e "$config" ] || fail "$config exists: run this as a user without aicentre"

if [ -n "$old" ]; then
	step "install $old"
	install "$old"
	want_version "$old"

	step "upgrade to $new"
	"$bin" upgrade --version "$new"
	want_version "$new"
	[ ! -e "$data/versions/$old" ] || fail "upgrade left $old in $data/versions"

	step "upgrade to $new again"
	"$bin" upgrade --version "$new" | grep -q 'already installed' ||
		fail "a second upgrade did not report $new as installed"

	step "uninstall --purge"
	"$bin" uninstall --purge
	gone "$bin" "$data" "$state" "$cache" "$config"
fi

step "install $new"
install "$new"
want_version "$new"
code=0
"$bin" check --json >/dev/null || code=$?
[ "$code" -le 1 ] || fail "check exited $code"

step "uninstall keeps settings"
mkdir -p "$config"
printf '[dtu]\nuser = "install-check"\n' >"$config/config.toml"
"$bin" uninstall </dev/null
gone "$bin" "$data" "$state" "$cache"
[ -f "$config/config.toml" ] || fail "uninstall removed config.toml"
rm -r "$config"

printf '\ninstall-check: ok\n'
