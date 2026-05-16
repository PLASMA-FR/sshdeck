use ratatui::{prelude::*, widgets::*};

use crate::{app::App, mouse::ClickTarget};

/// Single-line text input widget with mouse-focus affordance.
pub fn draw(f: &mut Frame, app: &mut App, area: Rect, label: &str, value: &str, target: ClickTarget, focused: bool) {
    app.mouse.register(area, target.clone());
    let style = if app.is_hovered(&target) || focused { app.theme.selected() } else { app.theme.normal() };
    let text = format!("{label:<14} {value}");
    f.render_widget(
        Paragraph::new(text).style(style).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(if focused { app.theme.active_border() } else { app.theme.inactive_border() }),
        ),
        area,
    );
}
