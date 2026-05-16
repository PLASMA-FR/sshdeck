use std::{process::Command, time::Duration};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers, MouseEvent},
    terminal::{Clear, ClearType},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{backend::Backend, Terminal};

use crate::{
    animation::Animator,
    config::{app_config::{AppConfig, HostMeta}, managed_hosts::{self, HostDraft, HostValidationLevel, HostValidationMessage}, ssh_config::parse_default_ssh_config, storage},
    event::{Event, EventLoop},
    files::transfer::TransferQueue,
    mouse::{ClickTarget, MouseAction, MouseState},
    ssh::{
        command::{display_command, is_dangerous_command, ssh_args_for, ssh_test_args_for},
        health::HealthInfo,
        host::SshHost,
        tunnel::{TunnelConfig, TunnelType},
    },
    theme::Theme,
    ui,
};

#[derive(Debug, Clone, Copy)]
pub struct AppOptions { pub no_animations: bool, pub ascii: bool, pub mouse: bool }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View { Dashboard, HostDetail, Files, Tunnels, CommandRunner, Logs, Settings, Help }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode { Normal, Search, Visual, Command, Rename, Confirm, Transfer, Palette, HostForm }

#[derive(Debug, Clone)]
pub struct Toast { pub message:String, pub ttl:u8, pub level: ToastLevel }
#[derive(Debug, Clone)]
pub enum ToastLevel { Success, Warning, Error, Info }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostFormMode { Add, Edit, Duplicate }

#[derive(Debug, Clone)]
pub struct HostFormState {
    pub mode: HostFormMode,
    pub draft: HostDraft,
    pub field: usize,
    pub messages: Vec<HostValidationMessage>,
    pub test_result: Option<String>,
    pub original_alias: Option<String>,
}

impl HostFormState {
    pub fn title(&self) -> &'static str { match self.mode { HostFormMode::Add => "Add Host", HostFormMode::Edit => "Edit Host", HostFormMode::Duplicate => "Duplicate Host" } }
    pub fn field_count(&self) -> usize { 11 }
}

#[derive(Debug, Clone)]
pub struct ContextMenu { pub title:String, pub items:Vec<(String, ClickTarget)> }

pub struct App {
    pub config: AppConfig,
    pub hosts: Vec<SshHost>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub host_scroll: usize,
    pub file_scroll: usize,
    pub preview_scroll: usize,
    pub view: View,
    pub mode: Mode,
    pub search: String,
    pub command_input: String,
    pub palette_input: String,
    pub logs: Vec<String>,
    pub theme: Theme,
    pub animator: Animator,
    pub ascii: bool,
    pub mouse_enabled: bool,
    pub mouse: MouseState,
    pub focused_pane: String,
    pub hover_target: Option<ClickTarget>,
    pub context_menu: Option<ContextMenu>,
    pub should_quit: bool,
    pub toast: Option<Toast>,
    pub health: HealthInfo,
    pub tunnel: TunnelConfig,
    pub remote_path: String,
    pub local_path: String,
    pub files_dual_pane: bool,
    pub active_file_pane: usize,
    pub selected_files: usize,
    pub transfer_queue: TransferQueue,
    pub command_output: String,
    pub managed_aliases: Vec<String>,
    pub host_form: Option<HostFormState>,
    pub hide_aliases: Vec<String>,
    pub render_reset_needed: bool,
}

