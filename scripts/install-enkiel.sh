#!/usr/bin/env bash
# Build this checkout and install it with send.enkiel.org as the upload default.
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
mode=install
if [[ ${1:-} == --rollback ]]; then
  mode=rollback
  shift
fi
install_prefix=${1:-"$HOME/.local"}
core_dir=$install_prefix/lib/ffsend-enkiel
bin_dir=$install_prefix/bin
wrapper_path=$bin_dir/ffsend
marker='# Managed by ffsend scripts/install-enkiel.sh'
upstream_path=$core_dir/ffsend-v0.2.77
upstream_sha256=ebd14a67c46e7d744ce84677f057d9dc07abc884eaa8f70d68a9e59d27357313

if [[ $# -gt 1 || $install_prefix != /* ]]; then
  printf 'Usage: %s [--rollback] [absolute-install-prefix]\n' "$0" >&2
  exit 2
fi
if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
  printf 'This installer supports Linux x86_64 only. See README.md for other systems.\n' >&2
  exit 2
fi
for command_name in git sha256sum install mktemp; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$command_name" >&2
    exit 2
  fi
done
if [[ -L $wrapper_path || -L $core_dir ]]; then
  printf 'Refusing to replace a symbolic link in %s or %s\n' "$wrapper_path" "$core_dir" >&2
  exit 1
fi
if [[ -e $wrapper_path ]]; then
  if ! { IFS= read -r first_line && IFS= read -r second_line &&
    [[ $first_line == '#!/usr/bin/env bash' && $second_line == "$marker" ]]; } < "$wrapper_path"; then
    printf 'Refusing to replace existing %s\n' "$wrapper_path" >&2
    exit 1
  fi
fi

if [[ $mode == rollback ]]; then
  if [[ ! -f $upstream_path || -L $upstream_path ]]; then
    printf 'Verified upstream fallback is missing: %s\n' "$upstream_path" >&2
    exit 1
  fi
  core_hash=$(sha256sum "$upstream_path")
  if [[ ${core_hash%% *} != "$upstream_sha256" ]]; then
    printf 'Upstream fallback checksum did not match.\n' >&2
    exit 1
  fi
  core_path=$upstream_path
else
  if [[ -n $(git -C "$repo_root" status --porcelain) ]]; then
    printf 'Commit or remove checkout changes before installing a named fork build.\n' >&2
    exit 1
  fi
  commit=$(git -C "$repo_root" rev-parse --verify HEAD)
  cargo_home=${CARGO_HOME:-"$HOME/.cargo"}
  if [[ -n ${CARGO:-} ]]; then
    cargo_bin=$CARGO
  elif command -v cargo >/dev/null 2>&1; then
    cargo_bin=$(command -v cargo)
  elif [[ -x $cargo_home/bin/cargo ]]; then
    cargo_bin=$cargo_home/bin/cargo
  else
    printf 'Cargo was not found on PATH or at %s/bin/cargo.\n' "$cargo_home" >&2
    if [[ -t 0 && -t 2 ]]; then
      printf 'Install Rust stable from https://rustup.rs/ into %s and %s? [y/N] ' \
        "$cargo_home" "${RUSTUP_HOME:-$HOME/.rustup}" >&2
      IFS= read -r answer || answer=
      case $answer in
        y|Y|yes|YES|Yes) ;;
        *) printf 'Install Rust from https://rustup.rs/ and rerun this script.\n' >&2; exit 2 ;;
      esac
      if ! command -v curl >/dev/null 2>&1; then
        printf 'curl is required to download the official Rust installer.\n' >&2
        exit 2
      fi
      rustup_script=$(mktemp)
      trap 'rm -f -- "$rustup_script"' EXIT
      curl --proto '=https' --tlsv1.2 -fsS https://sh.rustup.rs -o "$rustup_script"
      sh "$rustup_script" -y --no-modify-path --default-toolchain stable
      rm -f -- "$rustup_script"
      trap - EXIT
      cargo_bin=$cargo_home/bin/cargo
    else
      printf 'Install Rust stable from https://rustup.rs/ and rerun this script.\n' >&2
      exit 2
    fi
  fi
  if ! command -v "$cargo_bin" >/dev/null 2>&1; then
    printf 'Cargo executable not found: %s\n' "$cargo_bin" >&2
    exit 2
  fi
  if ! "$cargo_bin" --version >/dev/null 2>&1; then
    printf 'Cargo could not run: %s. Check the Rust toolchain installation.\n' "$cargo_bin" >&2
    exit 2
  fi
  build_target_dir=${CARGO_TARGET_DIR:-"$repo_root/target"}
  if [[ $build_target_dir != /* ]]; then
    build_target_dir=$repo_root/$build_target_dir
  fi
  (
    cd -- "$repo_root"
    CARGO_TARGET_DIR=$build_target_dir "$cargo_bin" build --release --locked
  )
  built_binary=$build_target_dir/release/ffsend
  if [[ $("$built_binary" --version) != 'ffsend 0.2.77' ]]; then
    printf 'Built binary reported an unexpected version.\n' >&2
    exit 1
  fi
  core_path=$core_dir/ffsend-fork-${commit:0:12}
  built_hash=$(sha256sum "$built_binary")
  if [[ -L $core_path ]]; then
    printf 'Refusing to replace symbolic link %s\n' "$core_path" >&2
    exit 1
  fi
  if [[ -e $core_path ]]; then
    core_hash=$(sha256sum "$core_path")
    if [[ ${core_hash%% *} != "${built_hash%% *}" ]]; then
      printf 'A different binary already exists for commit %s\n' "$commit" >&2
      exit 1
    fi
  fi
  mkdir -p -- "$core_dir"
  temp_core=$(mktemp "$core_dir/.ffsend-fork.XXXXXX")
  trap 'rm -f -- "$temp_core"' EXIT
  install -m 0755 -- "$built_binary" "$temp_core"
  mv -f -- "$temp_core" "$core_path"
fi

mkdir -p -- "$bin_dir"
temp_wrapper=$(mktemp "$bin_dir/.ffsend.XXXXXX")
trap 'rm -f -- "$temp_wrapper"' EXIT
printf -v quoted_core '%q' "$core_path"
{
  printf '#!/usr/bin/env bash\n%s\n' "$marker"
  printf 'set -euo pipefail\n'
  printf 'export FFSEND_HOST=https://send.enkiel.org/\n'
  printf 'exec %s "$@"\n' "$quoted_core"
} > "$temp_wrapper"
chmod 0755 "$temp_wrapper"
mv -f -- "$temp_wrapper" "$wrapper_path"

printf 'Installed %s and host wrapper %s\n' "$core_path" "$wrapper_path"
printf 'Run: %s --version\n' "$wrapper_path"
