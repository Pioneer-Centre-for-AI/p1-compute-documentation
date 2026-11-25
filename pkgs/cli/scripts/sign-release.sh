#!/usr/bin/env bash
# usage: sign-release.sh <private-key> [<tag>...]
# With no tag it signs every release that has SHA256SUMS and no signature.
set -euo pipefail

key=$1
shift
here=$(cd "$(dirname "$0")/.." && pwd)
repo=${GITHUB_REPOSITORY:-Pioneer-Centre-for-AI/p1-compute-documentation}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

assets() { gh release view "$1" --repo "$repo" --json assets --jq '.assets[].name'; }

sha256() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | cut -d' ' -f1
	else
		shasum -a 256 "$1" | cut -d' ' -f1
	fi
}

built_here() {
	local tag=$1 dir=$2 sum name seen=0
	while read -r sum name; do
		name=${name#\*}
		case $name in
		aicentre-*.tar.gz) ;;
		*) continue ;;
		esac
		seen=$((seen + 1))
		gh release download "$tag" --repo "$repo" --pattern "$name" --dir "$dir" || return 1
		if [ "$(sha256 "$dir/$name")" != "$sum" ]; then
			printf '%s: %s does not match SHA256SUMS\n' "$tag" "$name" >&2
			return 1
		fi
		if ! gh attestation verify "$dir/$name" --repo "$repo" \
			--signer-workflow "$repo/.github/workflows/release-cli.yaml" \
			--source-ref "refs/tags/$tag" >/dev/null 2>&1; then
			printf '%s: GitHub does not attest that the release workflow built %s from this tag\n' "$tag" "$name" >&2
			return 1
		fi
	done <"$dir/SHA256SUMS"
	if [ "$seen" -eq 0 ]; then
		printf '%s: SHA256SUMS lists no archive\n' "$tag" >&2
		return 1
	fi
}

if [ "$#" -eq 0 ]; then
	mapfile -t tags < <(gh release list --repo "$repo" --limit 200 --json tagName --jq '.[].tagName' | grep '^cli-v')
else
	tags=("$@")
fi

signed=0
refused=0
for tag in "${tags[@]}"; do
	names=$(assets "$tag")
	grep -qx SHA256SUMS <<<"$names" || {
		printf '%s has no SHA256SUMS, skipped\n' "$tag"
		continue
	}
	if [ "$#" -eq 0 ] && grep -qx SHA256SUMS.sig <<<"$names"; then
		continue
	fi
	dir=$tmp/$tag
	mkdir -p "$dir"
	gh release download "$tag" --repo "$repo" --pattern SHA256SUMS --dir "$dir"
	if ! built_here "$tag" "$dir"; then
		printf '%s was not signed\n' "$tag" >&2
		refused=$((refused + 1))
		continue
	fi
	# No agent: one that does not answer would hang the signing.
	SSH_AUTH_SOCK='' ssh-keygen -Y sign -f "$key" -n aicentre-release "$dir/SHA256SUMS" 2>/dev/null
	ssh-keygen -Y verify -f "$here/release.pub" -I release@hpc.aicentre.dk \
		-n aicentre-release -s "$dir/SHA256SUMS.sig" <"$dir/SHA256SUMS" >/dev/null
	gh release upload "$tag" --repo "$repo" --clobber "$dir/SHA256SUMS.sig"
	printf 'signed %s\n' "$tag"
	signed=$((signed + 1))
done
printf '%s release(s) signed\n' "$signed"
[ "$refused" -eq 0 ]