impl App {
    pub fn new(config: AppConfig, options: AppOptions) -> Result<Self> {
        let mut hosts = parse_default_ssh_config().unwrap_or_default();
        let managed_path = managed_hosts::managed_config_path();
        let managed_hosts_loaded = managed_hosts::read_managed_hosts(&managed_path).unwrap_or_default();
        let managed_aliases: Vec<String> = managed_hosts_loaded.iter().map(|h| h.alias.clone()).collect();
        hosts.extend(managed_hosts_loaded);
        for h in &mut hosts {
            if let Some(meta) = config.hosts.get(&h.alias) {
                h.tags = meta.tags.clone(); h.group = meta.group.clone(); h.favorite = meta.favorite; h.notes = meta.notes.clone();
            }
        }
        let theme = Theme::named(&config.ui.theme);
        let mut app = Self {
            filtered: (0..hosts.len()).collect(), hosts, selected: 0, host_scroll: 0, file_scroll: 0, preview_scroll: 0,
            view: View::Dashboard, mode: Mode::Normal, search: String::new(), command_input: String::new(), palette_input: String::new(),
            logs: storage::read_logs(), theme, animator: Animator::new(config.ui.animations && !options.no_animations),
            ascii: options.ascii || !config.ui.unicode, mouse_enabled: options.mouse, mouse: MouseState::default(), focused_pane: "hosts".into(), hover_target: None, context_menu: None,
            should_quit: false, toast: None, health: HealthInfo::empty(),
            tunnel: TunnelConfig { tunnel_type: TunnelType::Local, host_alias: String::new(), bind_address: None, local_port: 8080, target_host: Some("localhost".into()), target_port: Some(80) },
            remote_path: "~".into(), local_path: config.files.default_local_dir.clone(), files_dual_pane: false, active_file_pane: 1, selected_files: 0,
            transfer_queue: TransferQueue::default(), command_output: String::new(), managed_aliases, host_form: None, hide_aliases: Vec::new(), render_reset_needed: false, config,
        };
        app.toast(ToastLevel::Success, format!("Found {} host(s). Your SSH config stays untouched.", app.hosts.len()));
        Ok(app)
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut events = EventLoop::new(Duration::from_millis(80));
        while !self.should_quit {
            if self.render_reset_needed {
                terminal.clear()?;
                self.render_reset_needed = false;
            }
            terminal.draw(|f| ui::draw(f, self))?;
            match events.next()? {
                Event::Tick => self.on_tick(),
                Event::Resize(_, _) => {},
                Event::Key(k) => self.handle_key(k)?,
                Event::Mouse(m) if self.mouse_enabled => self.handle_mouse(m)?,
                Event::Mouse(_) => {},
            }
        }
        Ok(())
    }

    pub fn current_host(&self) -> Option<&SshHost> { self.filtered.get(self.selected).and_then(|i| self.hosts.get(*i)) }
    pub fn current_host_index(&self) -> Option<usize> { self.filtered.get(self.selected).copied() }
    pub fn on_tick(&mut self) { self.animator.tick(); if let Some(t)=self.toast.as_mut(){ if t.ttl>0 { t.ttl-=1; } else { self.toast=None; } } }
    pub fn toast(&mut self, level:ToastLevel, message:String) { storage::append_log(&message); self.logs.push(message.clone()); self.toast=Some(Toast{message,ttl:40,level}); }
    pub fn icons(&self) -> crate::design::icons::Icons { if self.ascii { crate::design::icons::ascii() } else { crate::design::icons::nerd() } }
    pub fn is_hovered(&self, target: &ClickTarget) -> bool { self.hover_target.as_ref().is_some_and(|h| h == target) }

    fn filter_hosts(&mut self) {
        if self.search.is_empty() { self.filtered = (0..self.hosts.len()).collect(); }
        else {
            let m = SkimMatcherV2::default();
            let mut scored: Vec<(i64, usize)> = self.hosts.iter().enumerate().filter_map(|(i,h)| m.fuzzy_match(&h.search_blob(), &self.search).map(|s|(s,i))).collect();
            scored.sort_by(|a,b| b.0.cmp(&a.0)); self.filtered = scored.into_iter().map(|(_,i)| i).collect();
        }
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
    }
    fn move_down(&mut self) { if self.selected + 1 < self.filtered.len() { self.selected += 1; } }
    fn move_up(&mut self) { self.selected = self.selected.saturating_sub(1); }

