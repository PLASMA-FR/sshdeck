use ratatui::{prelude::*, widgets::*};

use crate::{
    app::App,
    ssh::{command::ssh_command_for, host::SshHost},
    widgets::{server_meter, status_bar},
};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(5), Constraint::Length(1)]).split(area);
    f.render_widget(
        Paragraph::new("SSHDeck Host Detail")
            .style(app.theme.title())
            .block(Block::bordered().border_style(app.theme.border())),
        chunks[0],
    );

    let cols = Layout::horizontal([Constraint::Percentage(56), Constraint::Percentage(44)]).split(chunks[1]);

    if let Some(host) = app.current_host() {
        draw_inventory(f, app, host, cols[0]);
        draw_status(f, app, host, cols[1]);
    } else {
        f.render_widget(
            Paragraph::new("No host selected")
                .wrap(Wrap { trim: false })
                .block(Block::bordered().border_style(app.theme.border()).title(" Host ")),
            cols[0],
        );
    }

    status_bar::draw(f, app, chunks[2], "Enter connect · s files · t tunnel · r command · Esc back");
}

fn draw_inventory(f: &mut Frame, app: &App, host: &SshHost, area: Rect) {
    let tags = if host.tags.is_empty() { "none".into() } else { host.tags.join(", ") };
    let target = host.hostname.clone().unwrap_or_else(|| "from OpenSSH config".into());
    let user = host.user.clone().unwrap_or_else(|| "OpenSSH default".into());
    let group = host.group.clone().unwrap_or_else(|| "ungrouped".into());
    let notes = host.notes.clone().unwrap_or_else(|| "No note yet. Add the thing you usually forget.".into());
    let identity = host
        .identity_file
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "OpenSSH default / agent".into());
    let cert = host
        .certificate_file
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "none".into());

    let lines = vec![
        Line::from(Span::styled(host.alias.clone(), app.theme.title())),
        Line::from(format!("Login      {user}@{target}")),
        Line::from(format!("Port       {}", host.port_text())),
        Line::from(format!("Group      {group}")),
        Line::from(format!("Tags       {tags}")),
        Line::raw(""),
        Line::from(Span::styled(notes, app.theme.muted())),
        Line::raw(""),
        Line::from(Span::styled("OpenSSH material", app.theme.muted())),
        Line::from(format!("Identity   {identity}")),
        Line::from(format!("Cert       {cert}")),
        Line::raw(""),
        Line::from(Span::styled("Generated command", app.theme.muted())),
        Line::from(ssh_command_for(host)),
    ];

    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_style(app.theme.border()).title(" Inventory ")),
        area,
    );
}

fn draw_status(f: &mut Frame, app: &App, host: &SshHost, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Min(4),
    ])
    .split(area);
    let profile = host.access_profile();
    let mut profile_lines = vec![
        Line::from(format!("Auth   {}", profile.auth)),
        Line::from(format!("Path   {}", profile.path)),
        Line::from(format!("Agent  {}", profile.agent)),
        Line::from(format!("Trust  {}", profile.host_key)),
        Line::from(format!("Ports  {}", profile.forwards)),
        Line::from(Span::styled(profile.boundary, app.theme.muted())),
    ];
    if profile.warnings.is_empty() {
        profile_lines.push(Line::from(Span::styled("No local SSHDeck warning for this host.", app.theme.success())));
    } else {
        for warning in profile.warnings {
            profile_lines.push(Line::from(Span::styled(warning, app.theme.warning())));
        }
    }

    f.render_widget(
        Paragraph::new(profile_lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_style(app.theme.border()).title(" Access profile ")),
        rows[0],
    );
    f.render_widget(server_meter::gauge(app, "RAM", 0.42), rows[1]);
    f.render_widget(server_meter::gauge(app, "Disk", 0.37), rows[2]);
    f.render_widget(server_meter::gauge(app, "Services", 0.1), rows[3]);
    f.render_widget(
        Paragraph::new(format!(
            "Uptime: {}\nKernel: {}\nFailed services: {}\nDocker containers: {}",
            app.health.uptime, app.health.kernel, app.health.failed_services, app.health.docker_containers
        ))
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title(" Health ")),
        rows[4],
    );
}
