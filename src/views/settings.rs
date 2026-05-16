use ratatui::{prelude::*, widgets::*};

use crate::{
    app::App,
    mouse::ClickTarget,
    widgets::{button, status_bar},
};

struct SettingRow<'a> {
    id: &'a str,
    label: &'a str,
    value: String,
    hint: &'a str,
    mutable: bool,
}

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let shell = crate::design::layout::app_shell(area);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![Span::styled("Settings", app.theme.title()), Span::styled("  local preferences, saved immediately", app.theme.muted())]),
            Line::from(Span::styled("Click a row, or use j/k and Enter. Nothing here touches your ~/.ssh/config.", app.theme.muted())),
        ])
        .style(app.theme.surface())
        .block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.border())
                .title(" SSHDeck "),
        ),
        shell[0],
    );

    let cols = Layout::horizontal([
        Constraint::Percentage(62),
        Constraint::Percentage(38),
    ])
    .split(shell[1]);

    draw_settings_list(f, app, cols[0]);
    draw_settings_notes(f, app, cols[1]);
    status_bar::draw(f, app, shell[2], "j/k move · Enter toggle · , settings · Esc back · ? help");
}

fn rows(app: &App) -> Vec<SettingRow<'static>> {
    vec![
        SettingRow { id: "theme", label: "Theme", value: app.config.ui.theme.clone(), hint: "cycle blackout, minimal, cyber", mutable: true },
        SettingRow { id: "animations", label: "Animations", value: on_off(app.config.ui.animations), hint: "spinners, shimmer, transfer dots", mutable: true },
        SettingRow { id: "unicode", label: "Unicode", value: on_off(app.config.ui.unicode), hint: "rounded borders and symbols", mutable: true },
        SettingRow { id: "nerd_font", label: "Nerd Font", value: on_off(app.config.ui.nerd_font), hint: "server/file glyphs when your terminal supports them", mutable: true },
        SettingRow { id: "mouse", label: "Mouse", value: on_off(app.config.ui.mouse), hint: "clicks, scroll, right-click menus", mutable: true },
        SettingRow { id: "show_hidden", label: "Hidden files", value: if app.config.files.show_hidden { "shown".into() } else { "hidden".into() }, hint: "default for SSHDeck Files", mutable: true },
        SettingRow { id: "default_local_dir", label: "Local folder", value: app.config.files.default_local_dir.clone(), hint: "used by dual-pane files", mutable: false },
        SettingRow { id: "config_path", label: "Config", value: app.config.path.display().to_string(), hint: "SSHDeck metadata file", mutable: false },
    ]
}

fn draw_settings_list(f: &mut Frame, app: &mut App, area: Rect) {
    let mut items = Vec::new();
    for (index, row) in rows(app).iter().enumerate() {
        let target = ClickTarget::SettingRow(row.id.into());
        let y = area.y + 1 + index as u16;
        app.mouse.register(
            Rect { x: area.x + 1, y, width: area.width.saturating_sub(2), height: 1 },
            target.clone(),
        );
        let selected = app.settings_selected == index;
        let style = button::row_style(app, &target, selected);
        let marker = if selected { "›" } else { " " };
        let action = if row.mutable { "toggle" } else { "view" };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{marker} {:<16}", row.label), if selected { app.theme.accent() } else { app.theme.normal() }),
            Span::styled(format!("{:<20}", truncate(&row.value, 20)), app.theme.title()),
            Span::styled(format!("  {} · {}", action, row.hint), app.theme.muted()),
        ])).style(style));
    }

    f.render_widget(
        List::new(items).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.active_border())
                .title(" Preferences "),
        ),
        area,
    );
}

fn draw_settings_notes(f: &mut Frame, app: &App, area: Rect) {
    let selected = rows(app).get(app.settings_selected).map(|r| r.id).unwrap_or("theme");
    let details = match selected {
        "theme" => "Blackout is the default: near-black, quiet borders, one soft accent. Cyber is still there, but not the house style.",
        "animations" => "Animations are deliberately small: startup identity, spinners, and transfer dots. Turning this off keeps the UI still.",
        "unicode" => "Turn this off for stricter terminals. SSHDeck falls back to square ASCII labels and simpler borders.",
        "nerd_font" => "Use this only if your terminal font supports Nerd Font glyphs. Otherwise disable it for clean plain labels.",
        "mouse" => "Mouse support uses crossterm capture and a region registry. Keyboard shortcuts remain the fastest path.",
        "show_hidden" => "Controls the default hidden-file visibility for SSHDeck Files. You can still toggle per session with '.'.",
        "default_local_dir" => "The starting local directory for dual-pane transfers. Editing custom paths will land in a later pass.",
        "config_path" => "This is SSHDeck's metadata file. Host blocks created by SSHDeck live separately in ~/.config/sshdeck/ssh_config.",
        _ => "",
    };

    let text = vec![
        Line::from(Span::styled("How this works", app.theme.title())),
        Line::raw(""),
        Line::from(details),
        Line::raw(""),
        Line::from(Span::styled("Saved locally", app.theme.muted())),
        Line::from(app.config.path.display().to_string()),
        Line::raw(""),
        Line::from(Span::styled("Safety", app.theme.muted())),
        Line::from("Settings changes never rewrite your original ~/.ssh/config."),
    ];

    f.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: false }).style(app.theme.normal()).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.inactive_border())
                .title(" Notes "),
        ),
        area,
    );
}

fn on_off(value: bool) -> String { if value { "on" } else { "off" }.into() }

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max { value.into() } else { let mut out = value.chars().take(max.saturating_sub(1)).collect::<String>(); out.push('…'); out }
}