    pub fn handle_key(&mut self, key:crossterm::event::KeyEvent) -> Result<()> {
        self.context_menu = None;
        if self.mode==Mode::Search { return self.handle_search_key(key); }
        if self.mode==Mode::Palette { return self.handle_palette_key(key); }
        if self.mode==Mode::Command { return self.handle_command_key(key); }
        if self.mode==Mode::HostForm { return self.handle_host_form_key(key); }
        match key.code {
            KeyCode::Char('q') => { if self.view==View::Dashboard { self.should_quit=true } else { self.view=View::Dashboard; } },
            KeyCode::Char('?') => self.view=View::Help,
            KeyCode::Char('/') => { self.mode=Mode::Search; self.search.clear(); },
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => { self.mode=Mode::Palette; self.palette_input.clear(); },
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Char('g') => self.selected=0,
            KeyCode::Char('G') => self.selected=self.filtered.len().saturating_sub(1),
            KeyCode::Right | KeyCode::Char('i') => self.view=View::HostDetail,
            KeyCode::Enter => self.connect_selected()?,
            KeyCode::Char('s') => { self.view=View::Files; self.remote_path="/var/www/app".into(); },
            KeyCode::Char('t') => { if let Some(h)=self.current_host(){ self.tunnel.host_alias=h.alias.clone(); } self.view=View::Tunnels; },
            KeyCode::Char('r') => { self.view=View::CommandRunner; self.command_input="uptime".into(); },
            KeyCode::Char('h') => self.fetch_health(),
            KeyCode::Char('l') => { self.logs=storage::read_logs(); self.view=View::Logs; },
            KeyCode::Char('a') => self.open_host_form(HostFormMode::Add),
            KeyCode::Char('e') => self.open_host_form(HostFormMode::Edit),
            KeyCode::Char('D') => self.open_host_form(HostFormMode::Duplicate),
            KeyCode::Char('d') => self.confirm_delete_host(),
            KeyCode::Char(':') if self.view==View::Files => { self.mode=Mode::Command; self.command_input.clear(); },
            KeyCode::Tab if self.view==View::Files => { if self.files_dual_pane { self.active_file_pane=1-self.active_file_pane; } else { self.files_dual_pane=true; } },
            KeyCode::BackTab if self.view==View::Files => { self.files_dual_pane=true; self.active_file_pane=1-self.active_file_pane; },
            KeyCode::Char('.') if self.view==View::Files => { self.config.files.show_hidden=!self.config.files.show_hidden; self.toast(ToastLevel::Info,format!("hidden files: {}", self.config.files.show_hidden)); },
            KeyCode::Char('T') if self.view==View::Files => self.mode=Mode::Transfer,
            KeyCode::Esc => { self.view=View::Dashboard; self.mode=Mode::Normal; },
            _ => {}
        }
        Ok(())
    }

    pub fn handle_mouse(&mut self, event: MouseEvent) -> Result<()> {
        let action = self.mouse.resolve(event);
        match action {
            MouseAction::Click(target) => self.dispatch_click(target)?,
            MouseAction::DoubleClick(ClickTarget::HostRow(i)) => { self.select_host_by_index(i); self.connect_selected()?; },
            MouseAction::DoubleClick(ClickTarget::FileEntry(path)) => { self.toast(ToastLevel::Info, format!("Open/preview {path}")); },
            MouseAction::DoubleClick(t) => self.dispatch_click(t)?,
            MouseAction::RightClick(ClickTarget::HostRow(i)) => { self.select_host_by_index(i); self.open_host_context(); },
            MouseAction::RightClick(ClickTarget::FileEntry(path)) => self.open_file_context(path),
            MouseAction::RightClick(_) => {},
            MouseAction::Scroll{target, delta} => self.scroll_target(target, delta),
            MouseAction::Drag{target, ..} => { if matches!(target, Some(ClickTarget::FileEntry(_))) { self.selected_files = self.selected_files.saturating_add(1); } },
            MouseAction::Move{target, ..} => { self.hover_target = target; },
        }
        Ok(())
    }

