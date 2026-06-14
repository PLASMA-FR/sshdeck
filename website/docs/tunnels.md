# Tunnels

SSHDeck includes a tunnel command generator for local, remote, and dynamic forwards.

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

Command generation and UI scaffolding are present. The app config has reserved `tunnel_presets` fields for named presets. Preset editing/loading, live tunnel process start, stop, monitoring, and persistence are roadmap items.

## Safety notes

Generated commands should be reviewed before running. SSHDeck should keep using OpenSSH instead of implementing a custom SSH protocol for v1.
