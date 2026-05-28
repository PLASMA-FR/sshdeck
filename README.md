# SSHDeck

> Termius for the terminal — a local-first SSH command center built in Rust.

No cloud. No account. No Electron. Just your terminal and your existing OpenSSH config.

SSHDeck is an early open-source MVP. It already provides a polished Rust/ratatui shell for SSH host management, OpenSSH launching, managed host config, command palettes, broad mouse support, and a prototype Yazi-style remote file browser. Some headline workflows are intentionally marked partial below instead of being oversold.

![SSHDeck black theme dashboard](docs/images/blackout-dashboard.png)
![SSHDeck dashboard screenshot](docs/images/dashboard.png)

## Status

Implemented today:

- reads hosts from `~/.ssh/config` at startup
- creates SSHDeck-managed hosts in `~/.config/sshdeck/ssh_config`
- offers an optional backed-up `Include ~/.config/sshdeck/ssh_config` addition
- add/edit/duplicate/delete managed hosts from the TUI
- fuzzy host search
- keyboard-first dashboard and help screens
- broad mouse support: hover, click, double-click, right-click context menus, scrolling, modal buttons
- launches system `ssh` and restores the terminal afterward
- tunnel command generation for local, remote, and dynamic forwards
- command runner UI prototype with dangerous-command detection
- health panel placeholder with intended safe command list
- SSHDeck Files prototype: remote directory listing over `ssh`, three-column browsing, metadata preview, hidden-files toggle, refresh, breadcrumbs, transfer queue modal, and dual-pane UI placeholder
- settings screen for theme, animation, Unicode/Nerd Font, mouse, hidden files, and config path
- local logs with redaction for sensitive path markers and identity-file arguments
- doctor command for local environment checks
- unit tests for parsing, command generation, safety helpers, transfer queue state, and mouse hit regions

Not implemented yet:

- real SFTP/scp upload/download execution from the Files UI
- safe remote editing with backup/upload flow
- remote file delete/rename/new-file operations
- real command runner execution from the TUI
- remote health command execution/parsing
- persistent imported-host hiding across restarts
- full bookmarks UI
- true local-pane filesystem model in dual-pane Files mode
- live starting/stopping of tunnels from the TUI

## Quick install

From source:

```bash
cargo install --path .
sshdeck
```

Run during development:

```bash
cargo run
cargo run -- doctor
cargo run -- import
```

After publishing:

```bash
cargo install sshdeck
```

## Features

- SSH host dashboard
- in-app add/edit/duplicate/delete host management for SSHDeck-managed hosts
- managed OpenSSH config writer at `~/.config/sshdeck/ssh_config`
- backed-up config changes before managed config or include-line modifications
- broad mouse support for implemented views
- fuzzy search
- groups/tags/favorites metadata storage
- connection launcher using system OpenSSH
- port forwarding command generator
- command runner prototype with dangerous-command blocking
- health panel placeholder
- prototype Yazi-style remote file browser using `ssh` directory listing
- transfer queue data model and UI placeholder
- command palette
- themes: blackout default, cyber, minimal
- Unicode animations with ASCII/no-animation fallback
- local-first config

## Why SSHDeck

SSHDeck is built around a simple idea: your terminal should have a polished SSH command center without requiring a cloud account, Electron, or a rewritten SSH stack.

It reads `~/.ssh/config`, stores SSHDeck-only metadata in `~/.config/sshdeck/config.toml`, and uses your existing OpenSSH tools where appropriate.

For normal use you do not need to manually edit SSH config. Press `a` or click `[+ Add Host]` to create a host inside SSHDeck. App-created hosts are written to:

```text
~/.config/sshdeck/ssh_config
```

SSHDeck then offers to add this OpenSSH include line to `~/.ssh/config` after creating a timestamped backup:

```sshconfig
Include ~/.config/sshdeck/ssh_config
```

Core promise:

No cloud. No account. No Electron. No AI API. No tracking. No lock-in.

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

## Host management

Create hosts without leaving the app:

