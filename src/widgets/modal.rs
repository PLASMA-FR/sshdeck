use ratatui::{prelude::*, widgets::*};
use crate::{app::App, mouse::ClickTarget};

fn centered(area:Rect, w:u16, h:u16)->Rect{ crate::design::layout::centered(area,w,h) }

pub fn command_palette(f:&mut Frame, app:&mut App, area:Rect){
    let r=centered(area,56,13); f.render_widget(Clear,r);
    let actions=["󰉋 Open SSHDeck Files","󰩠 Create Tunnel"," Run Remote Command","󰋊 Fetch Health","󰈞 Copy SSH Command","󰗼 Open Logs","Quit"];
    let mut lines=vec![Line::from(vec![Span::styled("> ",app.theme.muted()),Span::styled(app.palette_input.clone(),app.theme.title())])];
    lines.push(Line::raw(""));
    for (i,a) in actions.iter().enumerate(){
        let y=r.y+3+i as u16; if y<r.y+r.height-1 { app.mouse.register(Rect{x:r.x+1,y,width:r.width-2,height:1}, ClickTarget::CommandPaletteItem((*a).into())); }
        lines.push(Line::from((*a).to_string()));
    }
    f.render_widget(Paragraph::new(lines).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" Command Palette ")),r);
}

pub fn search(f:&mut Frame, app:&mut App, area:Rect){ let r=centered(area,50,5); f.render_widget(Clear,r); f.render_widget(Paragraph::new(format!("/{}\n{} matches",app.search, app.filtered.len())).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" Fuzzy Search ")),r); }
pub fn command_mode(f:&mut Frame, app:&mut App, area:Rect){ let r=Rect{x:area.x,y:area.y+area.height.saturating_sub(4),width:area.width,height:3}; f.render_widget(Clear,r); f.render_widget(Paragraph::new(format!(":{}",app.command_input)).block(Block::bordered().border_style(app.theme.border()).title(" Command Mode ")),r); }

pub fn context_menu(f:&mut Frame, app:&mut App, area:Rect){
    let Some(menu)=app.context_menu.clone() else { return; };
    let w=menu.items.iter().map(|(s,_)|s.len() as u16).max().unwrap_or(16).saturating_add(4).min(54);
    let h=(menu.items.len() as u16 + 2).min(area.height.saturating_sub(2));
    let r=centered(area,w,h); f.render_widget(Clear,r);
    let mut rows=Vec::new();
    for (i,(label,target)) in menu.items.iter().enumerate(){
        let y=r.y+1+i as u16; app.mouse.register(Rect{x:r.x+1,y,width:r.width-2,height:1}, target.clone()); rows.push(ListItem::new(label.clone()));
    }
    f.render_widget(List::new(rows).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(format!(" {} ",menu.title))).highlight_style(app.theme.selected()),r);
}
