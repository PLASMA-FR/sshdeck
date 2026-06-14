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
            "high-contrast" => Self { name: ThemeName::Minimal, bg: Color::Black, fg: Color::White, muted: Color::Gray, surface: Color::Black, overlay: Color::Rgb(18, 18, 18), border_color: Color::White, accent: Color::Cyan, accent2: Color::Blue, success: Color::Green, warning: Color::Yellow, error: Color::Red },
            _ => Self::blackout(),
        }
    }
    pub fn blackout() -> Self {
        Self { name: ThemeName::Blackout, bg: Color::Rgb(10, 9, 8), fg: Color::Rgb(238, 232, 219), muted: Color::Rgb(148, 137, 122), surface: Color::Rgb(18, 16, 14), overlay: Color::Rgb(28, 25, 21), border_color: Color::Rgb(67, 58, 48), accent: Color::Rgb(218, 166, 82), accent2: Color::Rgb(111, 161, 151), success: Color::Rgb(123, 174, 130), warning: Color::Rgb(224, 168, 80), error: Color::Rgb(211, 101, 84) }
    }
    fn selection_bg(&self) -> Color { match self.name { ThemeName::Blackout => Color::Rgb(52, 43, 31), ThemeName::Cyber => Color::Rgb(10, 43, 62), ThemeName::Minimal => Color::DarkGray } }
    fn hover_bg(&self) -> Color { match self.name { ThemeName::Blackout => Color::Rgb(65, 52, 36), ThemeName::Cyber => Color::Rgb(18, 55, 77), ThemeName::Minimal => Color::Gray } }
    fn primary_hover_bg(&self) -> Color { match self.name { ThemeName::Blackout => Color::Rgb(235, 190, 112), ThemeName::Cyber => Color::Rgb(128, 210, 255), ThemeName::Minimal => Color::White } }
    fn secondary_hover_bg(&self) -> Color { match self.name { ThemeName::Blackout => Color::Rgb(41, 36, 31), ThemeName::Cyber => Color::Rgb(32, 44, 55), ThemeName::Minimal => Color::DarkGray } }
    pub fn title(&self) -> Style { Style::default().fg(self.fg).add_modifier(Modifier::BOLD) }
    pub fn accent(&self) -> Style { Style::default().fg(self.accent).add_modifier(Modifier::BOLD) }
    pub fn selected(&self) -> Style { Style::default().fg(self.fg).bg(self.selection_bg()).add_modifier(Modifier::BOLD) }
    pub fn hovered(&self) -> Style { Style::default().fg(self.fg).bg(self.hover_bg()).add_modifier(Modifier::BOLD | Modifier::UNDERLINED) }
    pub fn button_primary(&self) -> Style { Style::default().fg(self.bg).bg(self.accent).add_modifier(Modifier::BOLD) }
    pub fn button_primary_hover(&self) -> Style { Style::default().fg(self.bg).bg(self.primary_hover_bg()).add_modifier(Modifier::BOLD) }
    pub fn button_secondary(&self) -> Style { Style::default().fg(self.fg).bg(self.overlay).add_modifier(Modifier::BOLD) }
    pub fn button_secondary_hover(&self) -> Style { Style::default().fg(self.fg).bg(self.secondary_hover_bg()).add_modifier(Modifier::BOLD | Modifier::UNDERLINED) }
    pub fn button_danger(&self) -> Style { Style::default().fg(self.error).bg(self.overlay).add_modifier(Modifier::BOLD) }
    pub fn button_danger_hover(&self) -> Style { Style::default().fg(Color::Black).bg(self.error).add_modifier(Modifier::BOLD) }
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
