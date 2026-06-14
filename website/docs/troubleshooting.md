# Troubleshooting

## Run doctor first

```bash
sshdeck doctor
```

Doctor checks OpenSSH binaries, `ssh-keygen`, `ssh-add`, `ssh-agent`, advertised security-key auth support, agent socket state, SSH config parse status, managed config path, `.ssh` permissions, known_hosts state, referenced identity files, referenced certificates, per-host known_hosts files, terminal mode, and app config validity.

## No hosts found

Create a host inside SSHDeck with `a`, or add hosts to `~/.ssh/config` and restart/import.

## Identity file warning

The host references a key path that does not exist locally. Update the host, create the key, or remove the IdentityFile entry.

## Host-key warning

If doctor reports a missing known_hosts file, connect only after verifying the server fingerprint. If a host's access profile says strict host-key checking is disabled, review the matching `StrictHostKeyChecking` line in your SSH config.

## Security-key auth says unsupported

`sshdeck doctor` reads `ssh -Q key`. If it does not show FIDO/security-key key types, update OpenSSH or use normal public-key/certificate authentication for that machine.

## Mouse does not work

Check your terminal emulator, tmux mouse settings, and nested SSH sessions. SSHDeck needs terminal mouse reporting.

## Unicode icons look wrong

Disable Nerd Font mode or Unicode mode in settings.

## Files cannot open remote directory

Remote listing depends on `ssh`, the remote shell, and permissions for the selected path. Check the host alias, credentials, remote path, and server permissions.

## Website build fails

Run from the repository root:

```bash
npm install
npm run docs:build
```
