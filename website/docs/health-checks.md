# Health checks

The health panel fetches read-only server status over SSH and shows compact meters in the host detail view.

## Intended checks

- uptime
- disk usage
- memory usage
- kernel and OS
- failed systemd services
- Docker container count when Docker exists

## Current status

Remote execution and parsing are implemented. SSHDeck runs guarded read-only commands with a timeout, tolerates missing `systemctl` or Docker, and summarizes uptime, disk, memory, kernel, failed services, and Docker container count.

## Design requirement

Health checks must use safe, read-only commands and must surface permission or command failures inside the TUI.
