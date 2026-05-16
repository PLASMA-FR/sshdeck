use ratatui::{prelude::*, widgets::*}; use crate::{app::App, widgets::status_bar};
pub fn draw(f:&mut Frame, app:&App, area:Rect){ let chunks=Layout::vertical([Constraint::Min(5),Constraint::Length(1)]).split(area); let cmd=app.tunnel.command(); let text=format!("Type: Local Forward
Local: 127.0.0.1:{}
Target: {}:{}
Host: {}

Command: {}

{} Tunnel flow: localhost:{} {} {}:80

Enter start · Esc cancel", app.tunnel.local_port, app.tunnel.target_host.clone().unwrap_or_default(), app.tunnel.target_port.unwrap_or(80), app.tunnel.host_alias, cmd, app.animator.spinner(), app.tunnel.local_port, app.animator.flow(), app.tunnel.host_alias); f.render_widget(Paragraph::new(text).wrap(Wrap{trim:false}).block(Block::bordered().border_style(app.theme.border()).title(" Tunnel Builder ")), chunks[0]); status_bar::draw(f, app, chunks[1], "t tunnel builder · Enter start · Esc back"); }
