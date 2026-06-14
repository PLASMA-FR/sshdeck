# SSHDeck Files

SSHDeck Files is the Yazi-inspired remote file workflow inside SSHDeck.

## Design goal

A fast terminal-native browser for remote SSH workflows:

- Three-column remote navigation
- Preview/details panel
- Vim-style movement
- Optional local/remote dual-pane mode
- Bookmarks
- Transfer queue
- Safe remote editing with backups
- Command mode

## Current MVP status

Implemented today:

- Remote listing over `ssh` and `ls -la`
- File entry parsing
- Directory navigation shell
- Three-column layout
- Metadata preview
- Hidden-file toggle
- Breadcrumbs
- Selection model foundations
- Real local-pane filesystem entries in dual-pane mode
- Upload and download through system `scp`
- Transfer queue modal and state model
- Safe remote editing through `$EDITOR`, backup, and upload
- Remote mkdir, touch, rename, delete, chmod, and chown flows with confirmations
- Bookmarks commands
- Sensitive path helper checks
- Config fields for bookmarks and last local/remote paths

Not implemented yet:

- Native SFTP backend
- Robust machine-readable remote stat format
- Byte-accurate transfer progress, retries, and overwrite prompts
- Full bookmarks picker UI and automatic last-path restore

## Why it is still valuable

The architecture and visual model are in place, and the daily file workflows are usable through OpenSSH tools. Future work should focus on replacing `ls -la` with safer structured remote listing and adding a native SFTP backend for progress, retries, and unusual path handling.
