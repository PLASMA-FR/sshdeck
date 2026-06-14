# Keyboard shortcuts

## Navigation

```text
↑ / k       move up
↓ / j       move down
Enter       connect or open
Esc         back or close modal
q           quit or back
g           first host
G           last host
```

## Dashboard actions

```text
/           search
a           add host
e           edit host
D           duplicate host
d           delete host
s           open SSHDeck Files prototype
t           tunnel command generator
r           command runner prototype
h           health panel placeholder
l           logs
,           settings
Ctrl+p      command palette
?           help
```

## Files keys implemented today

SSHDeck Files is moving toward a Yazi-style model. The keys currently handled by the TUI are:

```text
j / k       move through remote entries
h           parent directory
l / Enter   open directory or select file for preview metadata
Backspace   parent directory
~           home directory
R           refresh current remote path
.           toggle hidden files
Tab         dual-pane view
Shift+Tab   switch dual-pane focus
T           transfers
:           command mode
```

Reserved but not wired yet: Space visual selection, upload/download shortcuts, bookmarks, remote edit, rename, delete, new file/folder, chmod, and chown.
