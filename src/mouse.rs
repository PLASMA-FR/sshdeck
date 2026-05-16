use std::time::{Duration, Instant};

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;

use crate::app::View;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickTarget {
    SidebarItem(String),
    SidebarGroup(String),
    HostRow(usize),
    HostActionButton { host_index: usize, action: String },
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
    AddHostButton,
    ContextMenuItem(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseButtonAction {
    Click,
    DoubleClick,
    RightClick,
    Drag,
    Hover,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollTarget {
    Hosts,
    Files,
    Preview,
    Transfers,
    Logs,
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
    pub fn regions(&self) -> &[Region] { &self.regions }
}

#[derive(Debug, Clone)]
pub struct MouseState { pub registry: RegionRegistry, last_click: Option<(ClickTarget, Instant)> }
impl Default for MouseState { fn default() -> Self { Self { registry: RegionRegistry::default(), last_click: None } } }

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
        // Crossterm and Ratatui both use zero-based terminal cell coordinates.
        // Keep this as a single normalization point so terminal-specific quirks
        // can be handled deliberately later without hiding per-widget row bugs.
        (x, y)
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
    fn mouse_coordinates_match_ratatui_zero_based_cells_without_global_offset() {
        let state = MouseState::default();
        assert_eq!(state.normalized_point(4, 9), (4, 9));
    }
}
