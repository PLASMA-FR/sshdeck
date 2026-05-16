use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;
use crate::{app::App, mouse::ClickTarget, widgets::button::{self, ButtonKind}};

pub fn draw(f:&mut Frame, app:&mut App, area:Rect, context:&str){
    let mode=format!("{:?}", app.mode).to_uppercase();
    let prefix=match app.view {
        crate::app::View::Files => format!(" {} │ {}:{} │ {} selected · {} transfer │ mouse:{} │ ", mode, app.current_host().map(|h|h.alias.as_str()).unwrap_or("no-host"), app.remote_path, app.selected_files, app.transfer_queue.active_count(), if app.mouse_enabled{"on"}else{"off"}),
        crate::app::View::CommandRunner => format!(" {} │ {} on {} {} │ ", mode, app.command_input, app.current_host().map(|h|h.alias.as_str()).unwrap_or("no-host"), if app.ascii{app.animator.ascii_spinner()}else{app.animator.spinner()}),
        _ => format!(" {} │ {} hosts │ mouse:{} │ ", mode, app.hosts.len(), if app.mouse_enabled{"on"}else{"off"}),
    };
    let shortcuts: Vec<&str> = context.split('·').map(str::trim).filter(|s| !s.is_empty()).collect();
    let mut spans=vec![Span::styled(prefix, Style::default().bg(app.theme.surface).fg(app.theme.fg))];
    let mut x=area.x + spans[0].content.width() as u16;
    for s in shortcuts {
        let label=s.split_whitespace().next().unwrap_or(s);
        let target=ClickTarget::StatusShortcut(label.to_string());
        let width=(s.chars().count() as u16).saturating_add(4);
        if x + width < area.x + area.width { app.mouse.register(Rect{x,y:area.y,width,height:1}, target.clone()); }
        spans.push(button::label(app,s,&target,ButtonKind::Ghost));
        spans.push(Span::styled(" ", Style::default().bg(app.theme.surface)));
        x=x.saturating_add(width+1);
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(app.theme.surface).fg(app.theme.fg)), area);
}
