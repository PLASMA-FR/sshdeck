use ratatui::{prelude::*, widgets::*};

use crate::{
    app::App,
    mouse::ClickTarget,
    widgets::{
        button::{self, ButtonKind},
        logo, status_bar,
    },
};

const NAV_ITEMS: [&str; 9] = [
    "All",
    "Favorites",
    "Production",
    "Homelab",
    "Recent",
    "Tunnels",
    "Commands",
    "Logs",
    "Settings",
];

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let shell = crate::design::layout::app_shell(area);

    f.render_widget(logo::block(app), shell[0]);

    if app.hosts.is_empty() {
        draw_empty_state(f, app, shell[1]);
    } else {
        let cols = crate::design::layout::dashboard(shell[1]);
        draw_nav(f, app, cols[0]);
        draw_hosts(f, app, cols[1]);
        draw_details(f, app, cols[2]);
    }

    status_bar::draw(
        f,
        app,
        shell[2],
        "/ search · a add · Enter connect · s files · t tunnel · h health · r command · ? help",
    );
}

fn draw_empty_state(f: &mut Frame, app: &mut App, area: Rect) {
    let card = crate::design::layout::centered(area, 58, 15);
    f.render_widget(Clear, card);

    let add_target = ClickTarget::ModalButton("add-host".into());
    let import_target = ClickTarget::ModalButton("import-hosts".into());
    app.mouse.register(Rect { x: card.x + 8, y: card.y + 7, width: 14, height: 1 }, add_target.clone());
    app.mouse.register(Rect { x: card.x + 25, y: card.y + 7, width: 24, height: 1 }, import_target.clone());

    let text = vec![
        Line::from(Span::styled("No hosts here yet", app.theme.title())),
        Line::raw(""),
        Line::from("SSHDeck reads ~/.ssh/config on startup and can keep its own host file."),
        Line::from("It will not rewrite your SSH setup unless you choose the Include helper."),
        Line::raw(""),
        Line::from(Span::styled("Start with one host:", app.theme.muted())),
        Line::from(vec![
            button::label(app, "Add Host", &add_target, ButtonKind::Primary),
            Span::raw("  "),
            button::label(app, "Use ~/.ssh/config", &import_target, ButtonKind::Secondary),
        ]),
        Line::raw(""),
        Line::from(Span::styled("Managed hosts live in ~/.config/sshdeck/ssh_config", app.theme.muted())),
    ];

    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(app.theme.surface())
            .block(
                Block::bordered()
                    .border_type(crate::design::borders::rounded(!app.ascii))
                    .border_style(app.theme.border())
                    .title(" Welcome "),
            ),
        card,
    );
}

fn draw_nav(f: &mut Frame, app: &mut App, area: Rect) {
    let mut rows = Vec::new();

    for (index, label) in NAV_ITEMS.iter().enumerate() {
        let target = ClickTarget::SidebarGroup((*label).into());
        app.mouse.register(
            Rect { x: area.x + 1, y: area.y + 1 + index as u16, width: area.width.saturating_sub(2), height: 1 },
            target.clone(),
        );
        rows.push(ListItem::new(nav_label(label)).style(button::row_style(app, &target, false)));
    }

    f.render_widget(
        List::new(rows).style(app.theme.normal()).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.inactive_border())
                .title(" Places "),
        ),
        area,
    );
}

fn nav_label(label: &str) -> String {
    match label {
        "All" => "all servers".into(),
        "Favorites" => "starred".into(),
        "Production" => "production".into(),
        "Homelab" => "homelab".into(),
        "Recent" => "recent".into(),
        "Tunnels" => "tunnels".into(),
        "Commands" => "commands".into(),
        "Logs" => "logbook".into(),
        "Settings" => "settings".into(),
        _ => label.into(),
    }
}

