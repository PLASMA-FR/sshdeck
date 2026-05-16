use ratatui::{prelude::*, widgets::*};
use crate::{app::App, mouse::ClickTarget, widgets::{logo, status_bar}};

pub fn draw(f:&mut Frame, app:&mut App, area:Rect){
    let shell=crate::design::layout::app_shell(area);
    f.render_widget(logo::block(app), shell[0]);
    if app.hosts.is_empty(){ draw_empty(f,app,shell[1]); } else { let cols=crate::design::layout::dashboard(shell[1]); draw_nav(f,app,cols[0]); draw_hosts(f,app,cols[1]); draw_details(f,app,cols[2]); }
    status_bar::draw(f, app, shell[2], "/ search · a add · Enter connect · s files · t tunnel · ? help");
}

fn draw_empty(f:&mut Frame, app:&mut App, area:Rect){
    let r=crate::design::layout::centered(area,54,13); f.render_widget(Clear,r);
    let add=Rect{x:r.x+12,y:r.y+7,width:12,height:1}; let import=Rect{x:r.x+26,y:r.y+7,width:21,height:1};
    app.mouse.register(add, ClickTarget::ModalButton("add-host".into())); app.mouse.register(import, ClickTarget::ModalButton("import-hosts".into()));
    let text=vec![Line::from(Span::styled("No SSH hosts yet", app.theme.title())),Line::raw(""),Line::from("Add your first server or import from"),Line::from("your existing ~/.ssh/config."),Line::raw(""),Line::from(vec![Span::styled("[ Add Host ]", app.theme.accent()),Span::raw("  "),Span::styled("[ Import SSH Config ]", app.theme.accent())]),Line::raw(""),Line::from(Span::styled("Tip: new hosts are stored in", app.theme.muted())),Line::from(Span::styled("~/.config/sshdeck/ssh_config", app.theme.muted()))];
    f.render_widget(Paragraph::new(text).alignment(Alignment::Center).style(app.theme.surface()).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" SSHDeck ")), r);
}

fn draw_nav(f:&mut Frame, app:&mut App, area:Rect){
    let items=["All","Favorites","Production","Homelab","Recent","Tunnels","Commands","Logs"];
    let mut rows=Vec::new();
    for (i,id) in items.iter().enumerate(){ let y=area.y+2+i as u16; app.mouse.register(Rect{x:area.x+1,y,width:area.width-2,height:1}, ClickTarget::SidebarGroup((*id).into())); rows.push(ListItem::new(*id)); }
    f.render_widget(List::new(rows).style(app.theme.normal()).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.inactive_border()).title(" ")), area);
}

fn draw_hosts(f:&mut Frame, app:&mut App, area:Rect){
    let visible_height=area.height.saturating_sub(4) as usize;
    if app.selected < app.host_scroll { app.host_scroll=app.selected; }
    if app.selected >= app.host_scroll + visible_height { app.host_scroll=app.selected.saturating_sub(visible_height.saturating_sub(1)); }
    let add=Rect{x:area.x+area.width.saturating_sub(14),y:area.y,width:12,height:1}; app.mouse.register(add, ClickTarget::ModalButton("add-host".into()));
    let mut lines=vec![ListItem::new(Line::from(vec![Span::styled("Hosts", app.theme.title()), Span::raw("                         "), Span::styled("[+ Add Host]", app.theme.accent())]))];
    for (display_pos, host_idx) in app.filtered.iter().enumerate().skip(app.host_scroll).take(visible_height){
        if let Some(h)=app.hosts.get(*host_idx){
            let selected=display_pos==app.selected; let status=if app.managed_aliases.contains(&h.alias){"●"}else{"○"};
            let row=format!("{} {:<22} {}", status, h.alias, h.group.clone().unwrap_or_default());
            let y=area.y+2+(display_pos-app.host_scroll) as u16; app.mouse.register(Rect{x:area.x+1,y,width:area.width-2,height:1}, ClickTarget::HostRow(*host_idx));
            lines.push(ListItem::new(row).style(if selected{app.theme.selected()}else{app.theme.normal()}));
        }
    }
    let mut state=ListState::default(); state.select(Some(app.selected.saturating_sub(app.host_scroll)+1));
    let list=List::new(lines).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" Hosts ")).highlight_symbol(" ");
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_details(f:&mut Frame, app:&mut App, area:Rect){
    let Some(h)=app.current_host() else { return; };
    let alias=h.alias.clone(); let display=h.display_user_host(); let port=h.port_text(); let notes=h.notes.clone().unwrap_or_default(); let idx=app.current_host_index().unwrap_or(0);
    let buttons=[("Connect",ClickTarget::HostConnectButton(idx)),("Files",ClickTarget::HostFilesButton(idx)),("Tunnel",ClickTarget::HostTunnelButton(idx)),("Edit",ClickTarget::HostEditButton(idx))];
    let mut lines=vec![Line::from(Span::styled(alias.clone(), app.theme.title())),Line::raw(""),Line::from(display),Line::from(format!("port {}", port)),Line::raw(""),Line::from(Span::styled(if notes.is_empty(){"No notes".into()}else{notes}, app.theme.muted())),Line::raw("")];
    let mut spans=Vec::new(); for (i,(label,target)) in buttons.iter().enumerate(){ let x=area.x+2+(i as u16%2)*11; let y=area.y+8+(i as u16/2)*2; app.mouse.register(Rect{x,y,width:10,height:1}, target.clone()); spans.push(Span::styled(format!("[{}] ",label), app.theme.accent())); if i==1 { lines.push(Line::from(spans.clone())); spans.clear(); } } if !spans.is_empty(){lines.push(Line::from(spans));}
    lines.extend([Line::raw(""),Line::from(Span::styled("Recent",app.theme.muted())),Line::from("uptime · docker ps"),Line::raw(""),Line::from(Span::styled("Status cache",app.theme.muted())),Line::from("manual refresh with R")]);
    f.render_widget(Paragraph::new(lines).wrap(Wrap{trim:false}).style(app.theme.normal()).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.inactive_border()).title(" Details ")), area);
}