    fn dispatch_click(&mut self, target: ClickTarget) -> Result<()> {
        match target {
            ClickTarget::SidebarGroup(g) => self.click_nav(g),
            ClickTarget::HostRow(i) => self.select_host_by_index(i),
            ClickTarget::HostConnectButton(i) => { self.select_host_by_index(i); self.connect_selected()?; },
            ClickTarget::HostFilesButton(i) => { self.select_host_by_index(i); self.view=View::Files; },
            ClickTarget::HostTunnelButton(i) => { self.select_host_by_index(i); if let Some(h)=self.current_host(){ self.tunnel.host_alias=h.alias.clone(); } self.view=View::Tunnels; },
            ClickTarget::HostHealthButton(i) => { self.select_host_by_index(i); self.fetch_health(); },
            ClickTarget::HostEditButton(i) => { self.select_host_by_index(i); self.open_host_form(HostFormMode::Edit); },
            ClickTarget::FileEntry(p) => { self.focused_pane="files".into(); self.toast(ToastLevel::Info, format!("Selected {p}")); },
            ClickTarget::FilePreview => self.focused_pane="preview".into(),
            ClickTarget::Breadcrumb(p) => { self.remote_path=p; self.toast(ToastLevel::Info,"Breadcrumb jump".into()); },
            ClickTarget::CommandPaletteItem(a) => self.run_palette_action(&a)?,
            ClickTarget::ModalButton(b) if b=="close" || b=="cancel" => { self.context_menu=None; self.host_form=None; self.mode=Mode::Normal; },
            ClickTarget::ModalButton(b) if b=="add-host" => self.open_host_form(HostFormMode::Add),
            ClickTarget::ModalButton(b) if b=="import-hosts" => self.toast(ToastLevel::Info,"Import reads ~/.ssh/config automatically on startup".into()),
            ClickTarget::ModalButton(b) if b=="test-host" => self.test_host_form(),
            ClickTarget::ModalButton(b) if b=="save-host" => self.save_host_form()?,
            ClickTarget::ModalButton(b) if b=="delete-host" => self.confirm_delete_host(),
            ClickTarget::ModalButton(b) if b=="delete-host-confirm" => self.delete_selected_host()?,
            ClickTarget::ModalButton(b) if b=="add-include" => self.add_include_line()?,
            ClickTarget::ModalButton(_) => {},
            ClickTarget::Tab(v) => self.view=v,
            ClickTarget::TransferItem(id) => self.toast(ToastLevel::Info, format!("Transfer #{id} selected")),
            ClickTarget::TunnelType(t) => { self.tunnel.tunnel_type = match t.as_str(){"remote"=>TunnelType::Remote,"dynamic"=>TunnelType::Dynamic,_=>TunnelType::Local}; },
            ClickTarget::FormField(f) => { let map=["alias","hostname/ip","user","port","identity-file","group","tags","notes"]; if let Some(form)=self.host_form.as_mut(){ if let Some(pos)=map.iter().position(|m| *m==f){ form.field=pos; } } self.focused_pane=f; },
            ClickTarget::ToastClose => self.toast=None,
            ClickTarget::StatusShortcut(s) => self.activate_status_shortcut(&s)?,
            ClickTarget::Pane(p) => self.focused_pane=p,
        }
        Ok(())
    }

