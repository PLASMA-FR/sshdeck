use ratatui::{prelude::*, widgets::*}; use crate::{app::App, ssh::command::ssh_command_for, widgets::{server_meter,status_bar}};
pub fn draw(f:&mut Frame, app:&App, area:Rect){ let chunks=Layout::vertical([Constraint::Length(3),Constraint::Min(5),Constraint::Length(1)]).split(area); f.render_widget(Paragraph::new("SSHDeck Host Detail").style(app.theme.title()).block(Block::bordered().border_style(app.theme.border())), chunks[0]); let cols=Layout::horizontal([Constraint::Percentage(55),Constraint::Percentage(45)]).split(chunks[1]); let text=if let Some(h)=app.current_host(){ format!("Alias: {}
HostName: {}
User: {}
Port: {}
IdentityFile: {}
ProxyJump: {}
Tags: {}
Group: {}
Notes: {}
Recent: {}
Generated SSH: {}

Tunnel configs:
  LocalForward: {:?}
  RemoteForward: {:?}

File shortcuts:
  s open SSHDeck Files
  ~ remote home
  / search files", h.alias, h.hostname.clone().unwrap_or_default(), h.user.clone().unwrap_or_default(), h.port_text(), h.identity_file.as_ref().map(|p|p.display().to_string()).unwrap_or_default(), h.proxy_jump.clone().unwrap_or_default(), h.tags.join(", "), h.group.clone().unwrap_or_default(), h.notes.clone().unwrap_or_default(), h.recent_connection.clone().unwrap_or_default(), ssh_command_for(h), h.local_forwards, h.remote_forwards) } else {"No host selected".into()}; f.render_widget(Paragraph::new(text).wrap(Wrap{trim:false}).block(Block::bordered().border_style(app.theme.border()).title(" Host ")), cols[0]); let r=Layout::vertical([Constraint::Length(5),Constraint::Length(5),Constraint::Length(5),Constraint::Min(3)]).split(cols[1]); f.render_widget(server_meter::gauge(app,"RAM",0.42), r[0]); f.render_widget(server_meter::gauge(app,"Disk",0.37), r[1]); f.render_widget(server_meter::gauge(app,"Services",0.1), r[2]); f.render_widget(Paragraph::new(format!("Uptime: {}
Kernel: {}
Failed services: {}
Docker containers: {}",app.health.uptime,app.health.kernel,app.health.failed_services,app.health.docker_containers)).block(Block::bordered().title(" Health ")), r[3]); status_bar::draw(f, app, chunks[2], "Enter connect · s files · t tunnel · r command · Esc back"); }
