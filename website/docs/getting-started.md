# Getting started

SSHDeck is a terminal app. It reads your existing SSH config, stores SSHDeck metadata locally, and launches system OpenSSH for connections.

## Requirements

- Rust toolchain
- A terminal with color support
- OpenSSH tools: `ssh`, `scp`, and `sftp`
- Optional: Nerd Font for icons

## Install from source

```bash
curl -fsSL https://raw.githubusercontent.com/PLASMA-FR/sshdeck/main/scripts/install.sh | bash
```

Or from a cloned checkout:

```bash
git clone https://github.com/PLASMA-FR/sshdeck
cd sshdeck
bash scripts/install.sh
```

## First run

```bash
sshdeck
```

If no hosts are found, SSHDeck shows an empty state with options to add a host, import from `~/.ssh/config`, or open help.

## Check your environment

```bash
sshdeck doctor
```

Doctor checks OpenSSH tools, config parsing, local config validity, referenced identity files, terminal capabilities, and the default local files directory.
