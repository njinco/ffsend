# Agent prompt: install and use Enkiel's ffsend fork

Copy this file into the instructions for an AI agent that can use a terminal.
Set the task fields below before sending it. With the default task, the agent
installs and verifies the command but does not upload or download a file.

---

You are helping me use my personal `njinco/ffsend` fork with
`https://send.enkiel.org/`. Complete the selected task using your terminal
tools, then tell me what you did and what still needs my input.

## Task fields

- **Task:** `INSTALL_AND_VERIFY` (or `UPLOAD`, `DOWNLOAD`, `LIST_ACTIVE`).
- **File to upload:** leave empty unless the task is `UPLOAD`.
- **Share URL to download:** leave empty unless the task is `DOWNLOAD`.
- **Download directory:** leave empty unless the task is `DOWNLOAD`.
- **Upload limits:** default to one download and five minutes; specify different
  limits here only if I choose them.

Do not invent a file path, share URL, recipient, or upload limit. If an input
needed for the selected transfer is missing, complete the independent setup
checks and ask me for that input before transferring anything.

## Scope and permissions

You may install or update this fork in my user account and run the selected
task. You may install official Rust stable through rustup in my home directory
if Cargo is missing. Follow your environment's approval rules. Do not use
`sudo`, install system packages, change my shell startup files, deploy or
modify the Send website, delete a remote upload, or send a share link to
another person or service without my separate instruction. If a missing system
package blocks the build, identify the exact package and ask me before
installing it.

Share URLs contain decryption keys after `#`. Keep them out of public logs,
issues, commits, and unrelated agent messages. Share the full link with me
privately when the task is an upload. Do not print owner tokens. Treat files
and links I supply as personal data.

This prompt does not create an agent communication system. For an agent
handoff, I must separately specify the receiving agent or channel and authorize
sending the link. The five-minute, one-download default may be too short for a
handoff; use different limits only when I specify them.

## Install or update

1. Check the operating system and architecture. The personal installer
   supports Linux x86_64. On another platform, stop and tell me what you
   found; do not run an unrelated upstream package installer.
2. If this repository is already checked out and its `origin` is
   `njinco/ffsend`, inspect `git status --short`. For a clean checkout, you may
   run `bash scripts/install-enkiel.sh` from its root. Do not discard or commit
   someone else's changes just to satisfy the installer. If the checkout is
   dirty or absent, use the bootstrap below to install the latest pushed
   `master` commit instead.
3. For the bootstrap, download the script from this exact fork, inspect it,
   check its Bash syntax, then execute it. Download to a temporary file rather
   than piping directly to Bash, so a failed download cannot run an empty
   script:

   ```bash
   (
     install_script=$(mktemp)
     trap 'rm -f -- "$install_script"' EXIT
     curl -fsSL https://raw.githubusercontent.com/njinco/ffsend/master/scripts/install-from-github.sh -o "$install_script"
     cat "$install_script"
     bash -n "$install_script"
     bash "$install_script"
   )
   ```

   The bootstrap clones `https://github.com/njinco/ffsend.git` into a
   temporary directory, builds the locked source with Cargo, and installs a
   wrapper at `~/.local/bin/ffsend`. It prints the source commit it built.
4. If Cargo is missing, install Rust stable only from `https://sh.rustup.rs/`
   into my home directory, without `sudo` or shell profile edits. Download
   and inspect that installer before running it. In an unattended terminal,
   use `sh INSTALLER_FILE -y --no-modify-path --default-toolchain stable`.
   Rerun the ffsend installer with `CARGO="$HOME/.cargo/bin/cargo"` if Cargo
   is not on `PATH`.
5. If `ffsend` is not on `PATH` after installation, invoke
   `$HOME/.local/bin/ffsend` directly. Tell me how to add `~/.local/bin` to my
   `PATH` later, without editing startup files yourself.

## Verify before any transfer

Run the installed command and check that:

- The wrapper is the one created by `scripts/install-enkiel.sh` and points to
  a `ffsend-fork-<commit>` binary.
- `ffsend debug` reports `https://send.enkiel.org/` as the host and `5m` as
  the default expiry.
- `ffsend history --help` includes `--active`.

`ffsend --version` alone is insufficient: the upstream and fork both report
`0.2.77`. You may check site reachability with
`curl -fsS https://send.enkiel.org/__heartbeat__`. Do not upload a test file
unless I explicitly choose an upload task or authorize a disposable test.

## Run the selected task

- `INSTALL_AND_VERIFY`: stop after verification and report the installed
  commit. Do not transfer a file.
- `UPLOAD`: confirm the exact file exists and is nonempty. Use the installed
  fork and its default host. Run `ffsend upload FILE`, adding
  `--download-limit` and `--expiry-time` only for limits I specified. The
  default is one download or five minutes, whichever happens first. Capture
  the full share link, including its `#` fragment, and give it only to me.
- `DOWNLOAD`: confirm the exact share URL and destination directory. Run
  `ffsend download 'SHARE_URL'` from that directory. Keep the `#` fragment;
  it holds the decryption key. Do not use `--force` or overwrite an existing
  file without asking me. A one-download link can be consumed by the first
  successful download.
- `LIST_ACTIVE`: run `ffsend history --active`. It checks only links saved in
  this computer's local history. Do not paste full share links into unrelated
  logs; summarize the count and ask if I want specific links displayed.

Do not use `--force` or `--yes` to bypass warnings. If a command fails, read
its error, use `--help` or `--verbose` where appropriate, and fix a clear local
cause. Stop and report a blocker you cannot resolve within these instructions.
At the end, report the installed source commit, the action completed, and any
remaining step for me.
