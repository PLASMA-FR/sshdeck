use ratatui::{prelude::*, widgets::*};
use crate::{app::App, mouse::ClickTarget, widgets::{button::{self, ButtonKind}, status_bar}};

pub fn draw(f:&mut Frame, app:&mut App, area:Rect){
    let chunks=Layout::vertical([Constraint::Min(5),Constraint::Length(1)]).split(area);
    let type_rows=[("Local Forward","local"),("Remote Forward","remote"),("Dynamic SOCKS","dynamic")];
    let mut lines=Vec::new();
    for (i,(label,id)) in type_rows.iter().enumerate(){
        let target=ClickTarget::TunnelType((*id).into());
        let y=chunks[0].y+1+i as u16;
        app.mouse.register(Rect{x:chunks[0].x+2,y,width:28,height:1}, target.clone());
        let active=format!("{:?}",app.tunnel.tunnel_type).to_lowercase()==*id;
        let marker=if active {"●"} else {"○"};
        lines.push(Line::from(vec![Span::raw(format!("{} ", marker)), button::label(app,label,&target,if active{ButtonKind::Primary}else{ButtonKind::Secondary})]));
    }
    let cmd=app.tunnel.command();
    lines.extend([Line::raw(""),Line::from(format!("Local: 127.0.0.1:{}",app.tunnel.local_port)),Line::from(format!("Target: {}:{}",app.tunnel.target_host.clone().unwrap_or_default(),app.tunnel.target_port.unwrap_or(80))),Line::from(format!("Host: {}",app.tunnel.host_alias)),Line::raw(""),Line::from(format!("Command: {}",cmd)),Line::raw(""),Line::from(format!("localhost:{} {} {}:80",app.tunnel.local_port,app.animator.flow(),app.tunnel.host_alias)),Line::raw("")]);
    let start=ClickTarget::ModalButton("start-tunnel".into()); let cancel=ClickTarget::ModalButton("cancel".into());
    let y=chunks[0].y+13;
    app.mouse.register(Rect{x:chunks[0].x+1,y,width:12,height:1},start.clone());
    app.mouse.register(Rect{x:chunks[0].x+15,y,width:12,height:1},cancel.clone());
    lines.push(Line::from(vec![button::label(app,"Start",&start,ButtonKind::Primary),Span::raw("  "),button::label(app,"Cancel",&cancel,ButtonKind::Secondary)]));
    f.render_widget(Paragraph::new(lines).wrap(Wrap{trim:false}).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" Tunnel Builder ")), chunks[0]);
    status_bar::draw(f, app, chunks[1], "click tunnel type · Enter start · Esc back");
}
