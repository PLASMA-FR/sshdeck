# Security model

SSHDeck is local-first and OpenSSH-compatible. It does not run a cloud service or require an account.

## Design stance

SSHDeck treats host inventory, credential material, and access policy as separate things:

- host names, groups, tags, notes, favorites, and recent state live in SSHDeck
- private keys, certificates, agents, host keys, and SSH policy stay in OpenSSH and the OS
- per-host access posture is shown in the TUI before you connect

This keeps SSHDeck useful as an SSH manager without turning it into a private-key vault.

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
- `CertificateFile`, `StrictHostKeyChecking`, and `UserKnownHostsFile` are parsed and preserved when SSHDeck builds direct OpenSSH commands for known hosts.

## Access profile

The dashboard and host detail screen show an access profile for the selected host:

- auth source: OpenSSH default/agent, key file, certificate, or hardware-backed `*-sk` key naming
- path: direct or through `ProxyJump`
- agent forwarding: off, on, or custom `ForwardAgent` value
- host-key posture: `StrictHostKeyChecking` and known_hosts file
- saved forwards: local and remote forward counts

SSHDeck warns when agent forwarding is enabled or strict host-key checking is explicitly disabled. It does not override the user's OpenSSH policy by surprise.

## Credential guidance

Prefer public-key authentication over passwords. For privileged hosts, prefer passphrase-protected keys, SSH certificates, or hardware-backed FIDO/security-key keys where your OpenSSH build supports them. Keep agent forwarding opt-in and avoid enabling it globally.

## Command safety

- SSH command generation uses argument vectors rather than shell string concatenation where possible.
- Destination separators are used to reduce leading-dash option injection risk.
- Remote command execution has timeout and output-size limits.
- Dangerous command patterns require a typed confirmation before execution.

## File safety

- Sensitive preview helpers block risky paths before invoking remote preview commands.
- File contents are not logged.
- Uploads and downloads run through system `scp` in a background task.
- Remote edit downloads to a temporary local file, opens `$EDITOR`, creates a remote backup, uploads the edited file, and removes the temporary local file.
- Remote delete, rename, chmod, and chown actions require typed confirmations.

## Doctor checks

`sshdeck doctor` checks local OpenSSH tools, key-management helpers, hardware-key support advertised by `ssh -Q key`, agent socket state, `.ssh` permissions, known_hosts state, config parsing, referenced identity files, referenced certificates, and per-host known_hosts files.

## Audit docs

See the repository files:

- `docs/QA.md`
- `docs/SECURITY_REVIEW.md`
