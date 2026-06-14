# SSHDeck Security Review

Date: 2026-05-29
Scope: local Rust source, SSH command construction, managed config writes, remote file helper paths, logs, README claims, and available automated tests.

Last implementation/docs update: 2026-06-14.

## Summary

SSHDeck is an early local-first TUI MVP. The audit focused on preventing local command/option injection, OpenSSH config injection, accidental secret logging, unsafe remote preview behavior, and false safety claims.

High-impact fixes completed in this pass:

- inserted `--` before OpenSSH destination arguments in SSHDeck-built `ssh` commands where practical
- blocked managed-host aliases beginning with `-`
- blocked newlines/control characters in managed-host form fields
- sanitized managed config rendering to avoid multi-line directive injection
- made managed config/include-line writes use temporary-file then rename
- changed backup timestamps to include subsecond precision
- blocked sensitive remote preview helper paths before invoking `ssh`
- checked non-zero remote preview exit status
- redacted identity-file arguments and common sensitive path markers from persistent logs
- fixed symlink `ls -la` parsing so `link -> target` is not treated as a path
- expanded regression tests from 39 to 47 tests
- rewrote README to mark partial/roadmap features honestly

Follow-up implementation completed on 2026-06-14:

- added per-host access profiles for auth source, jump path, agent forwarding, host-key posture, and saved forwards
- parsed and preserved OpenSSH `CertificateFile`, `StrictHostKeyChecking`, and `UserKnownHostsFile`
- preserved host security options in direct `ssh`, `scp`, remote file, command runner, health, and tunnel execution paths
- surfaced warnings for enabled agent forwarding and disabled strict host-key checking
- expanded `sshdeck doctor` checks for `ssh-keygen`, `ssh-add`, `ssh-agent`, security-key support, agent socket state, known_hosts, certificates, and per-host known_hosts files

## Command injection review

Reviewed command-building paths:

- `src/main.rs` quick connect
- `src/app.rs` interactive connect
- `src/ssh/command.rs` SSH args, test args, scp display command helpers, remote command helper
- `src/files/remote_fs.rs` remote listing and preview helpers
- `src/ssh/tunnel.rs` tunnel display command generator

Findings and fixes:

1. OpenSSH option injection via leading-dash host aliases or quick targets.
   - Risk: `ssh <untrusted>` can treat a destination beginning with `-` as an option.
   - Fix: SSHDeck-built command paths now add `--` before destination arguments.
   - Regression: `destination_is_separated_from_options` test added.

2. Remote file helper host aliases.
   - Risk: raw `list_remote(host, path)` and `preview_remote(host, path)` could pass leading-dash hosts as options.
   - Fix: raw helper paths now pass `ssh -- <host> <cmd>`.

3. Remote shell paths.
   - Current listing/preview helpers must use a remote shell command because OpenSSH executes a remote shell command string.
   - Paths are passed through shell quoting helpers.
   - Known limitation: remote listing uses `ls -la`; unusual filenames with newlines remain hard to represent correctly. A native SFTP backend is recommended before claiming production-grade file management.

4. Tunnel commands.
   - `src/ssh/tunnel.rs` builds argument vectors for local, remote, and dynamic forwards.
   - Live tunnel start/stop uses `std::process::Command` with args rather than shell strings.
   - When the target host is in SSHDeck's inventory, tunnel execution uses the resolved host profile instead of a bare alias.

## Secret exposure review

Reviewed likely secret paths and output channels:

- `.env`
- SSH private keys (`id_rsa`, `id_ed25519`)
- `authorized_keys`
- `/etc/shadow`
- identity-file command args
- logs at XDG data path `sshdeck/events.log`

Findings and fixes:

- Persistent logs previously stored every toast verbatim, including full SSH commands and remote paths.
- Added log redaction for identity-file args and common sensitive path markers.
- Sensitive preview helper now blocks before invoking ssh.
- README no longer claims full remote-file secret safety for unimplemented edit/delete/overwrite flows.

Known risks:

- Redaction is pattern-based, not a full secret scanner.
- Command output is not currently persisted by the command runner, but if execution is wired later, output must be size-limited and optionally redacted.
- Remote file preview is wired into the UI; sensitive paths require explicit confirmation and file contents are not persisted to logs.

## Config safety review

Reviewed:

- `~/.config/sshdeck/config.toml`
- `~/.config/sshdeck/ssh_config`
- optional `Include ~/.config/sshdeck/ssh_config` insertion into `~/.ssh/config`
- reserved config state for hidden imported hosts, recent hosts, tunnel presets, last paths, and bookmarks

Findings and fixes:

