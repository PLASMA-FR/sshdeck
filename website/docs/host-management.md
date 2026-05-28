# Host management

SSHDeck reads imported hosts from `~/.ssh/config` and writes app-created hosts to its own managed config file.

## Managed config path

```text
~/.config/sshdeck/ssh_config
```

## App metadata path

```text
~/.config/sshdeck/config.toml
```

Metadata includes groups, tags, favorites, notes, bookmarks, UI preferences, and file-manager preferences.

## Add a host

Press `a` or click Add Host. Fill in:

- Alias
- Hostname or IP
- User
- Port
- Identity file
- Group
- Tags
- Notes

SSHDeck validates required fields, numeric ports, leading-dash aliases, and config-control characters before writing managed config.

## Include managed config

After saving a managed host, SSHDeck can offer an Include line:

```text
Include ~/.config/sshdeck/ssh_config
```

The original SSH config is backed up before this line is appended.

## Delete behavior

Managed hosts are removed from SSHDeck's managed config after confirmation. Imported hosts are hidden only from the current SSHDeck view and may reappear after restart until persistent imported-host hiding is implemented.
