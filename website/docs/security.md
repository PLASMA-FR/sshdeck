# Security model

SSHDeck is local-first and OpenSSH-compatible. It does not run a cloud service or require an account.

## What SSHDeck does not do

- It does not upload your SSH config.
- It does not store private keys.
- It does not print private key contents.
- It does not require telemetry.
- It does not replace OpenSSH for v1 connection launching.

## Config safety

- Imported SSH config is read, not blindly rewritten.
- SSHDeck-managed hosts are written to a separate managed config file.
- Include-line changes create timestamped backups.
- Managed config fields are validated to prevent control-character injection.

## Command safety

- SSH command generation uses argument vectors rather than shell string concatenation where possible.
- Destination separators are used to reduce leading-dash option injection risk.
- Dangerous command patterns are flagged before remote execution paths are completed.

## File safety

- Sensitive preview helpers block risky paths before invoking remote preview commands.
- File contents are not logged.
- Remote edit, delete, and overwrite flows are not implemented yet and must require confirmation and backups when added.

## Audit docs

See the repository files:

- `docs/QA.md`
- `docs/SECURITY_REVIEW.md`
