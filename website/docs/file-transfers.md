# File transfers

SSHDeck can upload and download the selected file or directory with system `scp` from the Files UI. Transfers run in a background task and update the transfer queue when they finish or fail.

## Current UI and behavior

- `T` opens the transfer queue
- `u` uploads the selected local entry to the active remote directory
- `d` downloads the selected remote entry to the active local directory
- `:upload /local/path` uploads a specific path
- SSHDeck uses the selected host's port, identity file, certificate, jump host, and host-key options when it knows them
- Completed transfers refresh local and remote listings

## Current limitations

- Progress is job-state based today, not byte-accurate.
- Failed transfers are shown in the queue but do not have a retry button yet.
- The backend is `scp`; a native SFTP backend would be more robust for unusual path semantics.

## Safety notes

- Quote paths safely
- Handle spaces and Unicode filenames
- Confirm before overwriting remote files
- Avoid blocking the TUI thread
- Surface permission errors inside the TUI
- Never log file contents

## Recommended implementation path

The next step is a native SFTP backend or structured batch mode for better progress reporting, retries, and overwrite prompts.
