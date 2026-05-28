# Remote commands

The command runner is designed to run safe commands over SSH and show output in a scrollable panel.

## Intended safe commands

- `uptime`
- `df -h`
- `free -h`
- `docker ps`
- `systemctl --failed`
- `journalctl -xe`

## Dangerous command detection

SSHDeck includes helpers that flag destructive patterns such as:

- `rm -rf`
- `mkfs`
- `dd`
- `shutdown`
- `reboot`
- `passwd`
- `userdel`
- `iptables flush`
- `nft flush`
- `chmod -R 777`
- `chown -R`

## Current status

The UI prototype and safety helpers exist. TUI-backed remote command execution is not complete yet.
