use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName { Default, Cyber, Minimal }

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: ThemeName,
    pub fg: Color,
    pub muted: Color,
    pub surface: Color,
    pub accent: Color,
    pub accent2: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Theme {
    pub fn named(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "cyber" => Self { name: ThemeName::Cyber, fg: Color::Rgb(214, 232, 255), muted: Color::Rgb(90, 116, 140), surface: Color::Rgb(13, 18, 30), accent: Color::Rgb(0, 255, 209), accent2: Color::Rgb(255, 0, 128), success: Color::Rgb(80, 250, 123), warning: Color::Rgb(255, 184, 108), error: Color::Rgb(255, 85, 85) },
            "minimal" => Self { name: ThemeName::Minimal, fg: Color::Gray, muted: Color::DarkGray, surface: Color::Black, accent: Color::White, accent2: Color::Gray, success: Color::Green, warning: Color::Yellow, error: Color::Red },
            _ => Self { name: ThemeName::Default, fg: Color::Rgb(192, 202, 245), muted: Color::Rgb(86, 95, 137), surface: Color::Rgb(31, 34, 53), accent: Color::Rgb(122, 162, 247), accent2: Color::Rgb(187, 154, 247), success: Color::Rgb(158, 206, 106), warning: Color::Rgb(224, 175, 104), error: Color::Rgb(247, 118, 142) },
        }
    }
    pub fn title(&self) -> Style { Style::default().fg(self.accent).add_modifier(Modifier::BOLD) }
    pub fn selected(&self) -> Style { Style::default().fg(Color::Black).bg(self.accent).add_modifier(Modifier::BOLD) }
    pub fn muted(&self) -> Style { Style::default().fg(self.muted) }
    pub fn normal(&self) -> Style { Style::default().fg(self.fg) }
    pub fn border(&self) -> Style { Style::default().fg(self.accent) }
    pub fn inactive_border(&self) -> Style { Style::default().fg(self.muted) }
}
