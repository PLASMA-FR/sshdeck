use ratatui::{prelude::*, widgets::*};

use crate::{app::App, widgets::status_bar};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).split(area);
    let text = "SSHDeck Help

The shape of the app
  Dashboard     Pick a server, see enough context, act safely.
  Files         Move like Yazi: parent, current folder, preview.
  Tunnels       Build the SSH command before starting it.
  Commands      Run a command with timeout, cap, and guardrails.

Keyboard
  ↑/k       move up
  ↓/j       move down
  Enter     connect or open
  Esc       back out, close a modal, cancel search
  /         search
  s         open SSHDeck Files
  t         tunnel builder
  r         run a remote command
  h         check health
  Ctrl+p    command palette
  ,         settings
  q         quit or go back

Mouse
  Click rows, buttons, sidebar items, breadcrumbs, and footer chips.
  Double-click a host to connect.
  Right-click hosts for a small action menu.
  Scroll over the pane you want to move.

SSHDeck Files
  j/k move · h/l parent/open · R refresh · ~ home
  Tab dual pane · . hidden files · T transfer queue · : guarded command

What is real today
  Health queues uptime, disk, memory, and kernel checks.
  Command mode parses locally and confirms risky patterns before remote execution.
  Files can upload, download, edit with backup, and confirm remote mutations.
  Access profiles show auth, jump, agent, host-key, and forwarding posture.

If something feels scary, SSHDeck should slow down and explain why.";

    f.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: false }).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.border())
                .title(" Help "),
        ),
        chunks[0],
    );

    status_bar::draw(f, app, chunks[1], "Esc back · / search · Ctrl+p commands · ? help");
}
