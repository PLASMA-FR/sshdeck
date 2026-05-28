# Troubleshooting

## Run doctor first

```bash
sshdeck doctor
```

Doctor checks OpenSSH binaries, SSH config parse status, managed config path, `.ssh` permissions, referenced identity files, terminal mode, and app config validity.

## No hosts found

Create a host inside SSHDeck with `a`, or add hosts to `~/.ssh/config` and restart/import.

## Identity file warning

The host references a key path that does not exist locally. Update the host, create the key, or remove the IdentityFile entry.

## Mouse does not work

Check your terminal emulator, tmux mouse settings, and nested SSH sessions. SSHDeck needs terminal mouse reporting.

## Unicode icons look wrong

Disable Nerd Font mode or Unicode mode in settings.

## Files cannot open remote directory

The remote listing prototype depends on `ssh` and remote shell permissions. Check the host alias, credentials, remote path, and server permissions.

## Website build fails

Run from the repository root:

```bash
npm install
npm run docs:build
```
