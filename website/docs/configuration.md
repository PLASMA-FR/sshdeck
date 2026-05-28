# Configuration

SSHDeck stores application settings at:

```text
~/.config/sshdeck/config.toml
```

## Example

```toml
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

## Managed OpenSSH config

SSHDeck-created OpenSSH host blocks live at:

```text
~/.config/sshdeck/ssh_config
```