1. Managed ssh_config directive injection.
   - Risk: newline/control characters in fields could inject extra OpenSSH directives such as `ProxyCommand`.
   - Fix: validation blocks newlines/control chars; rendering also collapses controls as defense in depth.
   - Regression: `rejects_config_injection_control_characters` test added.

2. Leading-dash aliases.
   - Risk: an alias beginning with `-` can behave like an ssh option in command execution.
   - Fix: managed host validation rejects aliases beginning with `-`.
   - Regression: `rejects_aliases_that_look_like_ssh_options` test added.

3. Backup collisions and partial writes.
   - Risk: second-level backup filenames could collide; direct writes can corrupt files on crash.
   - Fix: backup timestamps include fractional seconds; managed/include/config writes use temp-file then rename.

Known risks:

- Existing file permissions are not fully normalized to `0600`; this should be added for SSHDeck config/log files on Unix before a packaged release.
- The parser is intentionally simple and does not fully implement every OpenSSH config grammar edge case.
- The new reserved config fields can store host aliases and filesystem paths. They must not be used for private keys, passwords, command output, or file contents.
- Persistent imported-host hiding is schema-ready but not wired through the TUI restore path yet; until then, imported-host deletion remains current-session behavior.

## File operation safety review

Implemented:

- remote directory listing with quoted path expression
- metadata-only preview in the main Files UI
- sensitive-path helper checks
- dangerous-delete helper checks
- transfer queue state model
- upload/download execution through system `scp`
- safe remote editing with temporary local file, remote backup, upload, and cleanup
- remote mkdir, touch, rename, chmod, chown, and delete actions with confirmations

Not implemented yet:

- native SFTP backend
- byte-accurate transfer progress
- overwrite prompts, cancel/retry process management, and richer multi-select actions
- hash comparison/final upload prompt for safe edit

Risk stance:

- Destructive remote operations require typed confirmation, block high-risk roots through helper checks, and use `Command` args plus carefully quoted remote shell snippets with tests.
- A native SFTP backend is still recommended before claiming production-grade file management for unusual filenames and path semantics.

## Destructive action review

Implemented:

- host deletion requires confirmation
- managed host deletion edits only SSHDeck managed config, not the user's original `~/.ssh/config`
- imported host deletion removes it from the current view/metadata only
- dangerous command patterns require typed confirmation before remote execution
- remote file delete, rename, chmod, and chown actions require typed confirmation
- tunnel start/stop lifecycle is implemented with child-process polling

## Terminal cleanup review

Reviewed:

- `TerminalCleanup` Drop guard in `src/main.rs`
- `connect_selected` raw-mode/alternate-screen handling in `src/app.rs`

Findings:

- Main TUI has a cleanup guard that disables raw mode and leaves alternate screen on error/unwind.
- SSH connection path disables raw mode, leaves alternate screen, runs system ssh, then restores alternate screen/raw mode.
- If the process is killed with SIGKILL, cleanup is impossible; this is normal for terminal apps.

Future hardening:

- Add signal handling for SIGINT/SIGTERM cleanup.
- Add integration tests using a pseudo-terminal for connect/return behavior.

## Dependency review

Current major dependencies are appropriate for a Rust TUI MVP:

- `ratatui` / `crossterm` for terminal UI and events
- `clap` for CLI
- `serde`, `toml`, `serde_json` for config/data
- `anyhow`, `color-eyre` for errors
- `dirs` for config/data paths
- `regex`, `fuzzy-matcher`, `unicode-width`, `chrono`, `shell-words`

No AI API, telemetry, cloud-sync, or Electron dependency was found in `Cargo.toml`.

Potential cleanup:

- `serde_json` should be removed if it remains unused after feature pruning.
- Audit transitive dependencies with `cargo audit` before publishing.

## Known risks and future hardening

- Add Unix `0600` permissions for app config, managed ssh_config, and logs.
- Replace `ls -la` parsing with SFTP or a machine-readable remote command (`stat`/NUL-separated output) before claiming robust file management.
- Add more complete overwrite prompts and retry/cancel controls to transfer execution.
- Add hash comparison or final upload prompt to safe remote editing.
- Add keyboard navigation for context menus.
- Add PTY/manual integration tests for mouse and terminal cleanup.
- Keep README feature claims synchronized with implementation.

## Verification artifacts

Automated tests added/verified:

- OpenSSH destination separator for leading-dash aliases
- recursive permission dangerous-command forms
- managed ssh_config injection validation
- leading-dash alias rejection
- sensitive preview blocking before ssh
- log redaction
- symlink parsing without target suffix

See `docs/QA.md` for manual QA coverage and remaining manual follow-up.
