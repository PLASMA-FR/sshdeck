use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;

use crate::{app::App, mouse::ClickTarget, widgets::button::{self, ButtonKind}};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect, context: &str) {
    let mode = format!("{:?}", app.mode).to_uppercase();
    let prefix = match app.view {
        crate::app::View::Files => format!(
            " {} │ {}:{} │ {} selected · {} transfer │ mouse:{} │ ",
            mode,
            app.current_host().map(|h| h.alias.as_str()).unwrap_or("no-host"),
            app.remote_path,
            app.selected_files,
            app.transfer_queue.active_count(),
            if app.mouse_enabled { "on" } else { "off" }
        ),
        crate::app::View::CommandRunner => format!(
            " {} │ {} on {} {} │ ",
            mode,
            app.command_input,
            app.current_host().map(|h| h.alias.as_str()).unwrap_or("no-host"),
            if app.ascii { app.animator.ascii_spinner() } else { app.animator.spinner() }
        ),
        _ => format!(
            " {} │ {} hosts │ mouse:{} │ ",
            mode,
            app.hosts.len(),
            if app.mouse_enabled { "on" } else { "off" }
        ),
    };

    let shortcuts: Vec<&str> = context
        .split('·')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let mut spans = vec![Span::styled(
        prefix.clone(),
        Style::default().bg(app.theme.surface).fg(app.theme.fg),
    )];
    let mut x = area.x + display_width(&prefix);
    let right_edge = area.x.saturating_add(area.width);

    for s in shortcuts {
        let action = shortcut_action(s);
        let target = ClickTarget::StatusShortcut(action.to_string());
        let chip = shortcut_chip(app, s, &target);
        let width = display_width(&chip);

        if x < right_edge && x.saturating_add(width) <= right_edge {
            app.mouse.register(Rect { x, y: area.y, width, height: 1 }, target.clone());
            spans.push(Span::styled(chip, button::style(app, ButtonKind::Ghost, app.is_hovered(&target))));
            if x.saturating_add(width) < right_edge {
                spans.push(Span::styled(" ", Style::default().bg(app.theme.surface)));
            }
        }
        x = x.saturating_add(width).saturating_add(1);
    }

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(app.theme.surface).fg(app.theme.fg)),
        area,
    );
}

fn shortcut_action(shortcut: &str) -> &str {
    shortcut.split_whitespace().next().unwrap_or(shortcut)
}

fn shortcut_chip(app: &App, text: &str, target: &ClickTarget) -> String {
    if app.is_hovered(target) {
        if app.ascii {
            format!("[ {text} ]")
        } else {
            format!("▸ {text} ◂")
        }
    } else {
        format!("  {text}  ")
    }
}

fn display_width(text: &str) -> u16 {
    UnicodeWidthStr::width(text) as u16
}
