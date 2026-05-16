use ratatui::prelude::*;

use crate::{app::App, mouse::ClickTarget, widgets::button};

/// Compact host-row renderer shared by dashboard-like screens.
pub fn line(app: &App, host: &crate::ssh::host::SshHost, index: usize) -> Line<'static> {
    let target = ClickTarget::HostRow(index);
    let dot = if host.favorite { "●" } else { "○" };
    let user = host.user.clone().unwrap_or_else(|| "default".into());
    let host_name = host.hostname.clone().unwrap_or_else(|| "from ssh config".into());
    let group = host.group.clone().unwrap_or_else(|| "ungrouped".into());
    Line::from(vec![
        Span::styled(format!("{dot} "), app.theme.accent()),
        Span::styled(host.alias.clone(), button::row_style(app, &target, false)),
        Span::styled(format!("  {user}@{host_name} · {group}"), app.theme.muted()),
    ])
}
