use ratatui::{prelude::*, widgets::*}; use crate::{app::App, widgets::status_bar}; pub fn draw(f:&mut Frame, app:&App, area:Rect){ let chunks=Layout::vertical([Constraint::Min(5),Constraint::Length(1)]).split(area); let text="SSHDeck Help

Navigation:
  ↑/k       up
  ↓/j       down
  Enter     connect/open
  Esc       back/close modal

Actions:
  /         search
  a         add host
  e         edit host
  d         delete host
  s         open files/SFTP
  t         tunnel builder
  r         run command
  h         health check
  Ctrl+p    command palette
  l         logs
  q         quit

SSHDeck Files Help

Navigation
  j/k move · h/l parent/open · g/G top/bottom · / search · . hidden files
Selection
  Space select · v visual mode · V select all · Ctrl+r clear selection
Operations
  y copy/yank · x cut · p paste · u upload · d download · D delete · r rename · n new · e edit · c copy path
Views
  Tab dual-pane · T transfers · b bookmarks · : command mode"; f.render_widget(Paragraph::new(text).wrap(Wrap{trim:false}).block(Block::bordered().border_style(app.theme.border()).title(" Help ? ")), chunks[0]); status_bar::draw(f, app, chunks[1], "Esc back · q quit"); }