# Personal setup for send.enkiel.org

This guide connects the `ffsend` command-line client in this repository to the
separate Send service intended for `https://send.enkiel.org`. The client encrypts
a file on your computer and uploads the encrypted result. It prints a share
link; a recipient uses that link to download and decrypt the file. The service
stores the upload and enforces its expiry and download count.

The server source lives in the sibling `send` repository. The deployment sample
lives in `send-docker-compose`. Installing this client does not deploy the
server.

## 1. Install the terminal command

On Linux x86_64, the personal installer builds the committed source in this
fork with the locked dependencies, then installs that binary under
`~/.local/lib/ffsend-enkiel/`. The `~/.local/bin/ffsend` wrapper selects
`https://send.enkiel.org/` as the upload default. The installer does not use
`sudo`, modify your shell startup files, or deploy the Send server. It needs
Rust stable with Cargo, `pkg-config`, and OpenSSL development libraries to
build. The resulting binary uses the system OpenSSL libraries at runtime.

From the root of this repository:

```bash
git status --short
git rev-parse --short=12 HEAD
bash scripts/install-enkiel.sh
command -v ffsend
ffsend --version
sed -n '5p' "$HOME/.local/bin/ffsend"
```

The first command should print nothing: the installer requires a clean
checkout so the installed filename identifies its exact source commit. The
last command shows which binary the wrapper runs. It should contain
`ffsend-fork-` followed by the commit prefix. The project version remains
`0.2.77`, so `ffsend --version` alone cannot distinguish the fork build from
the upstream release. The installer refuses to replace an existing
`~/.local/bin/ffsend` unless it was created by this installer. If
`command -v` cannot find the wrapper, start a new terminal or add
`$HOME/.local/bin` to that shell's `PATH`. You can also run it by its full
path: `$HOME/.local/bin/ffsend`.

The installer accepts an optional absolute prefix for testing or a separate
user-local location: `bash scripts/install-enkiel.sh /absolute/prefix`. The
wrapper is then installed in that prefix's `bin` directory. Set
`CARGO_TARGET_DIR` if you want the build files outside the checkout. The
installer preserves the previously installed official v0.2.77 binary. To
switch the wrapper back to that verified binary, run:

```bash
bash scripts/install-enkiel.sh --rollback
```

Rollback checks the official binary's pinned SHA-256 before switching. It
does not need Cargo. Run the installer again from a clean checkout to return
to the fork build.

