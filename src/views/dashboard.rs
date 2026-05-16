use ratatui::{prelude::*, widgets::*};
use crate::{app::App, mouse::ClickTarget, widgets::{logo, status_bar}};

pub fn draw(f:&mut Frame, app:&mut App, area:Rect){
    let shell=crate::design::layout::app_shell(area);
    f.render_widget(logo::block(app), shell[0]);
    let cols=crate::design::layout::dashboard(shell[1]);
    draw_nav(f,app,cols[0]); draw_hosts(f,app,cols[1]); draw_command_center(f,app,cols[2]);
    status_bar::draw(f, app, shell[2], "/ search │ Ctrl+p palette │ ? help");
}

fn draw_nav(f:&mut Frame, app:&mut App, area:Rect){
    let ic=app.icons();
    let items=[(format!("{} All",ic.all),"All"),(format!("{} Favorites",ic.favorite),"Favorites"),("󰞷 Production".into(),"Production"),(format!("{} Homelab",ic.files),"Homelab"),(format!("{} Tunnels",ic.tunnel),"Tunnels"),(format!("{} Commands",ic.terminal),"Commands"),(format!("{} Logs",ic.logs),"Logs")];
    let mut rows=Vec::new();
    for (i,(label,id)) in items.iter().enumerate(){ let y=area.y+2+i as u16; app.mouse.register(Rect{x:area.x+1,y,width:area.width-2,height:1}, ClickTarget::SidebarGroup((*id).into())); rows.push(ListItem::new(label.clone())); }
    f.render_widget(List::new(rows).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.inactive_border()).title(" NAVIGATION ")).style(app.theme.normal()), area);
}

fn draw_hosts(f:&mut Frame, app:&mut App, area:Rect){
    if app.hosts.is_empty(){
        let empty="No SSH hosts found.\n\nOptions:\n  a  Add your first host\n  i  Import from ~/.ssh/config\n  ?  Help\n\nMouse: click Add in the command palette.";
        f.render_widget(Paragraph::new(empty).alignment(Alignment::Center).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" HOSTS ")), area); return;
    }
    let visible_height=area.height.saturating_sub(2) as usize;
    if app.selected < app.host_scroll { app.host_scroll=app.selected; }
    if app.selected >= app.host_scroll + visible_height { app.host_scroll=app.selected.saturating_sub(visible_height.saturating_sub(1)); }
    let mut lines=Vec::new();
    for (display_pos, host_idx) in app.filtered.iter().enumerate().skip(app.host_scroll).take(visible_height){
        if let Some(h)=app.hosts.get(*host_idx){
            let selected=display_pos==app.selected; let status=if display_pos%2==0{"● online"}else{"○ unknown"};
            let tags=if h.tags.is_empty(){ h.group.clone().unwrap_or_else(||"local · openssh".into()) } else { h.tags.join(" · ") };
            let row=format!("{} {:<18} {:>9}\n  {} :{}\n  {}", app.icons().server, h.alias, status, h.display_user_host(), h.port_text(), tags);
            let y=area.y+1+(display_pos-app.host_scroll) as u16;
            app.mouse.register(Rect{x:area.x+1,y,width:area.width-2,height:3.min(area.height.saturating_sub(1))}, ClickTarget::HostRow(*host_idx));
            lines.push(ListItem::new(row).style(if selected{app.theme.selected()}else{app.theme.normal()}));
        }
    }
    let mut state=ListState::default(); state.select(Some(app.selected.saturating_sub(app.host_scroll)));
    let list=List::new(lines).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" HOSTS ")).highlight_symbol("◄ ");
    f.render_stateful_widget(list, area, &mut state);
    let scrollbar=Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight).begin_symbol(Some("↑")).end_symbol(Some("↓"));
    let mut st=ScrollbarState::new(app.filtered.len()).position(app.host_scroll);
    f.render_stateful_widget(scrollbar, area.inner(Margin{vertical:1,horizontal:0}), &mut st);
}

fn draw_command_center(f:&mut Frame, app:&mut App, area:Rect){
    let Some(h0)=app.current_host() else { f.render_widget(Paragraph::new("Select a host to unlock the command center").block(Block::bordered().title(" COMMAND CENTER ")), area); return; };
    let alias = h0.alias.clone();
    let display = h0.display_user_host();
    let port = h0.port_text();
    let idx=app.current_host_index().unwrap_or(0); let ic=app.icons();
    let buttons=[("Connect",ClickTarget::HostConnectButton(idx)),("Files",ClickTarget::HostFilesButton(idx)),("Tunnel",ClickTarget::HostTunnelButton(idx)),("Run",ClickTarget::StatusShortcut("run".into())),("Health",ClickTarget::HostHealthButton(idx)),("Edit",ClickTarget::HostEditButton(idx))];
    let mut text=vec![Line::from(vec![Span::styled(alias.clone(), app.theme.title())]), Line::from(format!("{} {} :{}", ic.terminal, display, port)), Line::from("● online · Ubuntu · OpenSSH"), Line::raw("")];
    let mut btnline=Vec::new();
    for (i,(label,target)) in buttons.iter().enumerate(){ let x=area.x+2+(i as u16%3)*11; let y=area.y+6+(i as u16/3)*2; app.mouse.register(Rect{x,y,width:10,height:1}, target.clone()); btnline.push(Span::styled(format!("[{}] ",label), app.theme.title())); if i==2{text.push(Line::from(btnline.clone())); btnline.clear();} }
    if !btnline.is_empty(){text.push(Line::from(btnline));}
    text.extend([Line::raw(""),Line::from("CPU  ▰▰▰▱▱ 42%"),Line::from("RAM  ▰▰▱▱▱ 31%"),Line::from("Disk ▰▰▰▰▱ 78%"),Line::raw(""),Line::from("Recent commands"),Line::from("  uptime · docker ps · df -h"),Line::from("Active tunnels"),Line::from(format!("  localhost:8080 {} {}:80", app.animator.flow(), alias)),Line::from("Recent paths"),Line::from("  /var/www/app · /var/log")]);
    f.render_widget(Paragraph::new(text).wrap(Wrap{trim:false}).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" COMMAND CENTER ")), area);
}
