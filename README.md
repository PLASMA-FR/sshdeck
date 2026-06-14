# SSHDeck

[![CI](https://github.com/PLASMA-FR/sshdeck/actions/workflows/ci.yml/badge.svg)](https://github.com/PLASMA-FR/sshdeck/actions/workflows/ci.yml)
[![GitHub Pages](https://github.com/PLASMA-FR/sshdeck/actions/workflows/pages.yml/badge.svg)](https://github.com/PLASMA-FR/sshdeck/actions/workflows/pages.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-8ce7ff.svg)](LICENSE)
![Rust](https://img.shields.io/badge/Rust-2024-111827?logo=rust)

> Termius + Yazi for your terminal.

SSHDeck is a clean, local-first SSH command center built in Rust. It helps you manage servers, SSH configs, files, tunnels, commands, health checks, and remote workflows from a polished terminal interface.

No cloud. No account. No Electron. No telemetry. Just your terminal and OpenSSH.

Website: https://plasma-fr.github.io/sshdeck/
Docs: https://plasma-fr.github.io/sshdeck/docs/

![SSHDeck black theme dashboard](docs/images/blackout-dashboard.png)

## Current status

SSHDeck is an early open-source MVP. It already provides a polished Rust/ratatui shell for host management, OpenSSH launching, managed host config, command palettes, broad mouse support, logs, and a prototype Yazi-style remote file browser.

Implemented today:

- reads hosts from `~/.ssh/config` at startup
- creates SSHDeck-managed hosts in `~/.config/sshdeck/ssh_config`
- offers an optional backed-up `Include ~/.config/sshdeck/ssh_config` addition
- add, edit, duplicate, and delete managed hosts from the TUI
- fuzzy host search
- keyboard-first dashboard and help screens
- broad mouse support: hover, click, double-click, right-click context menus, scrolling, modal buttons
- launches system `ssh` and restores the terminal afterward
- tunnel command generation for local, remote, and dynamic forwards
- command runner UI prototype with dangerous-command detection
- health panel placeholder with intended safe command list
- SSHDeck Files prototype: remote directory listing over `ssh`, three-column browsing, metadata preview, hidden-files toggle, refresh, breadcrumbs, transfer queue modal, and dual-pane UI placeholder
- settings screen for theme, animation, Unicode/Nerd Font, mouse, hidden files, and config path
- backward-compatible app config fields for bookmarks, hidden imported hosts, recent hosts, tunnel presets, and last file paths
- local logs with redaction for sensitive path markers and identity-file arguments
- `sshdeck doctor` for local environment checks
- unit tests for parsing, command generation, safety helpers, transfer queue state, and mouse hit regions

Not implemented yet:

- real SFTP/scp upload and download execution from the Files UI
- safe remote editing with backup/upload flow
- remote file delete, rename, new-file, chmod, and chown operations
- real command runner execution from the TUI
- remote health command execution and parsing
- TUI wiring for persistent imported-host hiding, recent hosts, tunnel presets, and last-path restore
- full bookmarks UI beyond the config schema
- true local-pane filesystem model in dual-pane Files mode
- live starting and stopping of tunnels from the TUI

## Install

SSHDeck is not published on crates.io yet. Install from source:

```bash
git clone https://github.com/PLASMA-FR/sshdeck
cd sshdeck
bash scripts/install.sh
sshdeck
```

Direct Cargo install from the checkout also works:

```bash
cargo install --locked --path .
```

Development commands:

```bash
cargo run --locked
cargo run --locked -- doctor
cargo run --locked -- import
cargo test --locked
```

## Quickstart

```bash
sshdeck doctor
sshdeck
```

Inside the TUI:

```text
/         search hosts
a         add host
e         edit host
Enter     connect with system ssh
s         open SSHDeck Files prototype
t         tunnel command generator
r         command runner prototype
h         health panel placeholder
Ctrl+p    command palette
?         help
```

Quick connect from the shell:

```bash
sshdeck user@host
```

CLI reference:

```text
sshdeck [OPTIONS] [TARGET] [COMMAND]

Commands:
  doctor          check local OpenSSH tools, config, terminal, and defaults
  import          parse ~/.ssh/config, report count, and create app config if needed

Options:
  --config <PATH>       use a specific SSHDeck config.toml
  --theme <THEME>       override the configured theme for this run
  --no-animations       disable animations for this run
  --ascii               force ASCII-friendly rendering
  --mouse               force mouse capture on for this run
  --no-mouse            disable mouse capture for this run
  --quick <TARGET>      quick-connect with system ssh
  -h, --help            print help
  -V, --version         print version
```

`TARGET` and `--quick <TARGET>` both bypass the TUI and run system `ssh -- <TARGET>`.

Config state fields now accepted in `~/.config/sshdeck/config.toml`:

- `hidden_imported_hosts = []`
- `recent_hosts = []`
- `[last_paths]` with `local` and `[last_paths.remote_by_host]`
- `[tunnel_presets.<name>]`
- `[bookmarks.<group>]`

These fields are backward-compatible. Missing fields default safely; several are reserved until the TUI restore/preset/bookmark workflows are wired.

## Features

- SSH host dashboard
- in-app add, edit, duplicate, and delete host management for SSHDeck-managed hosts
- managed OpenSSH config writer at `~/.config/sshdeck/ssh_config`
- backed-up config changes before managed config or include-line modifications
- broad mouse support for implemented views
- fuzzy search
- groups, tags, favorites, and notes in local metadata
- reserved config state for hidden imported hosts, recent hosts, tunnel presets, last paths, and bookmarks
- connection launcher using system OpenSSH
- port forwarding command generator
- command runner prototype with dangerous-command blocking
- health panel placeholder
- prototype Yazi-style remote file browser using `ssh` directory listing
- transfer queue data model and UI placeholder
- command palette
- themes: blackout default, cyber, minimal
- Unicode animations with ASCII and no-animation fallback
- local-first config

## SSHDeck Files

SSHDeck Files is a Yazi-inspired remote file workflow and one of the project's headline features.

Current prototype:

- three-column remote navigation
- metadata preview panel
- Vim-style movement foundations
- hidden-files toggle
- breadcrumbs
- mouse selection and right-click context menu
- transfer queue state and modal
- dual-pane layout placeholder
- sensitive path helper checks

Roadmap work:

- upload and download execution
- safe remote editing with backups
- remote mutation commands
- bookmarks UI
- native SFTP backend or structured remote listing

Read more: https://plasma-fr.github.io/sshdeck/docs/sshdeck-files/

## Safety model

SSHDeck is designed to be conservative:

- no cloud service
- no account
- no telemetry
- no private key storage
- no blind rewrites of complex user SSH config
- separate managed OpenSSH config for app-created hosts
- timestamped backups before include-line changes
- argument-vector command construction where possible
- destination separators to reduce leading-dash option injection risk
- dangerous command pattern detection in helpers
- sensitive path preview guards in file helpers
- local log redaction for common sensitive path markers

Security review: docs/SECURITY_REVIEW.md
QA checklist: docs/QA.md

## Documentation website

The website lives in `website/` and is built with VitePress plus a small Three.js hero component.

Run it locally:

```bash
npm install
npm run docs:dev
```

Build it:

```bash
npm run docs:build
```

GitHub Pages workflow:

```text
.github/workflows/pages.yml
```

## Comparison

| Feature | SSHDeck | Termius | Plain SSH |
|---|---:|---:|---:|
| Terminal-native | Yes | No | Yes |
| Local-first | Yes | No | Yes |
| No account required | Yes | No | Yes |
| Reads `~/.ssh/config` | Yes | Partial | Yes |
| Fuzzy host search | Yes | Yes | No |
| Mouse support | Partial/broad | Yes | No |
| Right-click context menus | Partial | Yes | No |
| Port forwarding UI | Partial: command generator | Yes | No |
| Remote command runner | Prototype | Partial | Manual |
| Server health dashboard | Placeholder | No | Manual |
| SFTP file manager | Roadmap; ssh listing exists | Yes | Manual |
| Yazi-style remote file browser | Partial | Partial | No |
| Safe remote editing backups | Roadmap | Partial | Manual |
| Open source | Yes | No | Yes |

## Roadmap

v0.1:

- SSH config import
- host list
- fuzzy search
- connect
- managed host config
- groups, tags, favorites
- command palette
- mouse hit registry

v0.2:

- tunnel command generation
- command runner UI
- health panel shell
- logs
- settings

v0.3:

- SSHDeck Files execution workflows
- upload/download
- remote preview improvements
- safe remote editing
- transfer queue execution
- bookmarks
- dual-pane local and remote file model

v0.4:

- tmux integration
- multi-host command execution
- live tunnel process management
- encrypted local vault

v1.0:

- stable plugin system
- packaged binaries
- theme gallery
- polished demo GIFs
- full safety review

## Contributing

Please run the checks that are available in your environment:

```bash
cargo fmt
cargo check --locked
cargo clippy --locked -- -D warnings
cargo test --locked
npm run docs:build
```

If `cargo fmt` or `cargo clippy` is not installed, note that in your PR.

Do not commit private SSH configs, keys, credentials, `.env` files, real server IPs, `target/`, or generated website output.

## License

MIT. See [LICENSE](LICENSE).
