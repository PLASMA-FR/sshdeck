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

## 3. Add or import hosts

Inside the TUI:

- `a` opens Add Host
- `i` imports from `~/.ssh/config` where available
- `/` searches hosts
- `Enter` connects to the selected host

## 4. Open common workflows

- `s` opens SSHDeck Files prototype
- `t` opens tunnel command generation
- `r` opens command runner prototype
- `h` opens the health panel placeholder
- `Ctrl+p` opens the command palette
- `?` opens help

## 5. Know what is partial

The MVP is honest: file transfer execution, remote editing, live tunnel process management, command execution from the TUI, and remote health parsing are still roadmap items.
