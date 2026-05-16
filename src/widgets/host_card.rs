use ratatui::{prelude::*, widgets::*}; use crate::{app::App, ssh::host::SshHost}; pub fn detail<'a>(app:&App, h:Option<&SshHost>)->Paragraph<'a>{ let text=if let Some(h)=h{ format!("Host: {}
User: {}
Port: {}
Target: {}
Tags: {}
Group: {}
Favorite: {}
Status: {}

SSH: ssh {}", h.alias, h.user.clone().unwrap_or_else(||"default".into()), h.port_text(), h.hostname.clone().unwrap_or_else(||"from config".into()), h.tags.join(", "), h.group.clone().unwrap_or_else(||"—".into()), if h.favorite{"yes"}else{"no"}, app.health.uptime, h.alias) } else { "No SSH hosts found.

Options:
  a  Add your first host
  i  Import from ~/.ssh/config
  ?  Help".into() }; Paragraph::new(text).style(app.theme.normal()).block(Block::bordered().border_style(app.theme.inactive_border()).title(" Details ")) }