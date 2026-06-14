# SSHDeck Manual QA Checklist

Status: reviewed during the end-to-end audit pass. Items marked complete were verified by code inspection and/or automated tests in this environment. Items marked partial require a real interactive terminal or remote SSH host for full validation.

## Build checks

- [x] `cargo test --quiet` passes after audit fixes.
- [x] `cargo check` scheduled in final release check.
- [ ] `cargo fmt` could not be run in this environment because the `rustfmt` cargo subcommand is not installed.
- [ ] `cargo clippy` could not be run if the `clippy` cargo subcommand is unavailable; final output records the actual result.
- [x] Project has a clean Rust module layout under `src/`.
- [x] README was reviewed for truthfulness against implementation.

Release gate:

- [ ] `cargo fmt --check` passes, or the release notes explicitly document why rustfmt was unavailable.
- [ ] `cargo check --locked` passes.
- [ ] `cargo test --locked` passes.
- [ ] `cargo clippy --locked -- -D warnings` passes, or the release notes explicitly document why clippy was unavailable.
- [ ] `cargo run --locked -- doctor` passes on the release machine.
- [ ] `npm run docs:build` passes for the website.

## Dashboard checks

- [x] Empty-host state exists and offers add/import/help actions.
- [x] Host rows render with details and quick action buttons.
- [x] Sidebar mouse regions are registered.
- [x] Search mode filters hosts with fuzzy matching.
- [x] Status bar shows view-specific shortcuts.
- [x] Terminal-too-small warning exists in UI rendering.

Manual follow-up:

- [ ] Verify dashboard visual polish in a real 100x30+ terminal.
- [ ] Verify double-click connect against a real SSH host.

## Host management checks

- [x] Add/edit/duplicate form exists.
- [x] Alias and hostname validation exists.
- [x] Numeric port validation exists.
- [x] Duplicate alias warning exists.
- [x] Control-character and newline injection into managed ssh_config is blocked.
- [x] Aliases beginning with `-` are blocked.
- [x] Managed host config is backed up before rewriting.
- [x] Optional Include line is backed up before modifying `~/.ssh/config`.

Manual follow-up:

- [ ] Add a host interactively with mouse and keyboard.
- [ ] Edit an existing managed host interactively.
- [ ] Confirm imported-host deletion only affects current view and metadata.

## Mouse checks

- [x] Mouse hit-test registry exists and tests cover several rendered regions.
- [x] Dashboard host rows, quick buttons, sidebar items, forms, settings rows, files rows, breadcrumbs, status shortcuts, tabs, and modal buttons register targets.
- [x] Right-click host/file context menu paths exist.
- [x] Scroll state is tracked for host/file/preview targets.

Known partial items:

- [ ] Context menus are mostly mouse-driven and need keyboard navigation.
- [ ] Files list scroll state is not yet fully reflected in rendering.
- [ ] File double-click currently selects/toasts rather than fully opening directories.
- [ ] Dual-pane local rows are UI placeholders, not real local filesystem entries.
- [ ] Toggling mouse off in settings updates app state but terminal mouse capture is only configured at startup/cleanup.

## Keyboard checks

- [x] Dashboard: `j/k`, arrows, Enter, `s`, `t`, `r`, `h`, `a`, `e`, `d`, `/`, Ctrl+p, `?`, `q`, Esc are handled.
- [x] Host form: Tab, Shift+Tab, Ctrl+s, Enter, Esc, typing, and Backspace are handled.
- [x] Files: `j/k`, `h/l`, Enter, Backspace, `~`, `R`, `.`, Tab, Shift+Tab, `T`, `:`, and `q`/Esc are handled.
- [x] Settings: arrows, `j/k`, Enter/Space are handled.
- [x] Public keyboard docs distinguish implemented Files keys from reserved shortcuts.

Known partial items:

- [ ] File shortcuts for upload/download/delete/rename/new/edit/bookmarks/visual selection are not implemented and README now marks them as roadmap.
- [ ] Command palette lacks keyboard result selection/filter execution; README treats it as an MVP command palette, not a complete launcher.
- [ ] Transfer modal has no dedicated transfer-row keyboard behavior yet.

