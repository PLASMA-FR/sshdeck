# Tunnels

SSHDeck includes a tunnel builder for local, remote, and dynamic forwards. The TUI can show, start, stop, and monitor the current tunnel process.

## Supported command shapes

Local forward:

```bash
ssh -L 8080:localhost:80 web-prod-1
```

Remote forward:

```bash
ssh -R 9000:localhost:3000 web-prod-1
```

Dynamic SOCKS forward:

```bash
ssh -D 1080 web-prod-1
```

## Current status

Command generation, live start, stop, and process polling are present. When the selected host exists in SSHDeck's inventory, the tunnel command uses that host's resolved OpenSSH profile, including port, identity file, certificate file, jump host, and host-key options. The app config has reserved `tunnel_presets` fields for named presets. Preset editing/loading and persistence are roadmap items.

## Safety notes

Generated commands should be reviewed before running. SSHDeck keeps using system OpenSSH instead of implementing a custom SSH protocol.
