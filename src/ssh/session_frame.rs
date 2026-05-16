use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

pub fn frame_chars(ascii: bool) -> FrameChars {
    if ascii {
        FrameChars {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
        }
    } else {
        FrameChars {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
        }
    }
}

pub fn frame_title(host: &str) -> String {
    format!(" SSHDeck SSH session: {host} ")
}

/// Draw a lightweight colored terminal frame before handing control to the
/// system ssh client. The remote shell still gets a normal terminal, but the
/// scroll region starts inside the frame so users get a strong visual cue that
/// they are inside an SSHDeck-managed session.
pub fn enter_session_frame<W: Write>(out: &mut W, host: &str, ascii: bool) -> io::Result<()> {
    let (width, height) = terminal::size().unwrap_or((100, 30));

    queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;

    if width < 24 || height < 6 {
        queue!(
            out,
            SetForegroundColor(Color::Cyan),
            Print(format!("SSHDeck → {host}\r\n\r\n")),
            ResetColor
        )?;
        out.flush()?;
        return Ok(());
    }

    let chars = frame_chars(ascii);
    let inner_width = width.saturating_sub(2) as usize;
    let title = frame_title(host);
    let safe_title = truncate_to_width(&title, inner_width.saturating_sub(2));
    let title_width = safe_title.chars().count();
    let remaining = inner_width.saturating_sub(title_width);
    let left_rule = remaining / 2;
    let right_rule = remaining.saturating_sub(left_rule);

    queue!(out, SetForegroundColor(Color::Cyan), MoveTo(0, 0), Print(chars.top_left))?;
    queue!(out, Print(chars.horizontal.to_string().repeat(left_rule)))?;
    queue!(out, SetForegroundColor(Color::White), Print(safe_title))?;
    queue!(out, SetForegroundColor(Color::Cyan), Print(chars.horizontal.to_string().repeat(right_rule)), Print(chars.top_right))?;

    for row in 1..height.saturating_sub(1) {
        queue!(
            out,
            SetForegroundColor(Color::DarkCyan),
            MoveTo(0, row),
            Print(chars.vertical),
            MoveTo(width.saturating_sub(1), row),
            Print(chars.vertical)
        )?;
    }

    let hint = " exit shell to return to SSHDeck ";
    let safe_hint = truncate_to_width(hint, inner_width.saturating_sub(2));
    let hint_width = safe_hint.chars().count();
    let remaining = inner_width.saturating_sub(hint_width);
    let left_rule = remaining / 2;
    let right_rule = remaining.saturating_sub(left_rule);
    let bottom = height.saturating_sub(1);
    queue!(out, SetForegroundColor(Color::Cyan), MoveTo(0, bottom), Print(chars.bottom_left))?;
    queue!(out, Print(chars.horizontal.to_string().repeat(left_rule)))?;
    queue!(out, SetForegroundColor(Color::DarkGrey), Print(safe_hint))?;
    queue!(out, SetForegroundColor(Color::Cyan), Print(chars.horizontal.to_string().repeat(right_rule)), Print(chars.bottom_right), ResetColor)?;

    // Keep normal shell scrolling away from the chrome. ANSI scroll regions are
    // 1-based and inclusive. The remote application may reset this itself, but
    // ordinary shells/prompts stay neatly inside the frame.
    write!(out, "\x1b[2;{}r", height.saturating_sub(1))?;
    queue!(out, MoveTo(1, 1), ResetColor)?;
    out.flush()?;
    Ok(())
}

pub fn leave_session_frame<W: Write>(out: &mut W) -> io::Result<()> {
    write!(out, "\x1b[r")?;
    queue!(out, ResetColor, Clear(ClearType::All), MoveTo(0, 0))?;
    out.flush()?;
    Ok(())
}

fn truncate_to_width(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        input.to_string()
    } else if max <= 1 {
        "…".chars().take(max).collect()
    } else {
        let mut out: String = input.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_frame_uses_plain_terminal_safe_chars() {
        let c = frame_chars(true);
        assert_eq!(c.horizontal, '-');
        assert_eq!(c.vertical, '|');
    }

    #[test]
    fn unicode_frame_uses_rounded_borders() {
        let c = frame_chars(false);
        assert_eq!(c.top_left, '╭');
        assert_eq!(c.bottom_right, '╯');
    }

    #[test]
    fn title_identifies_host_as_sshdeck_session() {
        assert_eq!(frame_title("web-prod-1"), " SSHDeck SSH session: web-prod-1 ");
    }
}
