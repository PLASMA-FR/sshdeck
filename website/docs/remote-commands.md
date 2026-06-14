# Remote commands

The command runner runs commands over system OpenSSH, shows output in the TUI, and caps execution time and captured output.

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

TUI-backed remote command execution is implemented. Normal commands run in a background task with a 20 second timeout and a 64 KiB output cap. Dangerous patterns require a typed confirmation before execution.

The detector is deliberately conservative and pattern-based. It is a last guardrail, not a shell sandbox.
