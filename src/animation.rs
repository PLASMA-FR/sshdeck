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
}
