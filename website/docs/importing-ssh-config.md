# Importing SSH config

SSHDeck parses common OpenSSH config blocks from:

```text
~/.ssh/config
```

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

## Managed hosts

Hosts created inside SSHDeck are written to:

```text
~/.config/sshdeck/ssh_config
```

This keeps app-managed blocks separate from your handcrafted SSH config.

## Limitations

The parser covers common config patterns. Exotic Match blocks, includes with complex expansion, and every OpenSSH edge case are not fully modeled yet.