fn draw_hosts(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_rows = area.height.saturating_sub(4) as usize;
    keep_selection_visible(app, visible_rows);

    let add_target = ClickTarget::ModalButton("add-host".into());
    app.mouse.register(
        Rect { x: area.x + area.width.saturating_sub(16), y: area.y + 1, width: 14, height: 1 },
        add_target.clone(),
    );

    let mut rows = vec![ListItem::new(Line::from(vec![
        Span::styled("Servers", app.theme.title()),
        Span::raw("  "),
        Span::styled(format!("{} total", app.filtered.len()), app.theme.muted()),
        Span::raw("                 "),
        button::label(app, "+ Add", &add_target, ButtonKind::Secondary),
    ]))];

    for (display_pos, host_idx) in app.filtered.iter().enumerate().skip(app.host_scroll).take(visible_rows) {
        if let Some(host) = app.hosts.get(*host_idx) {
            let selected = display_pos == app.selected;
            let target = ClickTarget::HostRow(*host_idx);
            let y = area.y + 2 + (display_pos - app.host_scroll) as u16;
            app.mouse.register(Rect { x: area.x + 1, y, width: area.width.saturating_sub(2), height: 1 }, target.clone());

            rows.push(ListItem::new(host_row(app, host, *host_idx)).style(button::row_style(app, &target, selected)));
        }
    }

    let mut state = ListState::default();
    state.select(Some(app.selected.saturating_sub(app.host_scroll) + 1));

    let list = List::new(rows)
        .block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.border())
                .title(" Hosts "),
        )
        .highlight_symbol(" ");

    f.render_stateful_widget(list, area, &mut state);
}

fn keep_selection_visible(app: &mut App, visible_rows: usize) {
    if app.selected < app.host_scroll {
        app.host_scroll = app.selected;
    }
    if app.selected >= app.host_scroll + visible_rows {
        app.host_scroll = app.selected.saturating_sub(visible_rows.saturating_sub(1));
    }
}

fn host_row(app: &App, host: &crate::ssh::host::SshHost, index: usize) -> String {
    let source = if app.managed_aliases.contains(&host.alias) { "managed" } else { "ssh" };
    let group = host.group.clone().unwrap_or_else(|| "ungrouped".into());
    let favorite = if host.favorite { " ★" } else { "" };
    let marker = if app.selected == index { "›" } else { " " };
    let user = host.user.clone().unwrap_or_else(|| "default".into());
    format!("{marker} {:<20} {:<10} {:<9} {source}{favorite}", host.alias, group, user)
}

fn draw_details(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(host) = app.current_host() else { return; };

    let alias = host.alias.clone();
    let idx = app.current_host_index().unwrap_or(0);
    let target = host.hostname.clone().unwrap_or_else(|| "from ~/.ssh/config".into());
    let user = host.user.clone().unwrap_or_else(|| "OpenSSH default".into());
    let notes = host.notes.clone().unwrap_or_else(|| "No note yet. Add the thing you usually forget.".into());
    let group = host.group.clone().unwrap_or_else(|| "Ungrouped".into());
    let tags = if host.tags.is_empty() { "No tags".into() } else { host.tags.join(", ") };

    let connect = ClickTarget::HostConnectButton(idx);
    let files = ClickTarget::HostFilesButton(idx);
    let tunnel = ClickTarget::HostTunnelButton(idx);
    let edit = ClickTarget::HostEditButton(idx);
    let health = ClickTarget::HostHealthButton(idx);

    let button_rows = [
        [("Connect", &connect, ButtonKind::Primary), ("Files", &files, ButtonKind::Secondary)],
        [("Tunnel", &tunnel, ButtonKind::Secondary), ("Check", &health, ButtonKind::Secondary)],
        [("Edit", &edit, ButtonKind::Secondary), ("", &edit, ButtonKind::Secondary)],
    ];

    let mut lines = vec![
        Line::from(Span::styled(alias.clone(), app.theme.title())),
        Line::from(Span::styled("ready", app.theme.success())),
        Line::raw(""),
        Line::from(format!("login  {}@{}", user, target)),
        Line::from(format!("where  port {} · {}", host.port_text(), group)),
        Line::from(Span::styled(format!("tags   {tags}"), app.theme.muted())),
        Line::raw(""),
        Line::from(Span::styled(notes, app.theme.muted())),
        Line::raw(""),
    ];

    for (row_index, row) in button_rows.iter().enumerate() {
        let mut spans = Vec::new();
        for (col_index, (label, target, kind)) in row.iter().enumerate() {
            if label.is_empty() { continue; }
            let x = area.x + 2 + col_index as u16 * 14;
            let y = area.y + 10 + row_index as u16;
            app.mouse.register(Rect { x, y, width: 13, height: 1 }, (*target).clone());
            spans.push(button::label(app, label, target, *kind));
        }
        lines.push(Line::from(spans));
    }

    lines.extend([
        Line::raw(""),
        Line::from(Span::styled("What happens next", app.theme.muted())),
        Line::from(format!("connect  ssh {}", alias)),
        Line::from("files    browse through your system OpenSSH"),
        Line::from("health   queues uptime, disk, memory, and kernel checks"),
        Line::from("command  opens the guarded command runner"),
    ]);

    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).style(app.theme.normal()).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.inactive_border())
                .title(" Details "),
        ),
        area,
    );
}
