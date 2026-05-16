# SSHDeck

<p align="center">
  <img src="docs/images/logo.png" alt="SSHDeck logo" width="180" />
</p>

> SSHDeck — Termius + Yazi for your terminal. A clean, local-first SSH command center built in Rust.

No cloud. No account. No Electron. Just your terminal and your existing OpenSSH config.

SSHDeck is a fast keyboard-first and mouse-friendly terminal SSH command center for managing SSH hosts, tunnels, commands, health checks, logs, and Yazi-style remote file workflows.

![SSHDeck black theme dashboard](docs/images/blackout-dashboard.png)

![SSHDeck dashboard screenshot](docs/images/dashboard.png)

## Animated demo

GIF placeholders:

- Add-host modal screenshot: `docs/images/add-host.png`
- Dashboard GIF: `docs/images/dashboard.gif`
- Mouse interaction GIF: `docs/images/mouse.gif`
- Files/SFTP GIF: `docs/images/files.gif`
- Tunnel builder GIF: `docs/images/tunnel.gif`

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
- in-app add/edit/duplicate/delete host management
- managed OpenSSH config writer at `~/.config/sshdeck/ssh_config`
- safe config backups in `~/.config/sshdeck/backups/`
- full mouse support: click, double-click, right-click, scroll, modal buttons
- hover highlights for clickable rows, breadcrumbs, status shortcuts, and buttons
- button-like interactive controls with primary/secondary/danger states
- fuzzy search
- groups/tags/favorites
- SSH config import
- connection launcher using system OpenSSH
- port forwarding builder
- remote command runner
- server health dashboard
- Yazi-style SFTP file manager
- transfer queue
- safe remote editing with backups design
- command palette
- themes: blackout default, cyber, minimal
- Unicode animations with ASCII/no-animation fallback
- local-first config

## Why SSHDeck

SSHDeck is built around a simple idea: your terminal should have a polished SSH command center without requiring a cloud account, Electron, or a rewritten SSH stack.

It reads `~/.ssh/config`, stores SSHDeck-only metadata in `~/.config/sshdeck/config.toml`, and uses your existing `ssh`, `scp`, and `sftp` tools where appropriate.

For normal use you do not need to manually edit SSH config. Press `a` or click `[+ Add Host]` to create a host inside SSHDeck. App-created hosts are written to:

```text
~/.config/sshdeck/ssh_config
```

SSHDeck then offers to add this safe OpenSSH include line to `~/.ssh/config` after creating a timestamped backup:

```sshconfig
Include ~/.config/sshdeck/ssh_config
```

Tagline:

“Termius for the terminal. No cloud. No account. No Electron.”

## Comparison

| Feature | SSHDeck | Termius | Plain SSH |
|---|---:|---:|---:|
| Terminal-native | Yes | No | Yes |
| Local-first | Yes | No | Yes |
| No account required | Yes | No | Yes |
| Reads ~/.ssh/config | Yes | Partial | Yes |
| Fuzzy host search | Yes | Yes | No |
| Mouse support | Yes | Yes | No |
| Right-click context menus | Yes | Yes | No |
| Port forwarding UI | Yes | Yes | No |
| Remote command runner | Yes | Partial | Manual |
| Server health dashboard | Yes | No | Manual |
| SFTP file manager | Yes | Yes | Manual |
| Yazi-like remote file browser | Yes | Partial | No |
| Safe remote editing backups | Yes | Partial | Manual |
| Open source | Yes | No | Yes |

## Host management

Create hosts without leaving the app:

- press `a` or click `[+ Add Host]`
- fill alias, hostname/IP, user, port, identity file, group, tags, and notes
- use Tab / Shift+Tab or mouse clicks to move between fields
- click `[ Test ]` or focus it and press Enter to run a safe SSH check:
  `ssh -o BatchMode=yes -o ConnectTimeout=5 <target> exit`
- click `[ Save ]`, press Enter on Save, or press Ctrl+s
- SSHDeck writes OpenSSH host blocks to `~/.config/sshdeck/ssh_config`
- tags, groups, favorites, and notes stay in `~/.config/sshdeck/config.toml`

Editing and deletion:

- `e` or the Edit button opens the selected host in the form
- `Shift+d` or the command palette duplicates the selected host with `-copy`
- `d` opens a delete confirmation
- imported hosts are hidden/metadata-removed rather than destructively removed from your original `~/.ssh/config`
- managed hosts are removed from SSHDeck's managed config after backup and confirmation

Validation:

- Alias and Hostname/IP are required
- Port must be numeric
- User defaults to the current local username when available
- Identity file is optional, but SSHDeck warns if the path does not exist
- Alias spaces and alias conflicts are shown as warnings

![Add Host modal](docs/images/add-host.png)

## Mouse-first and keyboard-first

SSHDeck is designed for both terminal power users and people coming from GUI SSH apps.

You can:

