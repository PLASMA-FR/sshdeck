use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn large_logo(unicode: bool, nerd: bool) -> String {
    if !unicode { return "[SSHDeck]\nlocal-first SSH command center".into(); }
    let badge = if nerd { "󰣀 SSHDeck " } else { "SSHDeck" };
    format!(r#"╭────────────────────────────╮
│   ███████╗███████╗██╗  ██╗ │
│   ██╔════╝██╔════╝██║  ██║ │
│   ███████╗███████╗███████║ │
│   ╚════██║╚════██║██╔══██║ │
│   ███████║███████║██║  ██║ │
│   ╚══════╝╚══════╝╚═╝  ╚═╝ │
│        {badge:<18}│
│  local-first command center│
╰────────────────────────────╯"#)
}

pub fn compact(app:&App)->String { if app.ascii { "SSHDeck".into() } else if app.config.ui.nerd_font { "󰣀 SSHDeck".into() } else { "SSHDeck".into() } }
pub fn options()->[&'static str;3]{["SSHDeck","╭─ SSHDeck ─╮","󰣀 SSHDeck"]}

pub fn block<'a>(app:&App)->Paragraph<'a>{
    Paragraph::new(format!("{}\nLocal-first SSH, files, tunnels. Your config stays yours.", compact(app)))
        .style(app.theme.title())
        .block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" SSHDeck "))
}
