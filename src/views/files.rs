use ratatui::{prelude::*, widgets::*};
use crate::{app::App, files::{file_entry::{FileEntry, FileKind}, remote_fs}, mouse::ClickTarget, widgets::{button, status_bar, transfer_progress}};

pub fn draw(f:&mut Frame, app:&mut App, area:Rect){
    let shell=crate::design::layout::app_shell(area);
    let host=app.current_host().map(|h|h.alias.clone()).unwrap_or_else(||"no-host".into());
    let title=if app.files_dual_pane { "Files · local ↔ remote".to_string() } else { format!("Files · {}:{}", host, app.remote_path) };
    let subtitle = if let Some(err)=&app.remote_error { format!("Could not open this folder: {err}") } else { format!("{}:{} · home-first browsing. Uploads, deletes, and edits ask before touching anything.", host, app.remote_path) };
    f.render_widget(Paragraph::new(subtitle).style(if app.remote_error.is_some(){app.theme.error()}else{app.theme.title()}).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(title)), shell[0]);
    if app.files_dual_pane { draw_dual(f,app,shell[1],&host); } else { draw_three(f,app,shell[1]); }
    if app.mode==crate::app::Mode::Transfer { let r=crate::design::layout::centered(area,58,10); f.render_widget(Clear,r); f.render_widget(transfer_progress::queue(app),r); }
    status_bar::draw(f, app, shell[2], "j/k move · h parent · l/Enter open · ~ home · R refresh · Tab dual-pane");
}

fn draw_three(f:&mut Frame, app:&mut App, area:Rect){
    let cols=crate::design::layout::files_three(area);
    app.mouse.register(cols[0], ClickTarget::Pane("parent".into())); app.mouse.register(cols[1], ClickTarget::Pane("files".into())); app.mouse.register(cols[2], ClickTarget::FilePreview);

    let parent_path = remote_fs::parent_remote_path(&app.remote_path);
    let parent_rows = parent_rows(&app.remote_path, &parent_path);
    let mut parent_items=Vec::new();
    for (i,p) in parent_rows.iter().enumerate(){ let target=ClickTarget::Breadcrumb(p.1.clone()); app.mouse.register(Rect{x:cols[0].x+1,y:cols[0].y+1+i as u16,width:cols[0].width.saturating_sub(2),height:1}, target.clone()); parent_items.push(ListItem::new(p.0.clone()).style(button::row_style(app,&target,false))); }

    let mut cur_items=Vec::new();
    if let Some(err)=&app.remote_error {
        cur_items.push(ListItem::new(format!("Could not open remote files: {err}" )).style(app.theme.error()));
        cur_items.push(ListItem::new("R retry · h parent · ~ home · Esc back").style(app.theme.muted()));
    } else if app.remote_loading {
        cur_items.push(ListItem::new(format!("Opening {} {}", app.remote_path, app.animator.flow())).style(app.theme.muted()));
    } else if app.remote_entries.is_empty() {
        cur_items.push(ListItem::new("This directory is empty.").style(app.theme.muted()));
        cur_items.push(ListItem::new("n create · u upload · h parent").style(app.theme.muted()));
    } else {
        let visible_rows = cols[1].height.saturating_sub(2) as usize;
        keep_file_selection_visible(app, visible_rows);
        for (i,e) in app.remote_entries.iter().enumerate().skip(app.file_scroll).take(visible_rows){
            let target=ClickTarget::FileEntry(e.path.clone());
            let y = cols[1].y+1+(i-app.file_scroll) as u16;
            app.mouse.register(Rect{x:cols[1].x+1,y,width:cols[1].width.saturating_sub(2),height:1}, target.clone());
            let selected = app.selected_file_paths.iter().any(|p| p == &e.path);
            let marker = if i==app.file_selected { "◄" } else if selected { "*" } else { " " };
            cur_items.push(ListItem::new(format_entry(app, e, marker)).style(if i==app.file_selected{app.theme.selected()}else{button::row_style(app,&target,false)}));
        }
    }

    f.render_widget(List::new(parent_items).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.inactive_border()).title(" Parent ")), cols[0]);
    f.render_widget(List::new(cur_items).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.border()).title(" Current ")).highlight_symbol("◄ "), cols[1]);
    let preview=preview_text(app);
    let p=Paragraph::new(preview).scroll((app.preview_scroll as u16,0)).wrap(Wrap{trim:false}).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(app.theme.inactive_border()).title(" Preview "));
    f.render_widget(p, cols[2]);
    let scrollbar=Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);
    let mut st=ScrollbarState::new(80).position(app.preview_scroll);
    f.render_stateful_widget(scrollbar, cols[2].inner(Margin{vertical:1,horizontal:0}), &mut st);
}