For other operating systems and architectures, see the main [installation
guide](../README.md#install). On those systems, use the host setting in the
next section explicitly until a platform-specific wrapper is available.

To remove this personal installation, after confirming these are the files
created by the script, remove `~/.local/bin/ffsend` and the installed binaries
under `~/.local/lib/ffsend-enkiel/`. This does not delete your local `ffsend`
history or uploads already stored on the server.

## 2. Check the prerequisites

Check your installed binary:

```bash
ffsend --version
ffsend debug
```

The second command shows the features compiled into your binary. The examples
below assume the default `history` and `archive` features. If the binary or
site is unavailable, resolve that before attempting an upload.

The personal fork's default build also shows `crypto-openssl` in this output.
That is the file encryption backend used by the installed command. The
`crypto-ring` option remains available for compatibility, but it is not
compiled into the default binary.

To check whether the service is reachable without uploading anything:

```bash
curl -fsS -o /dev/null -w '%{http_code}\n' https://send.enkiel.org/__heartbeat__
ffsend --api auto version --host https://send.enkiel.org/
```

Expect HTTP `200` from the heartbeat and a Send server version from `ffsend`.
These checks do not prove that file transfers work.

On September 28, 2026, a live check from this workspace returned HTTP `200`,
reported API version `3`, and completed an 18-byte upload/download round trip
with matching contents. The transfer used a temporary copy of the official
`ffsend` v0.2.77 Linux binary. A second round trip using the installed wrapper
also passed without `--host`, even with a conflicting inherited
`FFSEND_HOST` value. Both test uploads had five-minute expiry. After the fork
locked `tar` at 0.4.46, its release build also completed a live archive upload,
download, extraction, and content comparison.

## 3. Select your host

For a single upload, give the destination explicitly:

```bash
ffsend --api auto upload --host https://send.enkiel.org/ ./file.txt
```

When using an unwrapped upstream binary, set the host for the current terminal
session:

```bash
export FFSEND_HOST=https://send.enkiel.org/
```

`FFSEND_HOST` changes the **upload** destination. The personal `ffsend` wrapper
sets it automatically on every invocation. An explicit `--host` argument can
still override it. Download, info, and delete commands use the host in the
share link you supply. The unwrapped upstream binary defaults to the public
`send.vis.ee` service when `FFSEND_HOST` is absent. Check the host in every
returned link before sharing it. You can end a manually exported setting with
`unset FFSEND_HOST`; the wrapper still selects your host on its next run.

Do not set `FFSEND_BASIC_AUTH` unless your deployment actually uses HTTP Basic
Auth at a proxy. It is distinct from a file's optional download password.

## 4. Try a harmless round trip

Run this only after the service is reachable. It uploads a disposable text file
for five minutes, downloads it to a temporary directory, and compares the two
copies. This creates a real upload on your service.

```bash
(
set -e
test_dir=$(mktemp -d)
trap 'rm -rf -- "$test_dir"' EXIT
printf 'ffsend smoke test\n' > "$test_dir/source.txt"
link=$(ffsend --api auto --no-interact --incognito upload --host https://send.enkiel.org/ \
  --quiet --download-limit 2 --expiry-time 5m "$test_dir/source.txt")
case "$link" in
  https://send.enkiel.org/*) ;;
  *) printf 'Unexpected upload host\n' >&2; exit 1 ;;
esac
ffsend --api auto --no-interact --incognito download \
  --output "$test_dir/received.txt" "$link"
cmp "$test_dir/source.txt" "$test_dir/received.txt"
printf 'Round trip passed; the test link expires after five minutes.\n'
)
```

The link is kept in a shell variable during this test; avoid printing it into
shared logs. `--download-limit 2` leaves room for one retry. The sample server
configuration includes this count and a five-minute expiry, but the live
deployment may differ. If the server rejects either value, inspect its actual
settings before changing the test.

The temporary local files are removed when the command block finishes. Link
expiry makes the uploaded file unavailable through Send; storage cleanup is a
separate server responsibility.

## 5. Everyday terminal use

With the personal wrapper, upload an individual file:

```bash
ffsend --api auto upload --download-limit 2 --expiry-time 1h ./file.txt
```

To download, quote the entire share link. The part after `#` contains the
decryption secret. Do not paste a real link into public issues, logs, or chat:

```bash
ffsend --api auto download --output ./received.txt '<share-link>'
```

For a private file password, use `--password` without a value in an interactive
terminal so the client prompts for it. Putting the password directly in a
command may expose it through shell history or process listings. The share link
and any separate password must both reach the intended recipient.

## 6. File handoffs between AI agents

Use Send for the **file payload**. Use your existing trusted agent communication
channel to deliver the link and explain who should read it. Send does not
provide recipient identity, a message queue, delivery confirmation, or a
permanent record of the conversation.

For an unattended sender, set the host in that process, disable prompts, and
avoid saving the link and owner token in local `ffsend` history:

```bash
FFSEND_HOST=https://send.enkiel.org/ FFSEND_INCOGNITO=1 \
  ffsend --api auto --no-interact upload --quiet \
  --download-limit 2 --expiry-time 1h ./handoff.json
```

Pass the returned link to the intended recipient through the existing channel.
Include the file's purpose and expiry there. The recipient should verify that
the link begins with `https://send.enkiel.org/`, save to a new path, and process
the file only after deciding that its sender and contents are trusted. A shell
example, with `link` supplied securely by the existing channel, is:

```bash
case "$link" in
  https://send.enkiel.org/*) ;;
  *) printf 'Unexpected download host\n' >&2; exit 1 ;;
esac
ffsend --api auto --no-interact --incognito download \
  --output ./received-handoff.json "$link"
```

Keep links out of public logs and long-lived transcripts. Prefer short expiry
and a small number of downloads. Avoid `--force` and `--yes` in agent jobs: they
can bypass warnings or allow overwrites. Do not send archives to agents with
this workflow. With the default archive feature, a non-interactive download of
an uploaded archive accepts extraction by default. Agent jobs should use
individual files until that behavior is explicitly controlled and tested.

`FFSEND_INCOGNITO` is enabled by its **presence**, even if set to `0`. It stops
local history updates, which also means the sender may lack the saved owner
token needed to delete the upload early. Choose between history and incognito
deliberately for each workflow.

## Troubleshooting

- `ffsend: command not found`: install the client and verify it is on `PATH`.
- DNS or TLS errors: check that the domain resolves and its certificate is
  valid. Do not turn off certificate verification to make a transfer pass.
- Unexpected upload host: check `FFSEND_HOST` or use `--host` explicitly.
- API version error: retry with `--api auto` and check the server version.
- Limits rejected: compare the requested count and expiry with the deployed
  Send configuration. Do not add `--force` blindly.
- Download fails after a successful upload: confirm the full link, including
  its `#` fragment, was delivered intact and has not expired or reached its
  download limit.
