# Safe remote editing

Safe remote editing is available from the Files command flow for a selected remote file.

## Intended flow

1. Download selected remote file to a temporary local path.
2. Open `$EDITOR`.
3. Create a timestamped remote backup.
4. Upload the modified file.
5. Remove the temporary local file.
6. Refresh the listing and preview.

## Sensitive files

Editing should warn before opening paths such as:

- `.env`
- Private keys
- `id_rsa`
- `id_ed25519`
- `/etc/sudoers`
- `/etc/passwd`
- `/etc/shadow`
- service files
- nginx or apache configs

## Current status

The TUI asks for a typed confirmation before editing, downloads with `scp`, opens `$EDITOR`, creates a remote backup, uploads with `scp`, and refreshes the view. Hash comparison and a final upload prompt are still roadmap hardening items.