fn draw_dual(f:&mut Frame, app:&mut App, area:Rect, host:&str){
    let cols=crate::design::layout::files_dual(area);
    keep_file_selection_visible(app, cols[1].height.saturating_sub(2) as usize);
    keep_local_selection_visible(app, cols[0].height.saturating_sub(2) as usize);
    let local: Vec<String> = if let Some(err)=&app.local_error {
        vec![format!("Could not open local folder: {err}")]
    } else if app.local_entries.is_empty() {
        vec!["This local folder is empty".into()]
    } else {
        app.local_entries.iter().enumerate().skip(app.local_scroll).take(cols[0].height.saturating_sub(2) as usize).map(|(i,e)| {
            let marker = if i == app.local_selected { "◄" } else { " " };
            format_entry(app, e, marker)
        }).collect()
    };
    let remote:Vec<String> = if app.remote_entries.is_empty() {
        vec![app.remote_error.clone().unwrap_or_else(|| if app.remote_loading {"Opening remote folder".into()} else {"This directory is empty".into()})]
    } else {
        app.remote_entries.iter().enumerate().skip(app.file_scroll).take(cols[1].height.saturating_sub(2) as usize).map(|(i,e)| {
            let marker = if i == app.file_selected { "◄" } else { "" };
            format_entry(app, e, marker)
        }).collect()
    };
    let panels=[("LOCAL",local,0usize), ("REMOTE",remote,1usize)];
    for (idx,(title,items,pane)) in panels.iter().enumerate(){
        app.mouse.register(cols[idx], ClickTarget::Pane(title.to_lowercase()));
        let mut rows=Vec::new();
        for (i,it) in items.iter().enumerate(){
            let target=if *pane==0 {
                app.local_entries.get(app.local_scroll+i).map(|e|ClickTarget::FileEntry(e.path.clone())).unwrap_or_else(||ClickTarget::Pane("local".into()))
            } else {
                app.remote_entries.get(app.file_scroll+i).map(|e|ClickTarget::FileEntry(e.path.clone())).unwrap_or_else(||ClickTarget::Pane("remote".into()))
            };
            app.mouse.register(Rect{x:cols[idx].x+1,y:cols[idx].y+1+i as u16,width:cols[idx].width.saturating_sub(2),height:1}, target.clone());
            let selected = (*pane==0 && app.active_file_pane==0 && app.local_scroll+i==app.local_selected) || (*pane==1 && app.active_file_pane==1 && app.file_scroll+i==app.file_selected);
            rows.push(ListItem::new(it.clone()).style(if selected { app.theme.selected() } else { button::row_style(app,&target,false) }));
        }
        f.render_widget(List::new(rows).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).border_style(if app.active_file_pane==*pane{app.theme.border()}else{app.theme.inactive_border()}).title(format!(" {} · {} ",title, if *pane==0{app.local_path.clone()}else{format!("{}:{}",host,app.remote_path)}))), cols[idx]);
    }
}

fn parent_rows(path:&str, parent:&str)->Vec<(String,String)> {
    let mut rows = vec![("..".to_string(), parent.to_string()), ("~ home".to_string(), "~".to_string()), ("/ root".to_string(), "/".to_string())];
    if path.starts_with("~/") { rows.push((parent.to_string(), parent.to_string())); }
    rows
}

fn format_entry(app:&App, e:&FileEntry, marker:&str)->String {
    let icon = e.icon(!app.ascii && app.config.ui.nerd_font);
    let lock = if e.permissions.starts_with("drwx") || e.permissions.starts_with("-rw") { "" } else { " " };
    format!("{icon} {:<28} {:>8} {} {marker}{lock}", e.name, human_size(e.size), e.modified)
}

fn preview_text(app:&App)->String {
    if let Some(preview)=&app.remote_preview {
        return preview.clone();
    }
    if let Some(err)=&app.remote_error {
        return format!("Could not open this folder.\n\nHost: {}\nPath: {}\nReason: {err}\n\nR retry · h parent · ~ home · Esc back", app.current_host().map(|h|h.alias.as_str()).unwrap_or("no-host"), app.remote_path);
    }
    let Some(entry)=app.remote_entries.get(app.file_selected) else {
        return if app.remote_loading { "Opening remote folder...".into() } else { "This directory is empty.\n\nn create · u upload · h parent".into() };
    };
    let kind = match entry.kind { FileKind::Directory=>"Directory", FileKind::Symlink=>"Symlink", FileKind::Executable=>"Executable", FileKind::Archive=>"Archive", FileKind::Image=>"Image", FileKind::File=>"File", FileKind::Other=>"Other" };
    format!("{}\n{} · {}\n{} {}\nowner: {}:{}\npath: {}\n\n{}", entry.name, kind, human_size(entry.size), entry.permissions, entry.modified, entry.owner, entry.group, entry.path, if matches!(entry.kind, FileKind::Directory){"Enter/l opens this directory."}else{"Preview/download/edit actions are guarded for sensitive files."})
}

fn keep_file_selection_visible(app:&mut App, visible_rows:usize){
    if visible_rows == 0 { return; }
    if app.file_selected < app.file_scroll { app.file_scroll = app.file_selected; }
    if app.file_selected >= app.file_scroll + visible_rows {
        app.file_scroll = app.file_selected.saturating_sub(visible_rows.saturating_sub(1));
    }
}

fn keep_local_selection_visible(app:&mut App, visible_rows:usize){
    if visible_rows == 0 { return; }
    if app.local_selected < app.local_scroll { app.local_scroll = app.local_selected; }
    if app.local_selected >= app.local_scroll + visible_rows {
        app.local_scroll = app.local_selected.saturating_sub(visible_rows.saturating_sub(1));
    }
}

fn human_size(size:u64)->String{
    const UNITS:&[&str]=&["B","KB","MB","GB","TB"];
    let mut value=size as f64; let mut unit=0;
    while value>=1024.0 && unit+1<UNITS.len(){ value/=1024.0; unit+=1; }
    if unit==0 { format!("{} {}", size, UNITS[unit]) } else { format!("{value:.1} {}", UNITS[unit]) }
}
