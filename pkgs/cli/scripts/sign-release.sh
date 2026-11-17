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

if [ "$#" -eq 0 ]; then
	mapfile -t tags < <(gh release list --repo "$repo" --limit 200 --json tagName --jq '.[].tagName' | grep '^cli-v')
else
	tags=("$@")
fi

signed=0
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
	ssh-keygen -Y sign -f "$key" -n aicentre-release "$dir/SHA256SUMS" 2>/dev/null
	ssh-keygen -Y verify -f "$here/release.pub" -I release@hpc.aicentre.dk \
		-n aicentre-release -s "$dir/SHA256SUMS.sig" <"$dir/SHA256SUMS" >/dev/null
	gh release upload "$tag" --repo "$repo" --clobber "$dir/SHA256SUMS.sig"
	printf 'signed %s\n' "$tag"
	signed=$((signed + 1))
done
printf '%s release(s) signed\n' "$signed"
