#!/usr/bin/env bash
# Install the verified upstream CLI with send.enkiel.org as the upload default.
set -euo pipefail

version=0.2.77
asset=ffsend-v${version}-linux-x64-static
expected_sha256=ebd14a67c46e7d744ce84677f057d9dc07abc884eaa8f70d68a9e59d27357313
download_url=https://github.com/timvisee/ffsend/releases/download/v${version}/${asset}
install_prefix=${1:-"$HOME/.local"}
core_dir=$install_prefix/lib/ffsend-enkiel
core_path=$core_dir/ffsend-v${version}
bin_dir=$install_prefix/bin
wrapper_path=$bin_dir/ffsend
marker='# Managed by ffsend scripts/install-enkiel.sh'

if [[ $# -gt 1 || $install_prefix != /* ]]; then
  printf 'Usage: %s [absolute-install-prefix]\n' "$0" >&2
  exit 2
fi
if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
  printf 'This installer supports Linux x86_64 only. See README.md for other systems.\n' >&2
  exit 2
fi
for command_name in curl sha256sum install mktemp; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$command_name" >&2
    exit 2
  fi
done
if [[ -L $wrapper_path || -L $core_path ]]; then
  printf 'Refusing to replace a symbolic link in %s or %s\n' "$wrapper_path" "$core_path" >&2
  exit 1
fi
if [[ -e $wrapper_path ]]; then
  if ! { IFS= read -r first_line && IFS= read -r second_line &&
    [[ $first_line == '#!/usr/bin/env bash' && $second_line == "$marker" ]]; } < "$wrapper_path"; then
    printf 'Refusing to replace existing %s\n' "$wrapper_path" >&2
    exit 1
  fi
fi
if [[ -e $core_path ]]; then
  core_hash=$(sha256sum "$core_path")
  if [[ ${core_hash%% *} != "$expected_sha256" ]]; then
    printf 'Refusing to replace a different binary at %s\n' "$core_path" >&2
    exit 1
  fi
fi

temp_dir=$(mktemp -d)
trap 'rm -rf -- "$temp_dir"' EXIT
curl --proto '=https' --tlsv1.2 --fail --location --silent --show-error \
  --retry 3 --max-time 120 --output "$temp_dir/$asset" "$download_url"
printf '%s  %s\n' "$expected_sha256" "$temp_dir/$asset" | sha256sum --check --status || {
  printf 'Downloaded binary checksum did not match the pinned release.\n' >&2
  exit 1
}
chmod 700 "$temp_dir/$asset"
if [[ $("$temp_dir/$asset" --version) != "ffsend $version" ]]; then
  printf 'Downloaded binary reported an unexpected version.\n' >&2
  exit 1
fi

mkdir -p -- "$core_dir" "$bin_dir"
install -m 0755 -- "$temp_dir/$asset" "$core_path"
printf -v quoted_core '%q' "$core_path"
{
  printf '#!/usr/bin/env bash\n%s\n' "$marker"
  printf 'set -euo pipefail\n'
  printf 'export FFSEND_HOST=https://send.enkiel.org/\n'
  printf 'exec %s "$@"\n' "$quoted_core"
} > "$temp_dir/ffsend"
install -m 0755 -- "$temp_dir/ffsend" "$wrapper_path"

printf 'Installed %s and host wrapper %s\n' "$core_path" "$wrapper_path"
printf 'Run: %s --version\n' "$wrapper_path"
