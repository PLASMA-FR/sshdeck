use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn large_logo(unicode: bool, nerd: bool) -> String {
    if !unicode { return ascii_logo_mark("*"); }
    unicode_logo_mark(nerd, "◆")
}

pub fn splash_mark(unicode: bool, nerd: bool, shimmer: &str) -> String {
    if !unicode { return ascii_logo_mark(shimmer); }
    unicode_logo_mark(nerd, shimmer)
}

fn unicode_logo_mark(nerd: bool, shimmer: &str) -> String {
    let prompt = if nerd { "❯_" } else { ">_" };
    format!(r#"          ╭────────────────────╮
       ╭──┤  {prompt:<3}               │╮
       │  │                    ││
       │  │        ╭──╮        ││
       │  │       ╱    ╲       ││
   {shimmer}───┤       ╲    ╱       ││
       │  │        ╲  ╱        ││
       │  │         ╲╱         ││
       │  │   ○──────────◉    ││
       │  │                    ││
       │  │   ○──────────◉    ││
       ╰──┤      SSHDeck      │╯
          ╰────────────────────╯
            ╰──────────────────╯"#)
}

fn ascii_logo_mark(shimmer: &str) -> String {
    format!(r#"        .--------------------.
     .--|  >_                |.
     |  |                    ||
     |  |        .--.        ||
     |  |       /    \       ||
 {shimmer}---|       \    /       ||
     |  |        \  /        ||
     |  |         \/         ||
     |  |   o----------o    ||
     |  |                    ||
     |  |   o----------o    ||
     '--|      SSHDeck      |'
        '--------------------'
          '------------------'"#)
}

pub fn compact(app:&App)->String { if app.ascii { "[>_] SSHDeck".into() } else if app.config.ui.nerd_font { "󰣀 SSHDeck".into() } else { "▦ SSHDeck".into() } }
pub fn options()->[&'static str;3]{["[>_] SSHDeck","╭─ SSHDeck ─╮","󰣀 SSHDeck"]}

pub fn block<'a>(app:&App)->Paragraph<'a>{
    Paragraph::new(format!("{}\nLocal-first SSH, files, tunnels. Your config stays yours.", compact(app)))
        .style(app.theme.title())
        .block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" SSHDeck "))
}

pub fn splash<'a>(app:&App)->Paragraph<'a>{
    let mark = splash_mark(!app.ascii, app.config.ui.nerd_font, app.animator.shimmer());
    let scan = if app.ascii { "------------------------".to_string() } else { app.animator.scanline(24) };
    let text = format!("{mark}\n\nSSHDeck\nTermius for the terminal\nNo cloud · No account · No Electron\n\n{scan}\n{} Loading command center", if app.ascii { app.animator.ascii_spinner() } else { app.animator.spinner() });
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(app.theme.title())
        .block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.active_border()).title(" SSHDeck "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_logo_matches_generated_logo_motifs() {
        let logo = splash_mark(true, false, "◆");
        assert!(logo.contains(">_"));
        assert!(logo.contains("╭──╮"));
        assert!(logo.contains("╲╱"));
        assert!(logo.contains("○──────────◉"));
        assert!(logo.contains("╰──────────────────╯"));
    }

    #[test]
    fn ascii_logo_keeps_same_motifs_without_unicode() {
        let logo = splash_mark(false, false, "*");
        assert!(logo.contains(">_"));
        assert!(logo.contains(".--."));
        assert!(logo.contains("o----------o"));
        assert!(logo.contains("SSHDeck"));
    }
}
