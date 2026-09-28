#!/usr/bin/env bash
# Fetch this fork, then run its local source installer.
set -euo pipefail

if [[ $# -gt 1 || ( $# -eq 1 && $1 != /* ) ]]; then
  printf 'Usage: %s [absolute-install-prefix]\n' "$0" >&2
  exit 2
fi
if ! command -v git >/dev/null 2>&1; then
  printf 'Git is required to download this fork.\n' >&2
  exit 2
fi

repository_url=${FFSEND_INSTALL_REPOSITORY:-https://github.com/njinco/ffsend.git}
checkout_dir=$(mktemp -d)
trap 'rm -rf -- "$checkout_dir"' EXIT

git clone --quiet --depth 1 --branch master -- "$repository_url" "$checkout_dir/ffsend"
commit=$(git -C "$checkout_dir/ffsend" rev-parse --short=12 HEAD)
printf 'Building njinco/ffsend at commit %s\n' "$commit"

installer=$checkout_dir/ffsend/scripts/install-enkiel.sh
if [[ -t 2 && -r /dev/tty ]]; then
  bash "$installer" "$@" </dev/tty
else
  bash "$installer" "$@"
fi
