use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName { Blackout, Cyber, Minimal }

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: ThemeName,
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub surface: Color,
    pub overlay: Color,
    pub border_color: Color,
    pub accent: Color,
    pub accent2: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Theme {
    pub fn named(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "cyber" => Self { name: ThemeName::Cyber, bg: Color::Rgb(5, 8, 14), fg: Color::Rgb(214, 232, 255), muted: Color::Rgb(90, 116, 140), surface: Color::Rgb(13, 18, 30), overlay: Color::Rgb(22, 28, 44), border_color: Color::Rgb(45, 61, 79), accent: Color::Rgb(0, 220, 190), accent2: Color::Rgb(180, 110, 255), success: Color::Rgb(80, 220, 123), warning: Color::Rgb(230, 180, 90), error: Color::Rgb(235, 90, 95) },
            "minimal" => Self { name: ThemeName::Minimal, bg: Color::Black, fg: Color::Gray, muted: Color::DarkGray, surface: Color::Black, overlay: Color::Black, border_color: Color::DarkGray, accent: Color::White, accent2: Color::Gray, success: Color::Green, warning: Color::Yellow, error: Color::Red },
            _ => Self::blackout(),
        }
    }
    pub fn blackout() -> Self {
        Self { name: ThemeName::Blackout, bg: Color::Rgb(0, 0, 0), fg: Color::Rgb(238, 241, 245), muted: Color::Rgb(115, 121, 132), surface: Color::Rgb(8, 10, 13), overlay: Color::Rgb(15, 18, 23), border_color: Color::Rgb(49, 54, 63), accent: Color::Rgb(86, 182, 255), accent2: Color::Rgb(139, 170, 196), success: Color::Rgb(74, 222, 128), warning: Color::Rgb(245, 190, 88), error: Color::Rgb(248, 113, 113) }
    }
    pub fn title(&self) -> Style { Style::default().fg(self.fg).add_modifier(Modifier::BOLD) }
    pub fn accent(&self) -> Style { Style::default().fg(self.accent).add_modifier(Modifier::BOLD) }
    pub fn selected(&self) -> Style { Style::default().fg(self.fg).bg(Color::Rgb(10, 43, 62)).add_modifier(Modifier::BOLD) }
    pub fn muted(&self) -> Style { Style::default().fg(self.muted) }
    pub fn normal(&self) -> Style { Style::default().fg(self.fg).bg(self.bg) }
    pub fn surface(&self) -> Style { Style::default().fg(self.fg).bg(self.surface) }
    pub fn overlay(&self) -> Style { Style::default().fg(self.fg).bg(self.overlay) }
    pub fn border(&self) -> Style { Style::default().fg(self.border_color) }
    pub fn active_border(&self) -> Style { Style::default().fg(self.accent) }
    pub fn inactive_border(&self) -> Style { Style::default().fg(self.border_color) }
    pub fn error(&self) -> Style { Style::default().fg(self.error) }
    pub fn warning(&self) -> Style { Style::default().fg(self.warning) }
    pub fn success(&self) -> Style { Style::default().fg(self.success) }
}