    fn click_nav(&mut self, group: String) {
        match group.as_str() {
            "Tunnels" => self.view = View::Tunnels,
            "Commands" => self.view = View::CommandRunner,
            "Logs" => self.view = View::Logs,
            "All" => {
                self.view = View::Dashboard;
                self.filtered = (0..self.hosts.len()).collect();
            }
            _ => {
                self.view = View::Dashboard;
                self.toast(ToastLevel::Info, format!("Showing {group}"));
            }
        }
    }
    fn select_host_by_index(&mut self, host_index:usize) { if let Some(pos)=self.filtered.iter().position(|i| *i==host_index){ self.selected=pos; } }
    fn scroll_target(&mut self, target:Option<ClickTarget>, delta:i16) { match target { Some(ClickTarget::FilePreview) => self.preview_scroll = add_scroll(self.preview_scroll, delta), Some(ClickTarget::FileEntry(_)) => self.file_scroll = add_scroll(self.file_scroll, delta), _ => self.host_scroll = add_scroll(self.host_scroll, delta) } }
    fn open_host_context(&mut self) { if let Some(h)=self.current_host(){ let title=h.alias.clone(); self.context_menu=Some(ContextMenu{title, items:vec![ ("Connect".into(),ClickTarget::HostConnectButton(self.current_host_index().unwrap_or(0))), ("Files".into(),ClickTarget::HostFilesButton(self.current_host_index().unwrap_or(0))), ("Tunnel".into(),ClickTarget::HostTunnelButton(self.current_host_index().unwrap_or(0))), ("Run Command".into(),ClickTarget::StatusShortcut("run".into())), ("Health".into(),ClickTarget::HostHealthButton(self.current_host_index().unwrap_or(0))), ("Edit".into(),ClickTarget::HostEditButton(self.current_host_index().unwrap_or(0))), ("Delete".into(),ClickTarget::ModalButton("delete-host".into())) ]}); } }
    fn open_file_context(&mut self, path:String) { self.context_menu=Some(ContextMenu{title:path.clone(),items:vec![("Preview".into(),ClickTarget::FileEntry(path.clone())),("Edit".into(),ClickTarget::StatusShortcut("edit-file".into())),("Download".into(),ClickTarget::StatusShortcut("download".into())),("Rename".into(),ClickTarget::StatusShortcut("rename".into())),("Copy Path".into(),ClickTarget::Breadcrumb(path)),("Delete".into(),ClickTarget::ModalButton("delete-file".into()))]}); }

    fn activate_status_shortcut(&mut self, shortcut: &str) -> Result<()> {
        match shortcut.to_ascii_lowercase().as_str() {
            "/" => { self.mode = Mode::Search; self.search.clear(); },
            "?" => self.view = View::Help,
            "a" => self.open_host_form(HostFormMode::Add),
            "enter" => self.connect_selected()?,
            "s" => { self.view = View::Files; self.remote_path = "/var/www/app".into(); },
            "t" => { if let Some(h)=self.current_host(){ self.tunnel.host_alias=h.alias.clone(); } self.view=View::Tunnels; },
            "r" | ":" => { self.view=View::CommandRunner; self.command_input="uptime".into(); },
            "h" => self.fetch_health(),
            "l" => { self.logs=storage::read_logs(); self.view=View::Logs; },
            "esc" => { self.context_menu=None; self.host_form=None; self.mode=Mode::Normal; if self.view != View::Dashboard { self.view=View::Dashboard; } },
            "tab" => { if self.view==View::Files { if self.files_dual_pane { self.active_file_pane=1-self.active_file_pane; } else { self.files_dual_pane=true; } } },
            "ctrl+p" => { self.mode=Mode::Palette; self.palette_input.clear(); },
            "j/k" | "h/l" | "space" | "right-click" | "click" => self.toast(ToastLevel::Info, format!("Shortcut: {shortcut}")),
            other => self.run_palette_action(other)?,
        }
        Ok(())
    }
    
