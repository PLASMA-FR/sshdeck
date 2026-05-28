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
- Transfer queue modal and state model
- Sensitive path helper checks
- Dual-pane visual placeholder

Not implemented yet:

- Real upload/download execution from the TUI
- Safe remote edit lifecycle
- Remote delete, rename, touch, mkdir, chmod, or chown execution
- Native SFTP backend
- Robust machine-readable remote stat format
- Full bookmarks UI

## Why it is still valuable

The architecture and visual model are in place. Future work should focus on replacing `ls -la` with safer structured remote listing and completing transfer execution using scp or sftp batch mode.
