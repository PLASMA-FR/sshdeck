use std::time::{Duration, Instant};

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;

use crate::app::View;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickTarget {
    SidebarGroup(String),
    HostRow(usize),
    HostConnectButton(usize),
    HostEditButton(usize),
    HostFilesButton(usize),
    HostTunnelButton(usize),
    HostHealthButton(usize),
    FileEntry(String),
    FilePreview,
    Breadcrumb(String),
    CommandPaletteItem(String),
    ModalButton(String),
    Tab(View),
    TransferItem(u64),
    TunnelType(String),
    FormField(String),
    ToastClose,
    StatusShortcut(String),
    Pane(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseAction {
    Click(ClickTarget),
    DoubleClick(ClickTarget),
    RightClick(ClickTarget),
    Scroll { target: Option<ClickTarget>, delta: i16 },
    Drag { target: Option<ClickTarget>, x: u16, y: u16 },
    Move { target: Option<ClickTarget>, x: u16, y: u16 },
}

#[derive(Debug, Clone)]
pub struct Region { pub rect: Rect, pub target: ClickTarget }

#[derive(Debug, Default, Clone)]
pub struct RegionRegistry { regions: Vec<Region> }

impl RegionRegistry {
    pub fn clear(&mut self) { self.regions.clear(); }
    pub fn register(&mut self, rect: Rect, target: ClickTarget) { if rect.width > 0 && rect.height > 0 { self.regions.push(Region { rect, target }); } }
    pub fn hit(&self, x: u16, y: u16) -> Option<ClickTarget> { self.regions.iter().rev().find(|r| contains(r.rect, x, y)).map(|r| r.target.clone()) }
    pub fn len(&self) -> usize { self.regions.len() }
}

#[derive(Debug, Clone)]
pub struct MouseState { pub registry: RegionRegistry, last_click: Option<(ClickTarget, Instant)>, y_offset: i16 }
impl Default for MouseState { fn default() -> Self { Self { registry: RegionRegistry::default(), last_click: None, y_offset: 1 } } }

impl MouseState {
    pub fn begin_frame(&mut self) { self.registry.clear(); }
    pub fn register(&mut self, rect: Rect, target: ClickTarget) { self.registry.register(rect, target); }
    pub fn resolve(&mut self, event: MouseEvent) -> MouseAction {
        let (x, y) = self.normalized_point(event.column, event.row);
        let target = self.registry.hit(x, y);
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(t) = target.clone() {
                    let is_double = self.last_click.as_ref().is_some_and(|(last, at)| last == &t && at.elapsed() < Duration::from_millis(500));
                    self.last_click = Some((t.clone(), Instant::now()));
                    if is_double { MouseAction::DoubleClick(t) } else { MouseAction::Click(t) }
                } else { MouseAction::Click(ClickTarget::Pane("background".into())) }
            }
            MouseEventKind::Down(MouseButton::Right) => target.map(MouseAction::RightClick).unwrap_or(MouseAction::Click(ClickTarget::Pane("background".into()))),
            MouseEventKind::ScrollUp => MouseAction::Scroll { target, delta: -3 },
            MouseEventKind::ScrollDown => MouseAction::Scroll { target, delta: 3 },
            MouseEventKind::Drag(_) => MouseAction::Drag { target, x, y },
            MouseEventKind::Moved => MouseAction::Move { target, x, y },
            _ => MouseAction::Move { target, x, y },
        }
    }

    fn normalized_point(&self, x: u16, y: u16) -> (u16, u16) {
        // Some terminals report mouse row one line above Ratatui's rendered cell when
        // alternate-screen mouse capture is enabled. The observed SSHDeck behavior was
        // “click selects the row above the cursor”, so normalize all hit-testing one
        // row downward. Saturating arithmetic keeps top/bottom edges safe.
        let adjusted_y = if self.y_offset >= 0 {
            y.saturating_add(self.y_offset as u16)
        } else {
            y.saturating_sub((-self.y_offset) as u16)
        };
        (x, adjusted_y)
    }
}

fn contains(rect: Rect, x: u16, y: u16) -> bool { x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn region_hit_testing_uses_last_registered_topmost() {
        let mut r = RegionRegistry::default();
        r.register(Rect { x: 0, y: 0, width: 10, height: 10 }, ClickTarget::Pane("base".into()));
        r.register(Rect { x: 2, y: 2, width: 3, height: 3 }, ClickTarget::ModalButton("ok".into()));
        assert_eq!(r.hit(3,3), Some(ClickTarget::ModalButton("ok".into())));
        assert_eq!(r.hit(1,1), Some(ClickTarget::Pane("base".into())));
        assert_eq!(r.hit(20,20), None);
    }

    #[test]
    fn mouse_coordinates_are_shifted_down_to_match_rendered_rows() {
        let state = MouseState::default();
        assert_eq!(state.normalized_point(4, 9), (4, 10));
    }
}
