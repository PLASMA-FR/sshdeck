#[derive(Debug, Clone)]
pub struct Animator { pub enabled: bool, frame: usize }
impl Animator {
    pub fn new(enabled: bool) -> Self { Self { enabled, frame: 0 } }
    pub fn tick(&mut self) { if self.enabled { self.frame = self.frame.wrapping_add(1); } }
    pub fn spinner(&self) -> &'static str { if !self.enabled { "•" } else { ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"][self.frame % 10] } }
    pub fn ascii_spinner(&self) -> &'static str { if !self.enabled { "*" } else { ["-","\\","|","/"][self.frame % 4] } }
    pub fn flow(&self) -> &'static str { ["→","⇒","⇢","⇨"][self.frame % 4] }
    pub fn transfer_dots(&self) -> &'static str { ["●····","·●···","··●··","···●·","····●"][self.frame % 5] }
    pub fn pulse_index(&self) -> usize { self.frame % 6 }
    pub fn logo_phase(&self) -> usize { if self.enabled { self.frame } else { 0 } }
    pub fn shimmer(&self) -> &'static str { if !self.enabled { "◆" } else { ["◇","◈","◆","◈"][self.frame % 4] } }
    pub fn scanline(&self, width: usize) -> String {
        let width = width.max(8);
        let pos = if self.enabled { self.frame % width } else { width / 2 };
        (0..width).map(|i| if i == pos { '◆' } else if i.abs_diff(pos) == 1 { '◇' } else { '─' }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanline_keeps_requested_width() {
        let animator = Animator::new(false);
        assert_eq!(animator.scanline(12).chars().count(), 12);
    }

    #[test]
    fn shimmer_has_ascii_safe_fallback_when_disabled() {
        let animator = Animator::new(false);
        assert_eq!(animator.shimmer(), "◆");
    }
}