    fn run_palette_action(&mut self, action: &str) -> Result<()> {
        let action = action.to_ascii_lowercase();
        self.mode = Mode::Normal;

        if action.contains("include") {
            self.toast(ToastLevel::Info, "Add this line to ~/.ssh/config: Include ~/.config/sshdeck/ssh_config".into());
        } else if action.contains("add host") || action == "a" {
            self.open_host_form(HostFormMode::Add);
        } else if action.contains("duplicate") {
            self.open_host_form(HostFormMode::Duplicate);
        } else if action.contains("theme") {
            self.toggle_theme();
        } else if action.contains("file") || action == "s" {
            self.view = View::Files;
        } else if action.contains("tunnel") || action == "t" {
            self.view = View::Tunnels;
        } else if action.contains("health") || action == "h" {
            self.fetch_health();
        } else if action.contains("run") || action == "r" {
            self.view = View::CommandRunner;
        } else if action.contains("quit") {
            self.should_quit = true;
        } else {
            self.toast(ToastLevel::Info, format!("I don't know that action yet: {action}"));
        }

        Ok(())
    }


    fn open_host_form(&mut self, mode: HostFormMode) {
        let (draft, original_alias) = match mode {
            HostFormMode::Add => (HostDraft::default(), None),
            HostFormMode::Edit => self.current_host().map(|h| (HostDraft::from_host(h), Some(h.alias.clone()))).unwrap_or((HostDraft::default(), None)),
            HostFormMode::Duplicate => self.current_host().map(|h| { let mut d=HostDraft::from_host(h); d.alias=format!("{}-copy", h.alias); (d, None) }).unwrap_or((HostDraft::default(), None)),
        };
        self.host_form = Some(HostFormState { mode, draft, field: 0, messages: Vec::new(), test_result: None, original_alias });
        self.mode = Mode::HostForm;
        self.context_menu = None;
    }

