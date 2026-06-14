# FAQ

## Is SSHDeck a cloud service?

No. SSHDeck is local-first and runs in your terminal.

## Does SSHDeck replace OpenSSH?

No. SSHDeck uses your system OpenSSH tools for connection launching, tunnels, commands, health checks, file transfers, and remote editing.

## Does SSHDeck need an account?

No.

## Does SSHDeck support file transfers today?

Yes. SSHDeck Files can upload and download through system `scp`. Native SFTP is still roadmap work.

## Is `cargo install sshdeck` available?

Not yet. Use `bash scripts/install.sh` from a cloned repository until the project is published. Direct `cargo install --locked --path .` also works.

## Can SSHDeck edit my SSH config?

SSHDeck can write its own managed config and can offer to add an Include line after creating a backup. It should not blindly rewrite complex user SSH configs.

## Does it work without Nerd Fonts?

Yes. ASCII and Unicode fallback modes are part of the UI settings.