## Files checks

- [x] Remote directory listing uses `ssh` with argument separation and shell-quoted paths.
- [x] Parent/home navigation helpers are tested.
- [x] Symlink `ls -la` parsing strips ` -> target` from entry name/path.
- [x] Sensitive preview helper blocks `.env`, private keys, and dangerous system files before invoking ssh.
- [x] Large preview helper path has a max-byte guard.
- [x] README marks real SFTP/scp transfer execution, editing, delete/rename/new-file, bookmarks, and full dual-pane local model as not implemented.
- [x] Docs mark upload/download shortcuts and bookmarks UI as reserved until execution workflows are wired.

Manual follow-up:

- [ ] Test listing a real remote home directory.
- [ ] Test listing paths with spaces and Unicode names on a real host.
- [ ] Test permission denied and unreachable-host messages in a real terminal.

## Tunnels checks

- [x] Local forward command generation is tested.
- [x] Dynamic forward command generation is tested with quoted alias.
- [x] README marks live start/stop as not implemented.
- [x] Tunnel preset config fields are documented as reserved until TUI preset editing/loading is wired.

Manual follow-up:

- [ ] Add editable tunnel fields before claiming full tunnel builder support.
- [ ] Start/stop process management remains roadmap.

## Command runner checks

- [x] Dangerous command detection has tests.
- [x] Additional recursive permission form (`chmod -R777`) is detected.
- [x] README marks remote command execution as prototype/not wired.

Manual follow-up:

- [ ] Implement explicit confirmation flow if custom remote execution is wired later.
- [ ] Add timeout/output-size limits when execution is enabled.

## Health checks

- [x] Doctor/local health checks exist.
- [x] README marks remote server health execution as placeholder.

Manual follow-up:

- [ ] Wire safe remote health commands with timeouts and output parsing.
- [ ] Handle missing `docker`, missing `systemctl`, and permission errors.

## Error state checks

- [x] Missing SSH config is handled with empty/default host list behavior.
- [x] Missing/invalid app config returns errors through `anyhow` instead of unchecked panics.
- [x] Remote listing returns readable errors on non-zero ssh exit.
- [x] Sensitive preview returns a readable blocked error.
- [x] Terminal cleanup guard exists for normal errors/panic unwinding.

Manual follow-up:

- [ ] Verify no ugly panic in a real terminal for forced remote failures.
- [ ] Verify no ssh/scp/sftp binary state through `sshdeck doctor` on a PATH-controlled test shell.

## Security checks

- [x] OpenSSH destination arguments use `--` in SSHDeck-built command paths.
- [x] Managed ssh_config newline/control-character injection is blocked and sanitized in rendering.
- [x] Logs redact identity-file arguments and common sensitive path markers.
- [x] Sensitive preview helper blocks before invoking ssh.
- [x] README no longer claims unimplemented remote delete/edit/overwrite guarantees.

## Config compatibility checks

- [x] Legacy `config.toml` without hidden-host, recent-host, tunnel-preset, or last-path fields deserializes with defaults.
- [x] Reserved state fields deserialize from TOML.
- [ ] Confirm a saved config round trip keeps top-level state fields, bookmarks, host metadata, and settings.
- [ ] Before wiring hidden imported hosts, verify deleting an imported host persists and restores across restart without editing `~/.ssh/config`.
- [ ] Before wiring recent hosts, cap the list and define duplicate/missing-host cleanup behavior.

## Release checks

- [x] README is honest about MVP status and partial features.
- [x] `docs/QA.md` exists.
- [x] `docs/SECURITY_REVIEW.md` exists.
- [ ] Website docs and README list the same CLI commands/options.
- [ ] Keyboard docs match `src/app.rs` handlers and do not advertise reserved shortcuts as implemented.
- [ ] Feature pages separate implemented behavior from roadmap/reserved config fields.
- [ ] Final `cargo check`, `cargo test`, and `cargo run -- doctor` must pass before release tagging.
- [ ] Install missing Rust components and rerun `cargo fmt`/`cargo clippy` before publishing.
