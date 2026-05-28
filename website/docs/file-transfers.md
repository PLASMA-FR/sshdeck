# File transfers

SSHDeck has a transfer queue data model and UI placeholder. Actual upload/download execution from the Files UI is not complete yet.

## Intended behavior

- `u` uploads local selected files to the active remote directory
- `d` downloads remote selected files to the active local directory
- `T` opens the transfer queue
- Failed transfers can be retried
- Active transfers show a subtle Unicode progress animation

## Safety requirements before implementation

- Quote paths safely
- Handle spaces and Unicode filenames
- Confirm before overwriting remote files
- Avoid blocking the TUI thread
- Surface permission errors inside the TUI
- Never log file contents

## Recommended implementation path

Start with `scp -r` or `sftp` batch mode behind a background task, then consider a native SFTP crate when reliability and path semantics are proven.
