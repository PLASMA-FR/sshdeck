use ratatui::style::{Modifier,Style}; use crate::theme::Theme;
pub fn hero(t:&Theme)->Style{t.title().add_modifier(Modifier::BOLD)} pub fn label(t:&Theme)->Style{t.muted().add_modifier(Modifier::BOLD)} pub fn badge(t:&Theme)->Style{t.selected()} pub fn hint(t:&Theme)->Style{t.muted()}
