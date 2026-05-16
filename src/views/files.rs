use ratatui::{prelude::*, widgets::*};
use crate::{app::App, mouse::ClickTarget, widgets::{button, status_bar, transfer_progress}};

pub fn draw(f:&mut Frame, app:&mut App, area:Rect){
    let shell=crate::design::layout::app_shell(area);
    let host=app.current_host().map(|h|h.alias.clone()).unwrap_or_else(||"no-host".into());
    let title=if app.files_dual_pane { format!("󰉋 Files · Transfer Mode") } else { format!("󰉋 Files · {}:{}", host, app.remote_path) };
    f.render_widget(Paragraph::new(format!("Path: {}                                      Mode: {:?}", app.remote_path, app.mode)).style(app.theme.title()).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(title)), shell[0]);
    if app.files_dual_pane { draw_dual(f,app,shell[1],&host); } else { draw_three(f,app,shell[1]); }
    if app.mode==crate::app::Mode::Transfer { let r=crate::design::layout::centered(area,58,10); f.render_widget(Clear,r); f.render_widget(transfer_progress::queue(app),r); }
    status_bar::draw(f, app, shell[2], "j/k move · h/l open · Space select · Tab dual-pane · right-click menu");
}

fn draw_three(f:&mut Frame, app:&mut App, area:Rect){
    let cols=crate::design::layout::files_three(area);
    app.mouse.register(cols[0], ClickTarget::Pane("parent".into())); app.mouse.register(cols[1], ClickTarget::Pane("files".into())); app.mouse.register(cols[2], ClickTarget::FilePreview);
    let parent=["󰉋 var","󰉋 www","󰉋 app"];
    let current=["󰉋 public","󰉋 src","󰉋 node_modules","󰈙 package.json        ◄","󰈙 README.md","󰈙 app.js","󰈙 .env              "];
    let mut parent_items=Vec::new();
    for (i,p) in parent.iter().enumerate(){ let target=ClickTarget::Breadcrumb(format!("/{p}")); app.mouse.register(Rect{x:cols[0].x+1,y:cols[0].y+2+i as u16,width:cols[0].width-2,height:1}, target.clone()); parent_items.push(ListItem::new(*p).style(button::row_style(app,&target,false))); }
    let mut cur_items=Vec::new();
    for (i,p) in current.iter().enumerate(){ let target=ClickTarget::FileEntry((*p).into()); app.mouse.register(Rect{x:cols[1].x+1,y:cols[1].y+2+i as u16,width:cols[1].width-2,height:1}, target.clone()); cur_items.push(ListItem::new(*p).style(button::row_style(app,&target,false))); }
    f.render_widget(List::new(parent_items).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.inactive_border()).title(" Parent ")), cols[0]);
    f.render_widget(List::new(cur_items).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" Current ")).highlight_symbol("◄ "), cols[1]);
    let preview=r#"package.json
JSON · 2.1 KB
modified today

{
  "scripts": {
    "start": "node app.js"
  }
}

Sensitive file protection:
.env and private keys show a lock and require confirmation before preview."#;
    let p=Paragraph::new(preview).scroll((app.preview_scroll as u16,0)).wrap(Wrap{trim:false}).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.inactive_border()).title(" Preview "));
    f.render_widget(p, cols[2]);
    let scrollbar=Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);
    let mut st=ScrollbarState::new(80).position(app.preview_scroll);
    f.render_stateful_widget(scrollbar, cols[2].inner(Margin{vertical:1,horizontal:0}), &mut st);
}

fn draw_dual(f:&mut Frame, app:&mut App, area:Rect, host:&str){
    let cols=crate::design::layout::files_dual(area);
    let local=["󰉋 screenshots","󰈙 backup.tar.gz            ◄","󰈙 notes.txt"];
    let remote=["󰉋 public","󰉋 src","󰈙 package.json"];
    let panels=[("LOCAL",local.as_slice(),0usize), ("REMOTE",remote.as_slice(),1usize)];
    for (idx,(title,items,pane)) in panels.iter().enumerate(){
        app.mouse.register(cols[idx], ClickTarget::Pane(title.to_lowercase()));
        let mut rows=Vec::new();
        for (i,it) in items.iter().enumerate(){ let target=ClickTarget::FileEntry((*it).into()); app.mouse.register(Rect{x:cols[idx].x+1,y:cols[idx].y+2+i as u16,width:cols[idx].width-2,height:1}, target.clone()); rows.push(ListItem::new(*it).style(button::row_style(app,&target,false))); }
        f.render_widget(List::new(rows).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(if app.active_file_pane==*pane{app.theme.border()}else{app.theme.inactive_border()}).title(format!(" {} · {} ",title, if *pane==0{app.local_path.clone()}else{format!("{}:{}",host,app.remote_path)}))), cols[idx]);
    }
}
