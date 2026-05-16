use ratatui::prelude::*;

use crate::{app::App, widgets::modal};

/// Host add/edit lives as a modal, but keeping this module makes the view
/// boundary explicit for future full-screen host management.
pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    modal::host_form(f, app, area);
}
