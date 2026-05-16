use ratatui::{prelude::*, widgets::*};
use crate::{app::App, mouse::ClickTarget};

pub fn draw(f:&mut Frame, app:&mut App, area:Rect, context:&str){
    let mode=format!("{:?}", app.mode).to_uppercase();
    let text=match app.view {
        crate::app::View::Files => format!(" {} │ {}:{} │ {} selected · 18 MB │ {} transfer │ mouse:{} │ {}", mode, app.current_host().map(|h|h.alias.as_str()).unwrap_or("no-host"), app.remote_path, app.selected_files, app.transfer_queue.active_count(), if app.mouse_enabled{"on"}else{"off"}, context),
        crate::app::View::CommandRunner => format!(" {} │ {} on {} {} │ Esc cancel", mode, app.command_input, app.current_host().map(|h|h.alias.as_str()).unwrap_or("no-host"), if app.ascii{app.animator.ascii_spinner()}else{app.animator.spinner()}),
        _ => format!(" {} │ {} hosts │ 3 online │ 1 tunnel │ mouse:{} │ {}", mode, app.hosts.len(), if app.mouse_enabled{"on"}else{"off"}, context),
    };
    app.mouse.register(area, ClickTarget::StatusShortcut("status".into()));
    f.render_widget(Paragraph::new(text).style(Style::default().bg(app.theme.surface).fg(app.theme.fg)), area);
}
