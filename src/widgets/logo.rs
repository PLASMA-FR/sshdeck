use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn large_logo(unicode: bool, nerd: bool) -> String {
    if !unicode { return ascii_logo_mark("*", 0); }
    unicode_logo_mark(nerd, "◆", 0)
}

pub fn splash_mark(unicode: bool, nerd: bool, shimmer: &str) -> String {
    animated_splash_mark(unicode, nerd, shimmer, 0)
}

pub fn animated_splash_mark(unicode: bool, nerd: bool, shimmer: &str, phase: usize) -> String {
    if !unicode { return ascii_logo_mark(shimmer, phase); }
    unicode_logo_mark(nerd, shimmer, phase)
}

fn unicode_flow(width: usize, phase: usize, reverse: bool) -> String {
    let pos = phase % width;
    (0..width).map(|i| {
        let hit = if reverse { width - 1 - pos } else { pos };
        if i == hit { '◆' } else if i.abs_diff(hit) == 1 { '◇' } else { '─' }
    }).collect()
}

fn ascii_flow(width: usize, phase: usize, reverse: bool) -> String {
    let pos = phase % width;
    (0..width).map(|i| {
        let hit = if reverse { width - 1 - pos } else { pos };
        if i == hit { '*' } else if i.abs_diff(hit) == 1 { '=' } else { '-' }
    }).collect()
}

fn unicode_logo_mark(nerd: bool, shimmer: &str, phase: usize) -> String {
    let cursor = if phase % 2 == 0 { "_" } else { " " };
    let prompt = if nerd { format!("❯{cursor}") } else { format!(">{cursor}") };
    let core = ["◇◇", "◆◇", "◇◆", "◆◆"][phase % 4];
    let lock = ["╲╱", "◇◇", "╱╲", "◆◆"][phase % 4];
    let ssh_flow = unicode_flow(14, phase, false);
    let sftp_flow = unicode_flow(14, phase + 3, true);
    format!(r#"              ╭────────────────────────────╮
          ╭───┤  {prompt:<3} SSHDeck              │╮
          │   │  ────────────────          ││
          │   │                            ││
          │   │         ╭────────╮         ││
      {shimmer}───┤   │      ╭──┤  {core}  ├──╮      ││
          │   │      │  │ ╲  ╱ │  │      ││
          │   │      │  │  {lock}  │  │      ││
          │   │      │  │  ╱╲  │  │      ││
          │   │      ╰──┤ ╱  ╲ ├──╯      ││
          │   │         ╰────────╯        ││
          │   │                            ││
          │   │  ○{ssh_flow}◉  ssh     ││
          │   │  │              │          ││
          │   │  ○{sftp_flow}◉  sftp    ││
          │   │                            ││
          ╰───┤     No cloud · OpenSSH     │╯
              ╰────────────────────────────╯
                ╰──────────────────────────╯
                  ╰────────────────────────╯"#)
}

fn ascii_logo_mark(shimmer: &str, phase: usize) -> String {
    let cursor = if phase % 2 == 0 { "_" } else { " " };
    let core = ["**", "*+", "+*", "++"][phase % 4];
    let lock = ["\\/", "**", "/\\", "++"][phase % 4];
    let ssh_flow = ascii_flow(14, phase, false);
    let sftp_flow = ascii_flow(14, phase + 3, true);
    format!(r#"            .----------------------------.
        .---|  >{cursor} SSHDeck                |.
        |   |  ----------------          ||
        |   |                            ||
        |   |         .--------.         ||
    {shimmer}---|   |      .--|  {core}  |--.      ||
        |   |      |  | \\  / |  |      ||
        |   |      |  |  {lock}  |  |      ||
        |   |      |  |  /\\  |  |      ||
        |   |      '--| /  \\ |--'      ||
        |   |         '--------'        ||
        |   |                            ||
        |   |  o{ssh_flow}o  ssh     ||
        |   |  |              |          ||
        |   |  o{sftp_flow}o  sftp    ||
        |   |                            ||
        '---|     No cloud / OpenSSH     |'
            '----------------------------'
              '--------------------------'
                '------------------------'"#)
}

pub fn compact(app:&App)->String { if app.ascii { "[>_] SSHDeck".into() } else if app.config.ui.nerd_font { "󰣀 SSHDeck".into() } else { "▦ SSHDeck".into() } }
pub fn options()->[&'static str;3]{["[>_] SSHDeck","╭─ SSHDeck ─╮","󰣀 SSHDeck"]}

pub fn block<'a>(app:&App)->Paragraph<'a>{
    Paragraph::new(format!("{}\nLocal-first SSH, files, tunnels. Your config stays yours.", compact(app)))
        .style(app.theme.title())
        .block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" SSHDeck "))
}

pub fn splash<'a>(app:&App)->Paragraph<'a>{
    let mark = animated_splash_mark(!app.ascii, app.config.ui.nerd_font, app.animator.shimmer(), app.animator.logo_phase());
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
        assert!(logo.contains("╭────────╮"));
        assert!(logo.contains("╲╱"));
        assert!(logo.contains("○◆◇────────────◉"));
        assert!(logo.contains("No cloud · OpenSSH"));
        assert!(logo.contains("╰──────────────────────────╯"));
    }

    #[test]
    fn animated_logo_changes_cursor_core_and_flow() {
        let first = animated_splash_mark(true, false, "◇", 0);
        let second = animated_splash_mark(true, false, "◆", 1);
        assert_ne!(first, second);
        assert!(first.contains(">_  SSHDeck"));
        assert!(second.contains(">   SSHDeck"));
        assert!(first.contains("○◆◇────────────◉"));
        assert!(second.contains("○◇◆◇───────────◉"));
    }

    #[test]
    fn ascii_logo_keeps_same_motifs_without_unicode() {
        let logo = splash_mark(false, false, "*");
        assert!(logo.contains(">_"));
        assert!(logo.contains(".--------."));
        assert!(logo.contains("o*=------------o"));
        assert!(logo.contains("No cloud / OpenSSH"));
        assert!(logo.contains("SSHDeck"));
    }
}