- buttons highlight on hover so clickable areas are obvious
- click hosts, files, breadcrumbs, status shortcuts, and modal/context-menu buttons
- right-click for context menus
- scroll host, file, and preview panels
- click buttons
- click breadcrumbs
- use full keyboard shortcuts
- combine mouse navigation with Vim-style keyboard commands

Mouse support is implemented through crossterm mouse capture and a maintainable hit-test registry (`src/mouse.rs`). Each render pass registers clickable regions such as sidebar groups, host cards, quick action buttons, file entries, breadcrumbs, command palette items, modal buttons, tabs, and transfer items.

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
s         open files/SFTP
t         tunnel builder
r         run command
h         health check
l         logs
Ctrl+p    command palette
?         help
```

## Yazi-style SFTP File Manager

SSHDeck includes a fast keyboard-first remote file manager inspired by Yazi and Ranger.

Features:

- three-column remote navigation
- full mouse interaction with hover feedback
- optional local/remote dual-pane mode
- preview panel
- Vim-style keybindings
- upload/download queue
- bookmarks
- hidden files toggle
- safe remote editing with backups architecture
- command mode
- Unicode animations
- local-first configuration

![SSHDeck Files screenshot](docs/images/files.png)

Files shortcuts:

```text
j/k         move
h/l         parent/open
g/G         top/bottom
/           search
.           hidden files
Space       select
v           visual mode
V           select all
Ctrl+r      clear selection
y           copy/yank
x           cut
p           paste
u           upload
d           download
D           delete
r           rename
n           new file/folder
e           edit
c           copy path
Tab         dual-pane
T           transfers
b           bookmarks
:           command mode
```

## Tunnel builder

SSHDeck can generate safe OpenSSH tunnel commands:

```bash
ssh -L 8080:localhost:80 web-prod-1
ssh -R 9000:localhost:9000 web-prod-1
ssh -D 1080 web-prod-1
```

![SSHDeck tunnel screenshot](docs/images/tunnel.png)

## Configuration

SSHDeck creates and reads:

```text
~/.config/sshdeck/config.toml
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

[bookmarks.web-prod-1]
webroot = "/var/www/app"
logs = "/var/log"

[settings]
default_command = "ssh"
```

SSHDeck does not blindly rewrite your `~/.ssh/config`. Managed hosts are designed to live in SSHDeck's own managed config, with an optional OpenSSH include:

```sshconfig
Include ~/.config/sshdeck/ssh_config
```

## Safety notes

SSHDeck is designed to be conservative:

- never print private key contents
- never expose `.env` contents by default
- never log secrets or remote file contents
- never auto-delete SSH config
- backup config before modification
- confirm before deleting hosts or remote files
- confirm before overwriting remote files
- backup before editing remote files
- warn before destructive commands
- quote remote paths safely
- never allow deleting `/`

Dangerous command patterns are blocked or require explicit confirmation, including `rm -rf`, `mkfs`, `dd`, `shutdown`, `reboot`, `passwd`, `userdel`, firewall flushes, and recursive permission changes.

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

`sshdeck doctor` checks terminal mouse reporting, mouse config, terminal size, color support, Unicode/Nerd Font settings, OpenSSH binaries, SSH config parsing, managed config path, SSH directory state, identity files referenced by hosts, app config validity, and the default local files directory.

## Screenshot and GIF plan

1. Dashboard:
   - click host
   - right-click menu
   - double-click connect
2. Files:
   - open files
   - click folder
   - scroll preview
   - right-click file
3. Tunnels:
   - click tunnel builder
   - choose local/remote/dynamic tunnel type
   - create local forward

The target demo should communicate, without narration, that SSHDeck is a beautiful local-first SSH command center with keyboard and mouse workflows for SSH, files, tunnels, commands, and health.

## Roadmap

v0.1:
- SSH config import
- host list
- fuzzy search
- connect
- groups/tags
- command palette

v0.2:
- tunnel builder
- command runner
- health panel
- logs

v0.3:
- SSHDeck Files
- Yazi-style remote file browser
- upload/download
- remote preview
- safe remote editing
- transfer queue
- bookmarks
- dual-pane mode

v0.4:
- tmux integration
- multi-host command execution
- encrypted local vault

v0.5:
- Tailscale device discovery
- Cloudflare Tunnel awareness
- backup/sync through Git
- native SFTP backend if reliable

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

The MVP contains tests for SSH config parsing, app config loading, command generation, tunnel command generation, dangerous command detection, file path safety checks, remote command quoting helpers, file entry parsing, bookmark config loading, and transfer queue state transitions.

## Contributing

Contributions are welcome. Please keep SSHDeck local-first, safe by default, terminal-native, and respectful of existing OpenSSH configuration.

Before opening a PR:

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt
```

If `rustfmt` or `clippy` are not installed, install the Rust components for your toolchain and rerun the checks.

## License

MIT
