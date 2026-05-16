use std::rc::Rc;
use ratatui::prelude::*;
pub const MIN_WIDTH:u16=100; pub const MIN_HEIGHT:u16=30;
pub fn app_shell(area:Rect)->Rc<[Rect]>{ Layout::vertical([Constraint::Length(3),Constraint::Min(8),Constraint::Length(1)]).split(area) }
pub fn dashboard(area:Rect)->Rc<[Rect]>{ Layout::horizontal([Constraint::Length(18),Constraint::Percentage(44),Constraint::Percentage(56)]).split(area) }
pub fn files_three(area:Rect)->Rc<[Rect]>{ Layout::horizontal([Constraint::Percentage(25),Constraint::Percentage(40),Constraint::Percentage(35)]).split(area) }
pub fn files_dual(area:Rect)->Rc<[Rect]>{ Layout::horizontal([Constraint::Percentage(50),Constraint::Percentage(50)]).split(area) }
pub fn centered(area:Rect, w:u16, h:u16)->Rect{ let w=w.min(area.width.saturating_sub(4)); let h=h.min(area.height.saturating_sub(2)); Rect{x:area.x+(area.width.saturating_sub(w))/2,y:area.y+(area.height.saturating_sub(h))/2,width:w,height:h} }
