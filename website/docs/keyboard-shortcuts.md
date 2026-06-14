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
s           open SSHDeck Files
t           tunnel builder
r           command runner
h           health check
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
p           preview selected remote file
.           toggle hidden files
Space       toggle selected remote path
Tab         dual-pane view
Shift+Tab   switch dual-pane focus
T           transfers
:           command mode
u           upload selected local entry
d           download selected remote entry
x           delete selected remote file
n           start mkdir command
b           bookmark current remote path
```

Command mode handles `mkdir`, `touch`, `rename`, `chmod`, `chown`, `upload`, `download`, `bookmark add`, and `bookmark jump`.

Reserved but not wired yet: a full bookmarks picker UI and richer multi-select actions.
