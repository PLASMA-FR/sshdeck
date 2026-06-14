# Configuration

SSHDeck stores application settings at:

```text
~/.config/sshdeck/config.toml
```

## Example

```toml
hidden_imported_hosts = []
recent_hosts = []

[ui]
theme = "blackout"
animations = true
unicode = true
nerd_font = true
mouse = true

[files]
default_local_dir = "~/Downloads"
show_hidden = false
preview_max_bytes = 1048576

[settings]
default_command = "ssh"

[last_paths]
local = "~/Downloads"

[last_paths.remote_by_host]
web-prod-1 = "/var/www"

[tunnel_presets.web]
tunnel_type = "local"
host_alias = "web-prod-1"
local_port = 8080
target_host = "localhost"
target_port = 80
notes = "Local web app"

[bookmarks.global]
downloads = "~/Downloads"
home = "~"
```

## Host metadata

Host-specific metadata belongs in app config, not in `~/.ssh/config`:

```toml
[hosts.web-prod-1]
tags = ["production", "web"]
group = "Production"
favorite = true
notes = "Main production web server"
```

## State fields

The config schema is intentionally backward-compatible. Missing fields are filled with defaults when SSHDeck loads older config files.

- `hidden_imported_hosts`: reserved list of imported `~/.ssh/config` aliases that should stay hidden once the TUI restore path is wired.
- `recent_hosts`: reserved most-recently-used alias list.
- `last_paths.local`: reserved last local Files path.
- `last_paths.remote_by_host`: reserved last remote Files path by host alias.
- `tunnel_presets`: reserved named tunnel presets for local, remote, or dynamic forwards.
- `bookmarks`: persisted path bookmarks. The default `global` group contains `downloads` and `home`; the full bookmarks UI is still roadmap work.

## Managed OpenSSH config

SSHDeck-created OpenSSH host blocks live at:

```text
~/.config/sshdeck/ssh_config
```
