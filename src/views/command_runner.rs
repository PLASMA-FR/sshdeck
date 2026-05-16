use ratatui::{prelude::*, widgets::*}; use crate::{app::App, widgets::status_bar};
pub fn draw(f:&mut Frame, app:&App, area:Rect){ let chunks=Layout::vertical([Constraint::Length(8),Constraint::Min(5),Constraint::Length(1)]).split(area); let cmds="Safe commands:
  uptime
  df -h
  free -h
  docker ps
  systemctl --failed
  journalctl -xe

Custom commands are checked for destructive patterns."; f.render_widget(Paragraph::new(cmds).block(Block::bordered().border_style(app.theme.border()).title(" Remote Command Runner ")), chunks[0]); f.render_widget(Paragraph::new(app.command_output.clone()).wrap(Wrap{trim:false}).block(Block::bordered().title(" Output ")), chunks[1]); status_bar::draw(f, app, chunks[2], "r run · : custom command · Esc back"); }
