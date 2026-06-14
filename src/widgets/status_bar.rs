use ratatui::{prelude::*, widgets::*};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    app::{App, Mode, View},
    mouse::ClickTarget,
    widgets::button::{self, ButtonKind},
};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect, context: &str) {
    let mut shortcuts: Vec<String> = context
        .split('·')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    let help_label = take_shortcut(&mut shortcuts, "?").unwrap_or_else(|| "? help".into());
    let help_target = ClickTarget::StatusShortcut("?".into());
    let help_chip = shortcut_chip(app, &help_label, &help_target);
    let help_width = display_width(&help_chip);

    let mut prefix = status_prefix(app);
    let reserve_for_help = help_width.saturating_add(1).min(area.width);
    prefix = fit_to_width(&prefix, area.width.saturating_sub(reserve_for_help));

    let mut spans = vec![Span::styled(
        prefix.clone(),
        Style::default().bg(app.theme.surface).fg(app.theme.fg),
    )];
    let mut x = area.x + display_width(&prefix);
    let right_edge = area.x.saturating_add(area.width);
    let mut needs_gap = !prefix.is_empty() && !prefix.ends_with(' ');

    for s in shortcuts {
        let action = shortcut_action(&s);
        let target = ClickTarget::StatusShortcut(action.to_string());
        let chip = shortcut_chip(app, &s, &target);
        let width = display_width(&chip);
        let gap = u16::from(needs_gap);
        let reserved = help_width.saturating_add(1);
        let remaining = right_edge.saturating_sub(x);

        if remaining >= gap.saturating_add(width).saturating_add(reserved) {
            if gap > 0 {
                spans.push(Span::styled(" ", Style::default().bg(app.theme.surface)));
                x = x.saturating_add(1);
            }
            app.mouse.register(Rect { x, y: area.y, width, height: 1 }, target.clone());
            spans.push(Span::styled(chip, button::style(app, ButtonKind::Ghost, app.is_hovered(&target))));
            x = x.saturating_add(width);
            needs_gap = true;
        }
    }

    let gap = u16::from(needs_gap);
    if right_edge.saturating_sub(x) >= gap.saturating_add(help_width) {
        if gap > 0 {
            spans.push(Span::styled(" ", Style::default().bg(app.theme.surface)));
            x = x.saturating_add(1);
        }
        app.mouse.register(Rect { x, y: area.y, width: help_width, height: 1 }, help_target.clone());
        spans.push(Span::styled(help_chip, button::style(app, ButtonKind::Ghost, app.is_hovered(&help_target))));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(app.theme.surface).fg(app.theme.fg)),
        area,
    );
}

fn status_prefix(app: &App) -> String {
    let label = mode_label(app);
    match app.view {
        View::Files => {
            let transfers = app.transfer_queue.active_count();
            format!(
                " {label}  {}:{} · {} selected · {} {} · mouse {}  ",
                app.current_host().map(|h| h.alias.as_str()).unwrap_or("no-host"),
                app.remote_path,
                app.selected_files,
                transfers,
                if transfers == 1 { "transfer" } else { "transfers" },
                if app.mouse_enabled { "on" } else { "off" }
            )
        }
        View::CommandRunner => format!(
            " {label}  {} on {} {}  ",
            if app.command_input.is_empty() { "no command yet" } else { app.command_input.as_str() },
            app.current_host().map(|h| h.alias.as_str()).unwrap_or("no-host"),
            if app.ascii { app.animator.ascii_spinner() } else { app.animator.spinner() }
        ),
        _ => format!(
            " {label}  {} servers · mouse {}  ",
            app.hosts.len(),
            if app.mouse_enabled { "on" } else { "off" }
        ),
    }
}

fn mode_label(app: &App) -> &'static str {
    match app.mode {
        Mode::Normal => match app.view {
            View::Dashboard => "dashboard",
            View::HostDetail => "host details",
            View::Files => "files",
            View::Tunnels => "tunnels",
            View::CommandRunner => "commands",
            View::Logs => "logbook",
            View::Settings => "settings",
            View::Help => "help",
        },
        Mode::Search => "searching",
        Mode::Visual => "selecting",
        Mode::Command => "file command",
        Mode::Rename => "renaming",
        Mode::Confirm => "confirming",
        Mode::Transfer => "transfers",
        Mode::Palette => "command palette",
        Mode::HostForm => "editing host",
    }
}

fn take_shortcut(shortcuts: &mut Vec<String>, action: &str) -> Option<String> {
    let index = shortcuts.iter().position(|s| shortcut_action(s).eq_ignore_ascii_case(action))?;
    Some(shortcuts.remove(index))
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

fn fit_to_width(text: &str, max: u16) -> String {
    if display_width(text) <= max {
        return text.into();
    }
    if max == 0 {
        return String::new();
    }
    if max <= 3 {
        return ".".repeat(max as usize);
    }

    let marker_width: u16 = 3;
    let mut out = String::new();
    let mut width: u16 = 0;
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if width.saturating_add(ch_width).saturating_add(marker_width) > max {
            break;
        }
        out.push(ch);
        width = width.saturating_add(ch_width);
    }
    out.push_str("...");
    out
}
