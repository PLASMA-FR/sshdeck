# Importing SSH config

SSHDeck parses common OpenSSH config blocks from:

```text
~/.ssh/config
```

The TUI reads this file at startup. The `sshdeck import` command is currently a non-destructive parse/count helper: it reports how many hosts are visible and ensures the SSHDeck app config exists.

Supported fields include:

- Host
- HostName
- User
- Port
- IdentityFile
- ProxyJump
- LocalForward
- RemoteForward
- ForwardAgent
- ServerAliveInterval

## Safety model

SSHDeck does not rewrite complex user SSH config during normal import. It reads the file and stores SSHDeck-specific metadata separately.

Imported hosts connect by alias so OpenSSH can keep applying config directives that SSHDeck does not model yet.

## Managed hosts

Hosts created inside SSHDeck are written to:

```text
~/.config/sshdeck/ssh_config
```

This keeps app-managed blocks separate from your handcrafted SSH config.

## Limitations

The parser covers common config patterns. Exotic Match blocks, includes with complex expansion, and every OpenSSH edge case are not fully modeled yet.

The app config now has a reserved `hidden_imported_hosts` field, but the current TUI does not restore hidden imported hosts across restarts yet.