- press `a` or click `[+ Add Host]`
- fill alias, hostname/IP, user, port, identity file, group, tags, and notes
- use Tab / Shift+Tab or mouse clicks to move between fields
- click `[ Test ]` or focus it and press Enter to run:
  `ssh -o BatchMode=yes -o ConnectTimeout=5 -- <target> exit`
- click `[ Save ]`, press Enter on Save, or press Ctrl+s
- SSHDeck writes OpenSSH host blocks to `~/.config/sshdeck/ssh_config`
- tags, groups, favorites, and notes stay in `~/.config/sshdeck/config.toml`

Editing and deletion:

- `e` or the Edit button opens the selected host in the form
- `Shift+d` or the command palette duplicates the selected host with `-copy`
- `d` opens a delete confirmation
- deleting an imported host only removes it from the current SSHDeck view and metadata; SSHDeck does not modify the original `~/.ssh/config`, and imported hosts may reappear on restart
- managed hosts are removed from SSHDeck's managed config after confirmation, with a backup of the managed config

Validation:

- Alias and Hostname/IP are required
- Port must be numeric
- aliases may not start with `-` or contain config-control characters
- User defaults to the current local username when available
- Identity file is optional, but SSHDeck warns if the path does not exist
- Alias spaces and alias conflicts are shown as warnings

![Add Host modal](docs/images/add-host.png)

## Mouse-first and keyboard-first

SSHDeck is designed for both terminal power users and people coming from GUI SSH apps.

You can:

- click hosts, files, breadcrumbs, status shortcuts, and modal/context-menu buttons
- right-click host and file rows for context menus
- double-click host rows to connect
- scroll host, file, and preview panels
- combine mouse navigation with Vim-style keyboard commands

Mouse support is implemented through crossterm mouse capture and a maintainable hit-test registry (`src/mouse.rs`). Some registered file/transfer actions are placeholders until the backing feature is implemented.

## Keyboard shortcuts

Navigation:

```text
↑/k       up
↓/j       down
Enter     connect/open
Esc       back/close modal
q         quit/back
```

Actions:

```text
/         search
a         add host
e         edit host
d         delete host
s         open SSHDeck Files prototype
t         tunnel command generator
r         command runner prototype
h         health panel placeholder
l         logs
,         settings
Ctrl+p    command palette
?         help
```

## SSHDeck Files prototype

SSHDeck includes a Yazi-inspired remote file browser prototype.

Implemented today:

- remote directory listing over `ssh` + `ls -la`
- three-column remote navigation
- metadata preview panel
- hidden-files toggle
- refresh
- mouse selection and right-click context menu
- breadcrumbs
- dual-pane layout placeholder
- transfer queue modal placeholder
- sensitive-path helper checks in preview helpers

Not implemented yet:

- real SFTP/scp upload/download execution from the UI
- remote file content preview in the main UI
- remote editing with backup/upload
- delete/rename/new-file operations
- bookmarks UI
- full visual selection
- real local-pane filesystem model

![SSHDeck Files screenshot](docs/images/files.png)

Currently implemented Files shortcuts:

```text
j/k         move
h/l         parent/open directory
Enter       open directory or select file
~           remote home
R           refresh
.           hidden files
Tab         show/switch dual-pane UI
T           transfer queue modal
:           command input placeholder
Esc/q       back
```

## Tunnel command generator

SSHDeck can generate OpenSSH tunnel commands:

```bash
ssh -L 8080:localhost:80 web-prod-1
ssh -R 9000:localhost:9000 web-prod-1
ssh -D 1080 web-prod-1
```

Starting/stopping live tunnel processes from the TUI is not implemented yet.

![SSHDeck tunnel screenshot](docs/images/tunnel.png)

## Configuration

SSHDeck creates and reads:

```text
~/.config/sshdeck/config.toml
```

Managed OpenSSH host blocks live at:

```text
~/.config/sshdeck/ssh_config
```

Example:

