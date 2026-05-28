# FAQ

## Is SSHDeck a cloud service?

No. SSHDeck is local-first and runs in your terminal.

## Does SSHDeck replace OpenSSH?

No. v1 uses your system OpenSSH tools for connection launching and related workflows.

## Does SSHDeck need an account?

No.

## Does SSHDeck support SFTP today?

The Yazi-style Files interface and remote listing prototype exist. Full upload/download execution is still roadmap work.

## Is `cargo install sshdeck` available?

Not yet. Use `cargo install --path .` from a cloned repository until the project is published.

## Can SSHDeck edit my SSH config?

SSHDeck can write its own managed config and can offer to add an Include line after creating a backup. It should not blindly rewrite complex user SSH configs.

## Does it work without Nerd Fonts?

Yes. ASCII and Unicode fallback modes are part of the UI settings.
