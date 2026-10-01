# ffsend — Enkiel's Send client

This is my personal fork of [ffsend](https://github.com/timvisee/ffsend), set up
for [send.enkiel.org](https://send.enkiel.org/). It encrypts files on your
computer before uploading them and gives you a link to share. The command-line
client and the Send website are separate projects.

The Linux installer in this fork makes `send.enkiel.org` the default upload host.
An ordinary upload allows **one download** and expires after **five minutes**.
You can change either limit for an individual upload. This fork is for personal
terminal use; agent-to-agent communication is planned but is not implemented.

## Quick start

```bash
ffsend upload ~/Documents/example.txt
ffsend download 'https://send.enkiel.org/download/EXAMPLE#SECRET'
ffsend history --active
ffsend --help
```

A share link contains the key needed to decrypt the file. Give it only to the
people who should receive the file. `ffsend history --active` checks links saved
on this computer against their Send servers; it does not list browser uploads
or uploads from another computer.

To allow two downloads for one hour:

```bash
ffsend upload --download-limit 2 --expiry-time 1h ~/Documents/example.txt
```

To check a particular link without downloading it:

```bash
ffsend exists 'SHARE_URL'
```

See [the personal setup guide](docs/enkiel-setup.md) for more commands, testing,
troubleshooting, and the Send service relationship.
To have a terminal-capable AI agent install or use this fork, give it the
[agent setup prompt](docs/agent-setup-prompt.md) with your chosen task filled in.

## Install or update on Linux

The supported personal installer is for **Linux x86_64**. It downloads a fresh
checkout of this fork's `master` branch, builds it with Cargo, and installs the
command under `~/.local/`. It does not use `sudo` or deploy the Send website.
Run the same command again to update to the latest pushed commit:

```bash
curl -fsSL https://raw.githubusercontent.com/njinco/ffsend/master/scripts/install-from-github.sh | bash
```

This command runs the [bootstrap script](scripts/install-from-github.sh) from
this fork. Review the script first if you want to inspect what will run. It
clones this repository into a temporary directory, then runs the
[local installer](scripts/install-enkiel.sh). It builds from source; this fork
does not currently provide a prebuilt download. You need `git`, `curl`,
`pkg-config`, OpenSSL development libraries, and Rust with Cargo. If Cargo is
missing, the installer offers to install Rust using the official rustup script
when run in an interactive terminal. The build may take several minutes.

After installation:

```bash
ffsend
ffsend debug
command -v ffsend
```

The wrapper should be at `~/.local/bin/ffsend`. If your shell cannot find it,
start a new terminal or add `~/.local/bin` to your `PATH`. You can also run it
as `~/.local/bin/ffsend`.

### Install from an existing checkout

If you already cloned this repository, install exactly the commit you have
checked out:

```bash
cd ~/Documents/GitHub/ffsend
git status --short
bash scripts/install-enkiel.sh
```

The checkout must be clean so the installed binary's filename identifies the
source commit. The installed wrapper points to `send.enkiel.org`; use
`ffsend upload --host https://OTHER-SEND-HOST/ FILE` for another Send host.
The project version remains `0.2.77`, so use `ffsend debug` or inspect the
wrapper to confirm which fork build is installed.

## What this fork changes

- Makes `send.enkiel.org` the default host in the personal Linux installer.
- Defaults Send v3 uploads to one download and five minutes.
- Adds `ffsend history --active` for live checks of locally saved links.
- Keeps the existing ffsend upload, download, encryption, and management
  commands. Run `ffsend --help` for the full command list.

The [original ffsend project](https://github.com/timvisee/ffsend) was created
by Tim Visee. This fork is licensed under [GPL-3.0](LICENSE) and is not
affiliated with Firefox or Mozilla. For development details, see
[CONTRIBUTING.md](CONTRIBUTING.md) and [the setup guide](docs/enkiel-setup.md).
