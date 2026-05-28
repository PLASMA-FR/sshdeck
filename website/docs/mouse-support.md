# Mouse support

SSHDeck is keyboard-first, but it is also designed for users coming from GUI SSH apps.

## Implemented interaction model

- Click host rows
- Double-click host rows to connect
- Right-click host rows for context menus
- Click modal buttons and form fields
- Click status bar shortcuts where registered
- Scroll host, file, and preview panes

Mouse handling uses crossterm mouse capture and a central hit-test registry in `src/mouse.rs`.

## Partial areas

Some file and transfer context-menu actions are registered before their backing execution workflows are complete. The UI should not claim that upload, download, remote edit, or delete execution is finished until those paths are implemented.

## Terminal support

Mouse support depends on terminal emulator behavior. If clicks do not register, check terminal mouse reporting, multiplexers, and SSH nesting.
