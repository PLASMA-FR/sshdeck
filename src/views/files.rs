use ratatui::{prelude::*, widgets::*};
use crate::{app::App, widgets::{file_columns, file_preview, status_bar, transfer_progress}};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
    ]).split(area);
    let host = app.current_host().map(|h| h.alias.as_str()).unwrap_or("no-host");
    f.render_widget(
        Paragraph::new(format!(
            "SSHDeck Files: {}    Remote: {}:{}    Mode: {}",
            host,
            host,
            app.remote_path,
            if app.files_dual_pane { "Dual-pane" } else { "Remote" }
        ))
        .style(app.theme.title())
        .block(Block::bordered().border_style(app.theme.border())),
        chunks[0],
    );

    if app.files_dual_pane {
        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);
        let local = vec!["󰉋 screenshots/", "󰈙 backup.tar.gz", "󰈙 notes.txt", "󰈙 config.example.json"]
            .into_iter().map(String::from).collect();
        let remote = vec!["󰉋 public/", "󰉋 src/", "󰈙 package.json", "󰈙 app.js", "󰈙 .env"]
            .into_iter().map(String::from).collect();
        f.render_widget(file_columns::list(app, "Local Files", local, app.active_file_pane == 0), cols[0]);
        f.render_widget(file_columns::list(app, "Remote Files", remote, app.active_file_pane == 1), cols[1]);
    } else {
        let cols = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
        ]).split(chunks[1]);
        let parent = vec!["..", "etc/", "home/", "var/", "usr/"]
            .into_iter().map(String::from).collect();
        let cur = vec!["󰉋 public/", "󰉋 src/", "󰉋 node_modules/", "󰈙 package.json", "󰈙 README.md", "󰈙 app.js", "󰈙 .env"]
            .into_iter().map(String::from).collect();
        f.render_widget(file_columns::list(app, "Parent", parent, false), cols[0]);
        f.render_widget(file_columns::list(app, "Current", cur, true), cols[1]);
        let preview = r#"package.json
JSON file
2.1 KB
Modified today

{
  "scripts": {
    "start": "node app.js"
  }
}

Sensitive files like .env are blocked until confirmation."#;
        f.render_widget(file_preview::preview(app, preview.to_string()), cols[2]);
    }

    if app.mode == crate::app::Mode::Transfer {
        let r = Rect { x: area.x + area.width / 4, y: area.y + 5, width: area.width / 2, height: 10 };
        f.render_widget(Clear, r);
        f.render_widget(transfer_progress::queue(app), r);
    }
    status_bar::draw(
        f,
        app,
        chunks[2],
        "j/k move · h parent · l open · Space select · Tab dual-pane · u upload · d download · : command · T transfers",
    );
}
