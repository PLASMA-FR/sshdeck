# Safe remote editing

Safe remote editing is a roadmap workflow. It is documented here so the implementation has a clear contract.

## Intended flow

1. Download selected remote file to a temporary local path.
2. Open `$EDITOR`.
3. Compare hashes before and after editing.
4. If changed, ask whether to upload changes back.
5. Create a timestamped remote backup.
6. Upload the modified file.
7. Refresh the listing and preview.

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

The safety helper layer exists, but the full edit lifecycle is not implemented in the TUI yet.
