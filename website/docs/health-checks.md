# Health checks

The health panel is intended to fetch server status over SSH and show compact meters.

## Intended checks

- uptime
- disk usage
- memory usage
- kernel and OS
- failed systemd services
- Docker container count when Docker exists

## Current status

The panel shell and data structures exist, but remote execution and parsing are still roadmap items.

## Design requirement

Health checks must use safe, read-only commands and must surface permission or command failures inside the TUI.
