# Human-centered terminal design notes for SSHDeck

SSHDeck should feel like a calm operator sitting next to you, not a noisy control panel. The user is probably in the terminal because something needs attention: a deploy, a tunnel, a server that feels off, a file they need to grab quickly. The design should reduce anxiety and make the next safe action obvious.

## Sources checked

- Nielsen Norman Group, "10 Usability Heuristics for User Interface Design"
  - Visibility of system status: keep people informed with timely feedback.
  - Match the system to the real world: use the user's language and familiar concepts.
  - User control and freedom: make escape, back, cancel, and undo-like paths obvious.
  - Recognition rather than recall: visible shortcuts, labels, and help reduce memory load.
  - Aesthetic and minimalist design: keep what matters, remove filler.

- Nielsen Norman Group, "Error-Message Guidelines"
  - Put errors near the thing that caused them.
  - Use plain language.
  - Offer a next step, not just a problem statement.
  - Do not blame the user.
  - Do not use modals for minor issues.

- Command Line Interface Guidelines, clig.dev
  - Human-first design: modern CLIs are text UIs for people, not just script glue.
  - Say just enough: silence feels broken, floods feel hostile.
  - Ease of discovery: examples, suggestions, and visible commands help users learn.
  - Conversation as the norm: the tool and user go back and forth, especially after errors.
  - Robustness should be felt: no scary stack traces, no mystery hangs.

- Yazi quick-start/docs
  - Navigation should map to muscle memory: arrows and hjkl both work.
  - File managers feel fast when selection, preview, parent, and next action are always visible.
  - The help path is immediate: q to quit, F1 or ~ for help, clear action tables.

## Design principles for SSHDeck

1. Make state visible.
   Show the selected host, current mode, active path, transfer count, and whether mouse support is on. If the app is waiting on SSH, say what it is doing.

2. Talk like an admin would.
   Prefer "Ready to connect" over "Status: Unknown". Prefer "SSH config was not found yet" over "Invalid config syntax" when the file is simply absent.

3. Keep the next safe action in view.
   Empty states should show useful choices. Error states should say what happened and how to recover. Destructive actions should feel deliberately slower.

4. Let people learn by using it.
   The footer should show only current actions. The help screen should teach mental models, not dump every key.

5. Give the interface a pulse, not fireworks.
   Spinners and toasts should reassure. They should not compete with the file list or host details.

6. Make code read like the UI.
   Use named sections, small helpers, and constants for labels. Avoid long compressed lines that hide intent. The render tree should be easy to scan.

## Applied pass

- Rewrote the dashboard into named sections: empty state, navigation, host list, detail panel.
- Changed robotic placeholders into human copy:
  - "No SSH hosts yet" became "No servers in the deck yet".
  - "Status cache" became "Nothing checked yet" with a visible health action.
  - "Action: ..." paths are handled as real shortcuts where possible.
- Made help feel like a friendly field guide instead of a key dump.
- Softened the file manager preview copy around sensitive files.
- Kept all mouse hit regions tied to visible rows and buttons.

## Future passes

- Add inline field-level validation near each host form field.
- Add first-run onboarding that explains managed config vs user `~/.ssh/config` in one screen.
- Add clearer command-runner safe/unsafe states before executing remote commands.
- Add per-view microcopy tests for important empty/error states.