    fn handle_host_form_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> {
        use crossterm::event::KeyModifiers;
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) { return self.save_host_form(); }
        let Some(form)=self.host_form.as_mut() else { self.mode=Mode::Normal; return Ok(()); };
        match key.code {
            KeyCode::Esc => { self.host_form=None; self.mode=Mode::Normal; },
            KeyCode::Tab => form.field = (form.field + 1) % form.field_count(),
            KeyCode::BackTab => form.field = if form.field==0 { form.field_count()-1 } else { form.field-1 },
            KeyCode::Up => form.field = form.field.saturating_sub(1),
            KeyCode::Down => form.field = (form.field + 1).min(form.field_count()-1),
            KeyCode::Enter => match form.field { 8 => self.test_host_form(), 9 => self.save_host_form()?, 10 => { self.host_form=None; self.mode=Mode::Normal; }, _ => form.field = (form.field + 1) % form.field_count() },
            KeyCode::Backspace => self.edit_host_field_backspace(),
            KeyCode::Char(c) => self.edit_host_field_char(c),
            _ => {}
        }
        Ok(())
    }

    fn edit_host_field_char(&mut self, c: char) { if let Some(form)=self.host_form.as_mut(){ match form.field { 0=>form.draft.alias.push(c), 1=>form.draft.hostname.push(c), 2=>form.draft.user.push(c), 3=>form.draft.port.push(c), 4=>form.draft.identity_file.push(c), 5=>form.draft.group.push(c), 6=>form.draft.tags.push(c), 7=>form.draft.notes.push(c), _=>{} } } }
    fn edit_host_field_backspace(&mut self) { if let Some(form)=self.host_form.as_mut(){ match form.field { 0=>{form.draft.alias.pop();}, 1=>{form.draft.hostname.pop();}, 2=>{form.draft.user.pop();}, 3=>{form.draft.port.pop();}, 4=>{form.draft.identity_file.pop();}, 5=>{form.draft.group.pop();}, 6=>{form.draft.tags.pop();}, 7=>{form.draft.notes.pop();}, _=>{} } } }

    fn existing_aliases_for_form(&self)->Vec<String>{ self.hosts.iter().map(|h|h.alias.clone()).filter(|a| self.host_form.as_ref().and_then(|f|f.original_alias.as_ref()).map(|o|o!=a).unwrap_or(true)).collect() }
    fn validate_current_form(&mut self)->bool{ let aliases=self.existing_aliases_for_form(); let Some(form)=self.host_form.as_mut() else { return false; }; form.messages=managed_hosts::validate_host_draft(&form.draft,&aliases); !form.messages.iter().any(|m| m.level==HostValidationLevel::Error) }
    fn test_host_form(&mut self){ if !self.validate_current_form(){ return; } let Some(form)=self.host_form.as_mut() else { return; }; let Some(host)=form.draft.to_host() else { form.test_result=Some("✗ Connection test could not build an SSH command from this form".into()); return; }; let args=ssh_test_args_for(&host, 5); let command=display_command("ssh", &args); let status=std::process::Command::new("ssh").args(&args).status(); form.test_result=Some(match status { Ok(s) if s.success()=>format!("✓ Connection successful\n{command}"), Ok(s)=>format!("✗ Connection failed: ssh exited with {s}\n{command}"), Err(e)=>format!("✗ Connection failed: {e}\n{command}") }); }
    fn save_host_form(&mut self)->Result<()> { if !self.validate_current_form(){ return Ok(()); } let Some(form)=self.host_form.clone() else { return Ok(()); }; let Some(host)=form.draft.to_host() else { return Ok(()); };
        let original=form.original_alias.clone();
        self.hosts.retain(|h| Some(&h.alias)!=original.as_ref() && h.alias != host.alias);
        self.hosts.push(host.clone());
        if !self.managed_aliases.contains(&host.alias){ self.managed_aliases.push(host.alias.clone()); }
        let managed:Vec<_>=self.hosts.iter().filter(|h| self.managed_aliases.contains(&h.alias)).cloned().collect();
        managed_hosts::save_managed_hosts(&managed_hosts::managed_config_path(), &managed)?;
        self.config.hosts.insert(host.alias.clone(), HostMeta{ tags: host.tags.clone(), group: host.group.clone(), favorite: host.favorite, notes: host.notes.clone() });
        self.config.save()?;
        self.filtered=(0..self.hosts.len()).collect(); self.selected=self.hosts.iter().position(|h|h.alias==host.alias).unwrap_or(0);
        self.host_form=None; self.mode=Mode::Normal;
        self.context_menu=Some(ContextMenu{title:"Managed config saved".into(),items:vec![("Add Include automatically".into(),ClickTarget::ModalButton("add-include".into())), ("Show command: Include ~/.config/sshdeck/ssh_config".into(),ClickTarget::StatusShortcut("show-include".into())), ("Later".into(),ClickTarget::ModalButton("cancel".into()))]});
        self.toast(ToastLevel::Success, format!("Saved host {} to {}", host.alias, managed_hosts::managed_config_path().display()));
        Ok(()) }
    fn confirm_delete_host(&mut self){ if let Some(h)=self.current_host(){ self.context_menu=Some(ContextMenu{title:format!("Delete {}?",h.alias),items:vec![("Delete managed host / hide imported host".into(),ClickTarget::ModalButton("delete-host-confirm".into())), ("Cancel".into(),ClickTarget::ModalButton("cancel".into()))]}); } }
    fn delete_selected_host(&mut self)->Result<()> { let Some(idx)=self.current_host_index() else { return Ok(()); }; let alias=self.hosts[idx].alias.clone(); if self.managed_aliases.contains(&alias){ self.hosts.remove(idx); self.managed_aliases.retain(|a|a!=&alias); let managed:Vec<_>=self.hosts.iter().filter(|h| self.managed_aliases.contains(&h.alias)).cloned().collect(); managed_hosts::save_managed_hosts(&managed_hosts::managed_config_path(), &managed)?; } else { self.hide_aliases.push(alias.clone()); self.hosts.remove(idx); } self.config.hosts.remove(&alias); self.config.save()?; self.filtered=(0..self.hosts.len()).collect(); self.selected=self.selected.min(self.filtered.len().saturating_sub(1)); self.context_menu=None; self.toast(ToastLevel::Warning,format!("Removed {alias} from SSHDeck view")); Ok(()) }
    fn toggle_theme(&mut self){ self.config.ui.theme=match self.config.ui.theme.as_str(){"blackout"=>"minimal".into(),"minimal"=>"cyber".into(),_=>"blackout".into()}; self.theme=Theme::named(&self.config.ui.theme); let _=self.config.save(); self.toast(ToastLevel::Info,format!("Theme: {}",self.config.ui.theme)); }
    fn add_include_line(&mut self)->Result<()> { let path=dirs::home_dir().unwrap_or_default().join(".ssh/config"); let line="Include ~/.config/sshdeck/ssh_config"; let changed=managed_hosts::ensure_include_line(&path,line)?; self.context_menu=None; self.toast(ToastLevel::Success, if changed {format!("Added {line} to {} with backup", path.display())} else {"Include line already present".into()}); Ok(()) }

    fn handle_search_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> { match key.code { KeyCode::Esc => { self.search.clear(); self.mode=Mode::Normal; self.filter_hosts(); }, KeyCode::Enter => self.mode=Mode::Normal, KeyCode::Backspace => { self.search.pop(); self.filter_hosts(); }, KeyCode::Char(c) => { self.search.push(c); self.filter_hosts(); }, _=>{} } Ok(()) }
    fn handle_palette_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> { match key.code { KeyCode::Esc => self.mode=Mode::Normal, KeyCode::Enter => { let q=self.palette_input.clone(); self.run_palette_action(&q)?; }, KeyCode::Backspace=>{self.palette_input.pop();}, KeyCode::Char(c)=>self.palette_input.push(c), _=>{} } Ok(()) }
    fn handle_command_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> { match key.code { KeyCode::Esc=>self.mode=Mode::Normal, KeyCode::Backspace=>{self.command_input.pop();}, KeyCode::Enter=>{ let cmd=self.command_input.clone(); self.mode=Mode::Normal; if is_dangerous_command(&cmd){ self.toast(ToastLevel::Warning,"Dangerous command blocked pending explicit confirmation".into()); } else { self.command_output=format!("$ {}\n(command mode parsed; remote execution uses ssh in command runner)", cmd); self.toast(ToastLevel::Success,format!("Command accepted: {cmd}")); } }, KeyCode::Char(c)=>self.command_input.push(c), _=>{} } Ok(()) }

    pub fn connect_selected(&mut self)->Result<()> {
        if let Some(h)=self.current_host(){
            let alias=h.alias.clone(); let args=ssh_args_for(h); let cmd=display_command("ssh", &args); self.toast(ToastLevel::Info,format!("Connecting: {cmd}"));
            crossterm::terminal::disable_raw_mode()?;
            if self.mouse_enabled { crossterm::execute!(std::io::stdout(), Show, DisableMouseCapture, crossterm::terminal::LeaveAlternateScreen)?; } else { crossterm::execute!(std::io::stdout(), Show, crossterm::terminal::LeaveAlternateScreen)?; }
            let status=Command::new("ssh").args(&args).status();
            if self.mouse_enabled { crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0), Hide, EnableMouseCapture)?; } else { crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0), Hide)?; }
            crossterm::terminal::enable_raw_mode()?;
            self.render_reset_needed = true;
            match status { Ok(s) if s.success()=>self.toast(ToastLevel::Success,format!("Returned from {alias}")), Ok(s)=>self.toast(ToastLevel::Warning,format!("SSH exited with {s}: {cmd}")), Err(e)=>self.toast(ToastLevel::Error,format!("Could not start ssh: {e}")) }
        }
        Ok(())
    }
    pub fn fetch_health(&mut self){ if let Some(h)=self.current_host(){ self.health.uptime=format!("{} health check queued", h.alias); self.view=View::HostDetail; self.toast(ToastLevel::Info,"Health commands: uptime, df -h, free -h, uname -a".into()); } }
}

fn add_scroll(v:usize, delta:i16)->usize { if delta < 0 { v.saturating_sub((-delta) as usize) } else { v.saturating_add(delta as usize) } }
