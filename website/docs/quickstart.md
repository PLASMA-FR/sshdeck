# Quickstart

## 1. Run doctor

```bash
sshdeck doctor
```

Fix missing OpenSSH binaries or invalid config before relying on the TUI.

## 2. Start SSHDeck

```bash
sshdeck
```

## 3. Add or read hosts

Inside the TUI:

- `a` opens Add Host
- `/` searches hosts
- `Enter` connects to the selected host
- `i` opens host detail and access profile

SSHDeck reads `~/.ssh/config` at startup. From the shell, `sshdeck import` parses that file and reports how many hosts SSHDeck can see; it does not rewrite your OpenSSH config.

## 4. Open common workflows

- `s` opens SSHDeck Files
- `t` opens the tunnel builder
- `r` opens the command runner
- `h` runs a remote health check
- `Ctrl+p` opens the command palette
- `?` opens help

## 5. Know what is still rough

Native SFTP, byte-accurate transfer progress, transfer retries, overwrite prompts, full bookmark picker UI, tunnel preset editing, and enterprise access-plane features are still roadmap items.
