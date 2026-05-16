use ratatui::{prelude::*, widgets::*};
use crate::{app::{App, HostFormState}, config::managed_hosts::HostValidationLevel, mouse::ClickTarget};

fn centered(area:Rect, w:u16, h:u16)->Rect{ crate::design::layout::centered(area,w,h) }

pub fn command_palette(f:&mut Frame, app:&mut App, area:Rect){
    let r=centered(area,56,15); f.render_widget(Clear,r);
    let actions=["Add Host","Open SSHDeck Files","Create Tunnel","Run Remote Command","Fetch Health","Copy SSH Command","Duplicate Host","Toggle Theme","Quit"];
    let mut lines=vec![Line::from(vec![Span::styled("> ",app.theme.muted()),Span::styled(app.palette_input.clone(),app.theme.title())]), Line::raw("")];
    for (i,a) in actions.iter().enumerate(){ let y=r.y+3+i as u16; if y<r.y+r.height-1 { app.mouse.register(Rect{x:r.x+1,y,width:r.width-2,height:1}, ClickTarget::CommandPaletteItem((*a).into())); } lines.push(Line::from((*a).to_string())); }
    f.render_widget(Paragraph::new(lines).style(app.theme.overlay()).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.active_border()).title(" Command Palette ")),r);
}

pub fn search(f:&mut Frame, app:&mut App, area:Rect){ let r=centered(area,50,5); f.render_widget(Clear,r); f.render_widget(Paragraph::new(format!("/{}\n{} matches",app.search, app.filtered.len())).style(app.theme.overlay()).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.active_border()).title(" Search ")),r); }
pub fn command_mode(f:&mut Frame, app:&mut App, area:Rect){ let r=Rect{x:area.x,y:area.y+area.height.saturating_sub(4),width:area.width,height:3}; f.render_widget(Clear,r); f.render_widget(Paragraph::new(format!(":{}",app.command_input)).style(app.theme.overlay()).block(Block::bordered().border_style(app.theme.active_border()).title(" Command ")),r); }

pub fn host_form(f:&mut Frame, app:&mut App, area:Rect){
    let Some(form)=app.host_form.clone() else { return; };
    let r=centered(area,58,21); f.render_widget(Clear,r);
    let fields=[("Alias",form.draft.alias.clone()),("Hostname/IP",form.draft.hostname.clone()),("User",form.draft.user.clone()),("Port",form.draft.port.clone()),("Identity File",form.draft.identity_file.clone()),("Group",form.draft.group.clone()),("Tags",form.draft.tags.clone()),("Notes",form.draft.notes.clone())];
    let mut lines=Vec::new();
    for (i,(label,value)) in fields.iter().enumerate(){
        let y=r.y+2+i as u16; app.mouse.register(Rect{x:r.x+16,y,width:r.width.saturating_sub(20),height:1}, ClickTarget::FormField(label.to_ascii_lowercase().replace(' ', "-")));
        let marker=if form.field==i {"›"} else {" "};
        lines.push(Line::from(vec![Span::styled(format!("{marker} {label:<13}"), if form.field==i{app.theme.accent()}else{app.theme.muted()}), Span::styled(format!("[ {:<31} ]", truncate(value,31)), if form.field==i{app.theme.selected()}else{app.theme.normal()})]));
    }
    lines.push(Line::raw(""));
    let buttons=[("Test","test-host",8usize),("Save","save-host",9usize),("Cancel","cancel",10usize)];
    let mut btn_spans=vec![Span::raw("              ")];
    for (label,id,idx) in buttons { let x=r.x+16+(idx as u16-8)*9; let y=r.y+11; app.mouse.register(Rect{x,y,width:8,height:1},ClickTarget::ModalButton(id.into())); btn_spans.push(Span::styled(format!("[ {label} ] "), if form.field==idx{app.theme.selected()}else{app.theme.accent()})); }
    lines.push(Line::from(btn_spans));
    if let Some(result)=&form.test_result { lines.push(Line::raw("")); lines.push(Line::from(result.clone())); }
    for m in form.messages.iter().take(4){ let style=if m.level==HostValidationLevel::Error{app.theme.error()}else{app.theme.warning()}; lines.push(Line::from(Span::styled(format!("{} {}", if m.level==HostValidationLevel::Error{"✗"}else{"⚠"}, m.message), style))); }
    lines.push(Line::raw("")); lines.push(Line::from(Span::styled("Tab/Shift+Tab fields · Ctrl+s save · Esc cancel · mouse supported", app.theme.muted())));
    f.render_widget(Paragraph::new(lines).style(app.theme.overlay()).wrap(Wrap{trim:false}).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.active_border()).title(format!(" {} ", form.title()))), r);
}

pub fn context_menu(f:&mut Frame, app:&mut App, area:Rect){
    let Some(menu)=app.context_menu.clone() else { return; };
    let w=menu.items.iter().map(|(s,_)|s.len() as u16).max().unwrap_or(16).saturating_add(4).min(54);
    let h=(menu.items.len() as u16 + 2).min(area.height.saturating_sub(2));
    let r=centered(area,w,h); f.render_widget(Clear,r);
    let mut rows=Vec::new();
    for (i,(label,target)) in menu.items.iter().enumerate(){ let y=r.y+1+i as u16; app.mouse.register(Rect{x:r.x+1,y,width:r.width-2,height:1}, target.clone()); rows.push(ListItem::new(label.clone())); }
    f.render_widget(List::new(rows).style(app.theme.overlay()).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.active_border()).title(format!(" {} ",menu.title))).highlight_style(app.theme.selected()),r);
}

fn truncate(s:&str, max:usize)->String{ if s.chars().count()<=max {s.into()} else { let mut out=s.chars().take(max.saturating_sub(1)).collect::<String>(); out.push('…'); out } }
