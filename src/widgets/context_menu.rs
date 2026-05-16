use ratatui::{prelude::*, widgets::*};

use crate::{app::App, mouse::ClickTarget, widgets::{button, button::ButtonKind}};

/// Reusable context-menu renderer. Current app-level menus are rendered from
/// widgets::modal for stacking; this helper keeps the component available for
/// future pane-local menus.
pub fn draw_items(f: &mut Frame, app: &mut App, area: Rect, title: &str, items: &[(String, ClickTarget)]) {
    let rows: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (label, target))| {
            let y = area.y + 1 + i as u16;
            app.mouse.register(Rect { x: area.x + 1, y, width: area.width.saturating_sub(2), height: 1 }, target.clone());
            ListItem::new(button::label(app, label, target, ButtonKind::Ghost))
        })
        .collect();
    f.render_widget(
        List::new(rows).style(app.theme.overlay()).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.active_border())
                .title(format!(" {title} ")),
        ),
        area,
    );
}
