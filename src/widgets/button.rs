use ratatui::{prelude::*, widgets::*};
use crate::{app::App, mouse::ClickTarget};

#[derive(Debug, Clone, Copy)]
pub enum ButtonKind { Primary, Secondary, Danger, Ghost }

pub fn label(app: &App, text: &str, target: &ClickTarget, kind: ButtonKind) -> Span<'static> {
    let hovered = app.is_hovered(target);
    let style = style(app, kind, hovered);
    let pad = if hovered { format!("▸ {} ◂", text) } else { format!("  {}  ", text) };
    Span::styled(format!(" {} ", pad), style)
}

pub fn render(f: &mut Frame, app: &mut App, area: Rect, text: &str, target: ClickTarget, kind: ButtonKind) {
    app.mouse.register(area, target.clone());
    let hovered = app.is_hovered(&target);
    let label = if hovered { format!("▸ {} ◂", text) } else { format!("  {}  ", text) };
    let block = Block::bordered()
        .border_type(crate::design::borders::rounded(!app.ascii))
        .border_style(if hovered { app.theme.active_border() } else { app.theme.border() })
        .style(style(app, kind, hovered));
    f.render_widget(Paragraph::new(label).alignment(Alignment::Center).style(style(app, kind, hovered)).block(block), area);
}

pub fn row_style(app: &App, target: &ClickTarget, selected: bool) -> Style {
    if app.is_hovered(target) { app.theme.hovered() } else if selected { app.theme.selected() } else { app.theme.normal() }
}

pub fn style(app: &App, kind: ButtonKind, hovered: bool) -> Style {
    match (kind, hovered) {
        (ButtonKind::Primary, true) => app.theme.button_primary_hover(),
        (ButtonKind::Primary, false) => app.theme.button_primary(),
        (ButtonKind::Secondary, true) => app.theme.button_secondary_hover(),
        (ButtonKind::Secondary, false) => app.theme.button_secondary(),
        (ButtonKind::Danger, true) => app.theme.button_danger_hover(),
        (ButtonKind::Danger, false) => app.theme.button_danger(),
        (ButtonKind::Ghost, true) => app.theme.hovered(),
        (ButtonKind::Ghost, false) => app.theme.accent(),
    }
}
