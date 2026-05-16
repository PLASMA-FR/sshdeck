use ratatui::{prelude::*, widgets::*}; use crate::app::App; fn centered(area:Rect, w:u16, h:u16)->Rect{ let vertical=Layout::vertical([Constraint::Percentage((100-h)/2),Constraint::Length(h),Constraint::Min(0)]).split(area); Layout::horizontal([Constraint::Percentage((100-w)/2),Constraint::Percentage(w),Constraint::Min(0)]).split(vertical[1])[1] } pub fn command_palette(f:&mut Frame, app:&App, area:Rect){ let r=centered(area,60,14); f.render_widget(Clear,r); let actions="Connect to host
Open SSHDeck Files
Add host
Edit host
Delete host
Copy SSH command
Open tunnel builder
Run remote command
Refresh hosts
Open settings
Toggle theme
Show help
Quit"; f.render_widget(Paragraph::new(format!(" {}

{}",app.palette_input,actions)).block(Block::bordered().border_style(app.theme.border()).title(" Command Palette Ctrl+p ")),r); } pub fn search(f:&mut Frame, app:&App, area:Rect){ let r=centered(area,50,5); f.render_widget(Clear,r); f.render_widget(Paragraph::new(format!("/{}
{} matches",app.search, app.filtered.len())).block(Block::bordered().border_style(app.theme.border()).title(" Fuzzy Search ")),r); } pub fn command_mode(f:&mut Frame, app:&App, area:Rect){ let r=Rect{x:area.x,y:area.y+area.height.saturating_sub(4),width:area.width,height:3}; f.render_widget(Clear,r); f.render_widget(Paragraph::new(format!(":{}",app.command_input)).block(Block::bordered().border_style(app.theme.border()).title(" Command Mode ")),r); }