```toml
[ui]
theme = "blackout"
animations = true
unicode = true
nerd_font = true
mouse = true

[hosts.web-prod-1]
tags = ["production", "web"]
group = "Production"
favorite = true
notes = "Main production web server"

[files]
default_local_dir = "~/Downloads"
show_hidden = false
preview_max_bytes = 1048576

[bookmarks.global]
downloads = "~/Downloads"

[settings]
default_command = "ssh"
```

SSHDeck does not blindly rewrite your `~/.ssh/config`. Managed hosts are designed to live in SSHDeck's own managed config, with an optional OpenSSH include:

```sshconfig
Include ~/.config/sshdeck/ssh_config
```

## Safety notes

Implemented safety today:

- no SSH protocol reimplementation; system `ssh` is used
- OpenSSH destination arguments are separated from options with `--` where SSHDeck builds commands
- managed host validation blocks aliases that start with `-` and fields containing newlines/control characters
- managed config and include-line writes create backups
- config writes use temp-file then rename for lower corruption risk
- host deletion requires confirmation
- dangerous command patterns are detected and blocked in command input
- sensitive remote preview helper blocks paths such as `.env`, private keys, and `/etc/shadow`
- local logs redact common sensitive path markers and identity-file arguments
- remote paths are shell-quoted in listing/preview helper commands

Important limitations:

- remote file edit/delete/overwrite flows are not implemented yet, so their confirmations and backups are roadmap items
- the command runner UI does not execute remote commands yet
- the health panel does not execute remote health checks yet
- current remote file listing uses `ls -la`, which is pragmatic but not as robust as a native SFTP backend

Dangerous command patterns currently detected include `rm -rf`, `mkfs`, `dd`, `shutdown`, `reboot`, `passwd`, `userdel`, firewall flushes, and recursive permission changes.

## CLI

```bash
sshdeck
sshdeck --help
sshdeck --version
sshdeck --config ./config.toml
sshdeck --no-animations
sshdeck --no-mouse
sshdeck --mouse
sshdeck --ascii
sshdeck --quick root@1.2.3.4
sshdeck root@1.2.3.4
sshdeck import
sshdeck doctor
```

`sshdeck import` parses `~/.ssh/config`, reports how many host blocks SSHDeck can see, and initializes/saves SSHDeck config. It does not persistently copy imported hosts into the app config yet.

`sshdeck doctor` checks terminal mouse reporting, mouse config, terminal size, color support, Unicode/Nerd Font settings, OpenSSH binaries, SSH config parsing, managed config path, SSH directory state, identity files referenced by hosts, app config validity, and the default local files directory.

## Roadmap

v0.1:
- SSH config reading
- host list
- fuzzy search
- connect
- managed hosts
- command palette
- settings

v0.2:
- interactive tunnel builder inputs
- real command runner execution
- remote health execution/parsing
- logs polish

v0.3:
- SSHDeck Files real transfer execution
- native or robust SFTP backend
- remote preview with confirmation gates
- safe remote editing
- transfer queue execution/cancel/retry
- bookmarks UI
- full dual-pane local/remote model

v0.4:
- tmux integration
- multi-host command execution
- encrypted local vault

v0.5:
- Tailscale device discovery
- Cloudflare Tunnel awareness
- backup/sync through Git

v1.0:
- stable plugin system
- full docs
- packaged binaries
- theme gallery
- polished demo GIFs

## Development

```bash
cargo test
cargo run
cargo run -- doctor
cargo install --path .
```

The MVP contains tests for SSH config parsing, app config loading, command generation, tunnel command generation, dangerous command detection, file path safety checks, sensitive preview blocking, log redaction, remote command quoting helpers, file entry parsing, bookmark config loading, transfer queue state transitions, and mouse hit regions.

Before opening a PR:

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt
```

If `rustfmt` or `clippy` are not installed, install the Rust components for your toolchain and rerun the checks.

## Contributing

Contributions are welcome. Please keep SSHDeck local-first, safe by default, terminal-native, and respectful of existing OpenSSH configuration.

## License

MIT
