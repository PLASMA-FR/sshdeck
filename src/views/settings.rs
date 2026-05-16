use ratatui::{prelude::*, widgets::*}; use crate::{app::App, widgets::status_bar}; pub fn draw(f:&mut Frame, app:&App, area:Rect){ let chunks=Layout::vertical([Constraint::Min(5),Constraint::Length(1)]).split(area); f.render_widget(Paragraph::new(format!("Theme: {}
Animations: {}
Unicode: {}
Nerd Font: {}
Config: {}", app.config.ui.theme, app.config.ui.animations, app.config.ui.unicode, app.config.ui.nerd_font, app.config.path.display())).block(Block::bordered().title(" Settings ")), chunks[0]); status_bar::draw(f, app, chunks[1], "Ctrl+p commands · Esc back"); }