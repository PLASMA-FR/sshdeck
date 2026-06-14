use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

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
    files::{
        command_mode::{parse_command, FileCommand},
        file_entry::{FileEntry, FileKind},
        local_fs, remote_fs,
        safety::{self, is_sensitive_path},
        transfer::{TransferDirection, TransferQueue},
    },
    mouse::{ClickTarget, MouseAction, MouseState},
    ssh::{
        command::{
            display_command, is_dangerous_command, run_ssh_command_for, scp_download_args_for,
            scp_upload_args_for, ssh_args_for, ssh_test_args_for,
        },
        health::HealthInfo,
        host::SshHost,
        session_frame,
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

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteHost,
    DeleteRemoteFile(String),
    RunCommand(String),
    FileCommand(FileCommand),
    PreviewSensitive(String),
    SafeEdit(String),
}

#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub title: String,
    pub prompt: String,
    pub expected: String,
    pub input: String,
    pub action: ConfirmAction,
}

pub enum TaskResult {
    RemoteList { host_alias: String, path: String, result: Result<Vec<FileEntry>, String> },
    RemotePreview { path: String, result: Result<String, String> },
    TransferFinished { id: u64, result: Result<String, String> },
    RemoteCommand { command: String, result: Result<String, String> },
    Health { host_alias: String, result: Result<HealthInfo, String> },
    HostTest { command: String, result: String },
    RemoteAction { label: String, refresh_path: Option<String>, result: Result<String, String> },
}

pub struct TunnelSession {
    pub command: String,
    pub child: Child,
}

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
    pub remote_entries: Vec<FileEntry>,
    pub remote_error: Option<String>,
    pub remote_loading: bool,
    pub remote_preview: Option<String>,
    pub file_selected: usize,
    pub local_path: String,
    pub local_entries: Vec<FileEntry>,
    pub local_error: Option<String>,
    pub local_selected: usize,
    pub local_scroll: usize,
    pub files_dual_pane: bool,
    pub active_file_pane: usize,
    pub selected_files: usize,
    pub selected_file_paths: Vec<String>,
    pub transfer_queue: TransferQueue,
    pub settings_selected: usize,
    pub command_output: String,
    pub command_history: Vec<String>,
    pub managed_aliases: Vec<String>,
    pub host_form: Option<HostFormState>,
    pub confirm: Option<ConfirmState>,
    pub hide_aliases: Vec<String>,
    pub active_nav: String,
    pub palette_selected: usize,
    pub context_menu_selected: usize,
    pub active_tunnel: Option<TunnelSession>,
    task_tx: Sender<TaskResult>,
    task_rx: Receiver<TaskResult>,
    pub render_reset_needed: bool,
    pub splash_ticks: u8,
    pub splash_total_ticks: u8,
}

impl App {
    pub fn new(config: AppConfig, options: AppOptions) -> Result<Self> {
        let mut hosts = parse_default_ssh_config().unwrap_or_default();
        let managed_path = managed_hosts::managed_config_path();
        let managed_hosts_loaded = managed_hosts::read_managed_hosts(&managed_path).unwrap_or_default();
        let managed_aliases: Vec<String> = managed_hosts_loaded.iter().map(|h| h.alias.clone()).collect();
        hosts.extend(managed_hosts_loaded);
        hosts.retain(|h| managed_aliases.contains(&h.alias) || !config.hidden_imported_hosts.contains(&h.alias));
        for h in &mut hosts {
            if let Some(meta) = config.hosts.get(&h.alias) {
                h.tags = meta.tags.clone(); h.group = meta.group.clone(); h.favorite = meta.favorite; h.notes = meta.notes.clone();
            }
            if config.recent_hosts.contains(&h.alias) && h.recent_connection.is_none() {
                h.recent_connection = Some("recent".into());
            }
        }
        let theme = Theme::named(&config.ui.theme);
        let splash_total_ticks = if config.ui.animations && !options.no_animations { 18 } else { 0 };
        let (task_tx, task_rx) = mpsc::channel();
        let mut app = Self {
            filtered: (0..hosts.len()).collect(), hosts, selected: 0, host_scroll: 0, file_scroll: 0, preview_scroll: 0,
            view: View::Dashboard, mode: Mode::Normal, search: String::new(), command_input: String::new(), palette_input: String::new(),
            logs: storage::read_logs(), theme, animator: Animator::new(config.ui.animations && !options.no_animations),
            ascii: options.ascii || !config.ui.unicode, mouse_enabled: options.mouse, mouse: MouseState::default(), focused_pane: "hosts".into(), hover_target: None, context_menu: None,
            should_quit: false, toast: None, health: HealthInfo::empty(),
            tunnel: TunnelConfig { tunnel_type: TunnelType::Local, host_alias: String::new(), bind_address: None, local_port: 8080, target_host: Some("localhost".into()), target_port: Some(80) },
            remote_path: "~".into(), remote_entries: Vec::new(), remote_error: None, remote_loading: false, remote_preview: None, file_selected: 0,
            local_path: config.files.default_local_dir.clone(), local_entries: Vec::new(), local_error: None, local_selected: 0, local_scroll: 0,
            files_dual_pane: false, active_file_pane: 1, selected_files: 0, selected_file_paths: Vec::new(),
            transfer_queue: TransferQueue::default(), settings_selected: 0, command_output: String::new(), command_history: Vec::new(), managed_aliases,
            host_form: None, confirm: None, hide_aliases: Vec::new(), active_nav: "All".into(), palette_selected: 0, context_menu_selected: 0, active_tunnel: None,
            task_tx, task_rx, render_reset_needed: false, splash_ticks: 0, splash_total_ticks, config,
        };
        app.refresh_local_files();
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
    pub fn show_splash(&self) -> bool { self.splash_ticks < self.splash_total_ticks }
    pub fn dismiss_splash(&mut self) { self.splash_ticks = self.splash_total_ticks; }
    pub fn on_tick(&mut self) {
        self.animator.tick();
        if self.show_splash() {
            self.splash_ticks = self.splash_ticks.saturating_add(1);
        }
        while let Ok(result) = self.task_rx.try_recv() {
            self.apply_task_result(result);
        }
        self.poll_tunnel();
        if let Some(t)=self.toast.as_mut(){ if t.ttl>0 { t.ttl-=1; } else { self.toast=None; } }
    }
    pub fn toast(&mut self, level:ToastLevel, message:String) { storage::append_log(&message); self.logs.push(message.clone()); self.toast=Some(Toast{message,ttl:40,level}); }
    pub fn icons(&self) -> crate::design::icons::Icons { if self.ascii { crate::design::icons::ascii() } else { crate::design::icons::nerd() } }
    pub fn is_hovered(&self, target: &ClickTarget) -> bool { self.hover_target.as_ref().is_some_and(|h| h == target) }

    fn queue_task<F>(&self, task: F)
    where
        F: FnOnce() -> TaskResult + Send + 'static,
    {
        let tx = self.task_tx.clone();
        thread::spawn(move || {
            let _ = tx.send(task());
        });
    }

    fn apply_task_result(&mut self, result: TaskResult) {
        match result {
            TaskResult::RemoteList { host_alias, path, result } => {
                self.remote_loading = false;
                if path != self.remote_path {
                    return;
                }
                match result {
                    Ok(entries) => {
                        let count = entries.len();
                        self.remote_entries = entries;
                        self.remote_error = None;
                        self.file_selected = self.file_selected.min(self.remote_entries.len().saturating_sub(1));
                        self.toast(ToastLevel::Success, format!("Opened {host_alias}:{path} · {count} item(s)"));
                    }
                    Err(e) => {
                        self.remote_entries.clear();
                        self.remote_error = Some(e);
                        self.toast(ToastLevel::Error, format!("Could not open {host_alias}:{path}"));
                    }
                }
            }
            TaskResult::RemotePreview { path, result } => match result {
                Ok(text) => {
                    self.remote_preview = Some(text);
                    self.toast(ToastLevel::Info, format!("Preview loaded: {path}"));
                }
                Err(e) => {
                    self.remote_preview = Some(format!("Could not preview {path}\n\n{e}"));
                    self.toast(ToastLevel::Warning, format!("Could not preview {path}"));
                }
            },
            TaskResult::TransferFinished { id, result } => match result {
                Ok(msg) => {
                    self.transfer_queue.complete(id);
                    self.toast(ToastLevel::Success, msg);
                    self.refresh_after_transfer();
                }
                Err(e) => {
                    self.transfer_queue.fail(id, e.clone());
                    self.toast(ToastLevel::Error, format!("Transfer failed: {e}"));
                }
            },
            TaskResult::RemoteCommand { command, result } => match result {
                Ok(output) => {
                    self.command_output = if output.trim().is_empty() {
                        format!("$ {command}\n(no output)")
                    } else {
                        format!("$ {command}\n{output}")
                    };
                    self.command_history.push(command.clone());
                    self.toast(ToastLevel::Success, format!("Command finished: {command}"));
                }
                Err(e) => {
                    self.command_output = format!("$ {command}\n{e}");
                    self.toast(ToastLevel::Error, format!("Command failed: {command}"));
                }
            },
            TaskResult::Health { host_alias, result } => match result {
                Ok(info) => {
                    self.health = info;
                    self.view = View::HostDetail;
                    self.toast(ToastLevel::Success, format!("Health updated for {host_alias}"));
                }
                Err(e) => {
                    self.view = View::HostDetail;
                    self.health.uptime = format!("Could not check {host_alias}");
                    self.toast(ToastLevel::Error, e);
                }
            },
            TaskResult::HostTest { command, result } => {
                if let Some(form) = self.host_form.as_mut() {
                    form.test_result = Some(format!("{result}\n{command}"));
                }
            }
            TaskResult::RemoteAction { label, refresh_path, result } => match result {
                Ok(msg) => {
                    self.toast(ToastLevel::Success, if msg.trim().is_empty() { label } else { msg });
                    if let Some(path) = refresh_path {
                        self.open_remote_path(path);
                    }
                }
                Err(e) => self.toast(ToastLevel::Error, format!("{label} failed: {e}")),
            },
        }
    }

    fn poll_tunnel(&mut self) {
        let Some(session) = self.active_tunnel.as_mut() else { return; };
        match session.child.try_wait() {
            Ok(Some(status)) => {
                let command = session.command.clone();
                self.active_tunnel = None;
                if status.success() {
                    self.toast(ToastLevel::Info, format!("Tunnel closed: {command}"));
                } else {
                    self.toast(ToastLevel::Warning, format!("Tunnel exited with {status}: {command}"));
                }
            }
            Ok(None) => {}
            Err(e) => {
                self.active_tunnel = None;
                self.toast(ToastLevel::Error, format!("Could not poll tunnel: {e}"));
            }
        }
    }

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
        if self.show_splash() {
            self.dismiss_splash();
            if !matches!(key.code, KeyCode::Char('q')) { return Ok(()); }
        }
        if self.mode==Mode::Search { return self.handle_search_key(key); }
        if self.mode==Mode::Palette { return self.handle_palette_key(key); }
        if self.mode==Mode::Command { return self.handle_command_key(key); }
        if self.mode==Mode::HostForm { return self.handle_host_form_key(key); }
        if self.mode==Mode::Confirm { return self.handle_confirm_key(key); }
        if self.context_menu.is_some() {
            match key.code {
                KeyCode::Esc => { self.context_menu = None; return Ok(()); }
                KeyCode::Down | KeyCode::Char('j') => { self.move_context_menu(1); return Ok(()); }
                KeyCode::Up | KeyCode::Char('k') => { self.move_context_menu(-1); return Ok(()); }
                KeyCode::Enter => { self.activate_context_menu_selected()?; return Ok(()); }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Char('q') => { if self.view==View::Dashboard { self.should_quit=true } else { self.view=View::Dashboard; } },
            KeyCode::Char('?') => self.view=View::Help,
            KeyCode::Char('/') => { self.mode=Mode::Search; self.search.clear(); },
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => { self.mode=Mode::Palette; self.palette_input.clear(); self.palette_selected=0; },
            KeyCode::Down | KeyCode::Char('j') if self.view==View::Settings => self.move_setting_down(),
            KeyCode::Up | KeyCode::Char('k') if self.view==View::Settings => self.move_setting_up(),
            KeyCode::Enter | KeyCode::Char(' ') if self.view==View::Settings => self.activate_selected_setting()?,
            KeyCode::Enter if self.view==View::Files => self.open_selected_remote_entry(),
            KeyCode::Char(' ') if self.view==View::Files => self.toggle_selected_file(),
            KeyCode::Enter if self.view==View::Tunnels => self.toggle_tunnel(),
            KeyCode::Char('c') if self.view==View::Tunnels => self.copy_tunnel_command(),
            KeyCode::Char('S') if self.view==View::Tunnels => self.stop_tunnel(),
            KeyCode::Down | KeyCode::Char('j') if self.view==View::Files => self.move_file_down(),
            KeyCode::Up | KeyCode::Char('k') if self.view==View::Files => self.move_file_up(),
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace if self.view==View::Files => self.open_remote_path(remote_fs::parent_remote_path(&self.remote_path)),
            KeyCode::Right | KeyCode::Char('l') if self.view==View::Files => self.open_selected_remote_entry(),
            KeyCode::Char('~') if self.view==View::Files => self.open_remote_path("~".into()),
            KeyCode::Char('R') if self.view==View::Files => self.refresh_remote_files(),
            KeyCode::Char('p') if self.view==View::Files => self.preview_selected_remote_entry(),
            KeyCode::Char('u') if self.view==View::Files => self.queue_upload_selected(),
            KeyCode::Char('d') if self.view==View::Files => self.queue_download_selected(),
            KeyCode::Char('x') if self.view==View::Files => self.confirm_delete_selected_file(),
            KeyCode::Char('n') if self.view==View::Files => self.open_file_command_template("mkdir "),
            KeyCode::Char('b') if self.view==View::Files => self.add_bookmark_for_current_path(),
            KeyCode::Char('r') if self.view==View::CommandRunner => self.run_current_command(),
            KeyCode::Char(':') if self.view==View::CommandRunner => { self.mode=Mode::Command; self.command_input.clear(); },
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Char('g') => self.selected=0,
            KeyCode::Char('G') => self.selected=self.filtered.len().saturating_sub(1),
            KeyCode::Right | KeyCode::Char('i') => self.view=View::HostDetail,
            KeyCode::Enter => self.connect_selected()?,
            KeyCode::Char('s') => self.open_files_home(),
            KeyCode::Char('t') => { if let Some(h)=self.current_host(){ self.tunnel.host_alias=h.alias.clone(); } self.view=View::Tunnels; },
            KeyCode::Char('r') => { self.view=View::CommandRunner; self.command_input="uptime".into(); },
            KeyCode::Char('h') => self.fetch_health(),
            KeyCode::Char('l') => { self.logs=storage::read_logs(); self.view=View::Logs; },
            KeyCode::Char(',') => { self.view=View::Settings; self.settings_selected=0; },
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
        if self.show_splash() {
            self.dismiss_splash();
            return Ok(());
        }
        let action = self.mouse.resolve(event);
        match action {
            MouseAction::Click(target) => self.dispatch_click(target)?,
            MouseAction::DoubleClick(ClickTarget::HostRow(i)) => { self.select_host_by_index(i); self.connect_selected()?; },
            MouseAction::DoubleClick(ClickTarget::FileEntry(path)) => {
                if let Some(pos)=self.local_entries.iter().position(|e| e.path==path){ self.local_selected=pos; self.active_file_pane=0; }
                if let Some(pos)=self.remote_entries.iter().position(|e| e.path==path){ self.file_selected=pos; self.active_file_pane=1; }
                self.open_selected_remote_entry();
            },
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
            ClickTarget::SidebarGroup(g) | ClickTarget::SidebarItem(g) => self.click_nav(g),
            ClickTarget::HostRow(i) => self.select_host_by_index(i),
            ClickTarget::HostActionButton { host_index, action } => { self.select_host_by_index(host_index); self.run_palette_action(&action)?; },
            ClickTarget::HostConnectButton(i) => { self.select_host_by_index(i); self.connect_selected()?; },
            ClickTarget::HostFilesButton(i) => { self.select_host_by_index(i); self.open_files_home(); },
            ClickTarget::HostTunnelButton(i) => { self.select_host_by_index(i); if let Some(h)=self.current_host(){ self.tunnel.host_alias=h.alias.clone(); } self.view=View::Tunnels; },
            ClickTarget::HostHealthButton(i) => { self.select_host_by_index(i); self.fetch_health(); },
            ClickTarget::HostEditButton(i) => { self.select_host_by_index(i); self.open_host_form(HostFormMode::Edit); },
            ClickTarget::FileEntry(p) => {
                self.focused_pane="files".into();
                if self.view==View::Files {
                    if let Some(pos)=self.local_entries.iter().position(|e| e.path==p){ self.local_selected=pos; self.active_file_pane=0; }
                    if let Some(pos)=self.remote_entries.iter().position(|e| e.path==p){ self.file_selected=pos; self.active_file_pane=1; }
                }
                self.toast(ToastLevel::Info, format!("Selected {p}"));
            },
            ClickTarget::FilePreview => self.focused_pane="preview".into(),
            ClickTarget::Breadcrumb(p) => { self.open_remote_path(p); },
            ClickTarget::CommandPaletteItem(a) => self.run_palette_action(&a)?,
            ClickTarget::SettingRow(id) => self.activate_setting(&id)?,
            ClickTarget::ModalButton(b) if b=="close" || b=="cancel" => { self.context_menu=None; self.host_form=None; self.mode=Mode::Normal; },
            ClickTarget::ModalButton(b) if b=="add-host" => self.open_host_form(HostFormMode::Add),
            ClickTarget::AddHostButton => self.open_host_form(HostFormMode::Add),
            ClickTarget::ModalButton(b) if b=="import-hosts" => self.toast(ToastLevel::Info,"Import reads ~/.ssh/config automatically on startup".into()),
            ClickTarget::ModalButton(b) if b=="test-host" => self.test_host_form(),
            ClickTarget::ModalButton(b) if b=="save-host" => self.save_host_form()?,
            ClickTarget::ModalButton(b) if b=="delete-host" => self.confirm_delete_host(),
            ClickTarget::ModalButton(b) if b=="delete-host-confirm" => self.delete_selected_host()?,
            ClickTarget::ModalButton(b) if b=="start-tunnel" => self.toggle_tunnel(),
            ClickTarget::ModalButton(b) if b=="stop-tunnel" => self.stop_tunnel(),
            ClickTarget::ModalButton(b) if b=="copy-tunnel" => self.copy_tunnel_command(),
            ClickTarget::ModalButton(b) if b=="add-include" => self.add_include_line()?,
            ClickTarget::ModalButton(_) => {},
            ClickTarget::Tab(v) => self.view=v,
            ClickTarget::TransferItem(id) => self.toast(ToastLevel::Info, format!("Transfer #{id} selected")),
            ClickTarget::TunnelType(t) => { self.tunnel.tunnel_type = match t.as_str(){"remote"=>TunnelType::Remote,"dynamic"=>TunnelType::Dynamic,_=>TunnelType::Local}; },
            ClickTarget::FormField(f) => { let map=["alias","hostname/ip","user","port","identity-file","group","tags","notes"]; if let Some(form)=self.host_form.as_mut(){ if let Some(pos)=map.iter().position(|m| *m==f){ form.field=pos; } } self.focused_pane=f; },
            ClickTarget::ToastClose => self.toast=None,
            ClickTarget::StatusShortcut(s) | ClickTarget::ContextMenuItem(s) => self.activate_status_shortcut(&s)?,
            ClickTarget::Pane(p) => self.focused_pane=p,
        }
        Ok(())
    }

    fn click_nav(&mut self, group: String) {
        self.active_nav = group.clone();
        match group.as_str() {
            "Tunnels" => self.view = View::Tunnels,
            "Commands" => self.view = View::CommandRunner,
            "Logs" => self.view = View::Logs,
            "Settings" => self.view = View::Settings,
            "All" => {
                self.view = View::Dashboard;
                self.filtered = (0..self.hosts.len()).collect();
                self.selected = 0;
            }
            "Favorites" => self.filter_by(|h| h.favorite),
            "Production" => self.filter_by(|h| h.group.as_deref().is_some_and(|g| g.eq_ignore_ascii_case("production")) || h.tags.iter().any(|t| t.eq_ignore_ascii_case("production") || t.eq_ignore_ascii_case("prod"))),
            "Homelab" => self.filter_by(|h| h.group.as_deref().is_some_and(|g| g.eq_ignore_ascii_case("homelab")) || h.tags.iter().any(|t| t.eq_ignore_ascii_case("homelab") || t.eq_ignore_ascii_case("home"))),
            "Recent" => self.filter_by(|h| h.recent_connection.is_some()),
            _ => {
                self.view = View::Dashboard;
                self.toast(ToastLevel::Info, format!("Showing {group}"));
            }
        }
    }

    fn filter_by(&mut self, predicate: impl Fn(&SshHost) -> bool) {
        self.view = View::Dashboard;
        self.filtered = self.hosts.iter().enumerate().filter_map(|(i, h)| predicate(h).then_some(i)).collect();
        self.selected = 0;
        self.host_scroll = 0;
    }
    fn select_host_by_index(&mut self, host_index:usize) { if let Some(pos)=self.filtered.iter().position(|i| *i==host_index){ self.selected=pos; } }
    fn scroll_target(&mut self, target:Option<ClickTarget>, delta:i16) { match target { Some(ClickTarget::FilePreview) => self.preview_scroll = add_scroll(self.preview_scroll, delta), Some(ClickTarget::Pane(p)) if p=="local" => self.local_scroll = add_scroll(self.local_scroll, delta), Some(ClickTarget::FileEntry(_)) if self.active_file_pane==0 && self.files_dual_pane => self.local_scroll = add_scroll(self.local_scroll, delta), Some(ClickTarget::FileEntry(_)) => self.file_scroll = add_scroll(self.file_scroll, delta), _ => self.host_scroll = add_scroll(self.host_scroll, delta) } }

    fn open_files_home(&mut self) {
        self.view = View::Files;
        self.files_dual_pane = false;
        self.active_file_pane = 1;
        self.refresh_local_files();
        self.open_remote_path("~".into());
    }

    fn refresh_remote_files(&mut self) {
        self.open_remote_path(self.remote_path.clone());
    }

    fn open_remote_path(&mut self, path: String) {
        self.view = View::Files;
        self.remote_path = if path.trim().is_empty() { "~".into() } else { path };
        self.file_selected = 0;
        self.file_scroll = 0;
        self.preview_scroll = 0;
        self.remote_preview = None;
        let Some(host) = self.current_host().cloned() else {
            self.remote_entries.clear();
            self.remote_error = Some("No host selected".into());
            self.remote_loading = false;
            return;
        };
        let host_label = host.alias.clone();
        let remote_path = self.remote_path.clone();
        let show_hidden = self.config.files.show_hidden;
        self.remote_loading = true;
        self.remote_error = None;
        self.toast(ToastLevel::Info, format!("Opening {host_label}:{remote_path}"));
        self.queue_task(move || {
            let result = remote_fs::list_remote_host_with_hidden(&host, &remote_path, show_hidden)
                .map_err(|e| e.to_string());
            TaskResult::RemoteList { host_alias: host_label, path: remote_path, result }
        });
    }

    fn move_file_down(&mut self) {
        if self.active_file_pane == 0 && self.files_dual_pane {
            if self.local_selected + 1 < self.local_entries.len() {
                self.local_selected += 1;
            }
        } else if self.file_selected + 1 < self.remote_entries.len() {
            self.file_selected += 1;
        }
    }

    fn move_file_up(&mut self) {
        if self.active_file_pane == 0 && self.files_dual_pane {
            self.local_selected = self.local_selected.saturating_sub(1);
        } else {
            self.file_selected = self.file_selected.saturating_sub(1);
        }
    }

    fn open_selected_remote_entry(&mut self) {
        if self.active_file_pane == 0 && self.files_dual_pane {
            self.open_selected_local_entry();
            return;
        }
        let Some(entry) = self.remote_entries.get(self.file_selected).cloned() else { return; };
        if matches!(entry.kind, FileKind::Directory | FileKind::Symlink) {
            self.open_remote_path(entry.path);
        } else {
            self.preview_selected_remote_entry();
        }
    }

    fn open_selected_local_entry(&mut self) {
        let Some(entry) = self.local_entries.get(self.local_selected).cloned() else { return; };
        if matches!(entry.kind, FileKind::Directory | FileKind::Symlink) {
            self.local_path = entry.path;
            self.local_selected = 0;
            self.local_scroll = 0;
            self.refresh_local_files();
        } else {
            self.toast(ToastLevel::Info, format!("Selected local file: {}", entry.path));
        }
    }

    fn refresh_local_files(&mut self) {
        let expanded = local_fs::expand_tilde(&self.local_path);
        match local_fs::list_dir(&expanded, self.config.files.show_hidden) {
            Ok(entries) => {
                self.local_entries = entries;
                self.local_error = None;
                self.local_selected = self.local_selected.min(self.local_entries.len().saturating_sub(1));
            }
            Err(e) => {
                self.local_entries.clear();
                self.local_error = Some(e.to_string());
            }
        }
    }

    fn toggle_selected_file(&mut self) {
        let Some(entry) = self.remote_entries.get(self.file_selected) else { return; };
        if let Some(pos) = self.selected_file_paths.iter().position(|p| p == &entry.path) {
            self.selected_file_paths.remove(pos);
        } else {
            self.selected_file_paths.push(entry.path.clone());
        }
        self.selected_files = self.selected_file_paths.len();
    }

    fn preview_selected_remote_entry(&mut self) {
        let Some(entry) = self.remote_entries.get(self.file_selected).cloned() else { return; };
        if matches!(entry.kind, FileKind::Directory) {
            self.remote_preview = Some(format!("{}\n\nDirectory. Enter opens it.", entry.path));
            return;
        }
        if is_sensitive_path(&entry.path) {
            let expected = safety::sensitive_confirmation_phrase(&entry.path);
            self.ask_confirmation(
                "Preview sensitive file",
                format!("Type {expected} to preview this file. SSHDeck will not save it to logs."),
                expected,
                ConfirmAction::PreviewSensitive(entry.path),
            );
            return;
        }
        self.queue_remote_preview(entry.path);
    }

    fn queue_remote_preview(&mut self, path: String) {
        self.queue_remote_preview_with_confirmation(path, false);
    }

    fn queue_remote_preview_confirmed(&mut self, path: String) {
        self.queue_remote_preview_with_confirmation(path, true);
    }

    fn queue_remote_preview_with_confirmation(&mut self, path: String, confirmed: bool) {
        let Some(host) = self.current_host().cloned() else { return; };
        let max_bytes = self.config.files.preview_max_bytes;
        self.remote_preview = Some(format!("Loading preview for {path}..."));
        self.queue_task(move || {
            let result = if confirmed {
                remote_fs::preview_remote_host_confirmed(&host, &path, max_bytes)
            } else {
                remote_fs::preview_remote_host(&host, &path, max_bytes)
            }
            .map_err(|e| e.to_string());
            TaskResult::RemotePreview { path, result }
        });
    }

    fn queue_upload_selected(&mut self) {
        let Some(local) = self.local_entries.get(self.local_selected).cloned() else {
            self.toast(ToastLevel::Warning, "Pick a local file first.".into());
            return;
        };
        let remote_dest = remote_fs::join_remote_path(&self.remote_path, &local.name);
        self.start_transfer(TransferDirection::Upload, local.path, remote_dest);
    }

    fn queue_download_selected(&mut self) {
        let Some(remote) = self.remote_entries.get(self.file_selected).cloned() else {
            self.toast(ToastLevel::Warning, "Pick a remote file first.".into());
            return;
        };
        let local_dest = local_fs::expand_tilde(&self.local_path).join(&remote.name).display().to_string();
        self.start_transfer(TransferDirection::Download, remote.path, local_dest);
    }

    fn start_transfer(&mut self, direction: TransferDirection, source: String, destination: String) {
        let Some(host) = self.current_host().cloned() else { return; };
        let id = self.transfer_queue.enqueue(direction.clone(), source.clone(), destination.clone());
        self.transfer_queue.start(id);
        self.mode = Mode::Transfer;
        self.toast(ToastLevel::Info, format!("Queued transfer #{id}"));
        self.queue_task(move || {
            let args = match direction {
                TransferDirection::Upload => scp_upload_args_for(&host, &source, &destination),
                TransferDirection::Download => scp_download_args_for(&host, &source, &destination),
            };
            let display = display_command("scp", &args);
            let result = Command::new("scp")
                .args(&args)
                .output()
                .map_err(|e| e.to_string())
                .and_then(|out| {
                    if out.status.success() {
                        Ok(format!("Transfer finished: {display}"))
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                        Err(if stderr.is_empty() { format!("scp exited with {}", out.status) } else { stderr })
                    }
                });
            TaskResult::TransferFinished { id, result }
        });
    }

    fn confirm_delete_selected_file(&mut self) {
        let Some(entry) = self.remote_entries.get(self.file_selected).cloned() else { return; };
        let expected = safety::destructive_delete_confirmation_requirement(&entry.path)
            .phrase()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| entry.path.clone());
        self.ask_confirmation(
            "Delete remote file",
            format!("Type {expected} to delete it. This runs rm -rf on the remote host."),
            expected,
            ConfirmAction::DeleteRemoteFile(entry.path),
        );
    }

    fn open_file_command_template(&mut self, template: &str) {
        self.mode = Mode::Command;
        self.command_input = template.into();
    }

    fn add_bookmark_for_current_path(&mut self) {
        let host = self.current_host().map(|h| h.alias.clone()).unwrap_or_else(|| "global".into());
        let name = self.remote_path.trim_matches('/').rsplit('/').next().unwrap_or("remote").replace('~', "home");
        self.config.bookmarks.entry(host).or_default().insert(name, self.remote_path.clone());
        match self.config.save() {
            Ok(()) => self.toast(ToastLevel::Success, format!("Bookmarked {}", self.remote_path)),
            Err(e) => self.toast(ToastLevel::Error, format!("Could not save bookmark: {e}")),
        }
    }

    fn refresh_after_transfer(&mut self) {
        self.refresh_local_files();
        if self.view == View::Files {
            self.refresh_remote_files();
        }
    }

    fn ask_confirmation(
        &mut self,
        title: impl Into<String>,
        prompt: impl Into<String>,
        expected: impl Into<String>,
        action: ConfirmAction,
    ) {
        self.confirm = Some(ConfirmState {
            title: title.into(),
            prompt: prompt.into(),
            expected: expected.into(),
            input: String::new(),
            action,
        });
        self.mode = Mode::Confirm;
    }

    fn handle_confirm_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.confirm = None;
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                if let Some(confirm) = self.confirm.as_mut() {
                    confirm.input.pop();
                }
            }
            KeyCode::Enter => self.apply_confirmation()?,
            KeyCode::Char(c) => {
                if let Some(confirm) = self.confirm.as_mut() {
                    confirm.input.push(c);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_confirmation(&mut self) -> Result<()> {
        let Some(confirm) = self.confirm.clone() else { return Ok(()); };
        if confirm.input.trim() != confirm.expected {
            self.toast(ToastLevel::Warning, "Confirmation did not match.".into());
            return Ok(());
        }
        self.confirm = None;
        self.mode = Mode::Normal;
        match confirm.action {
            ConfirmAction::DeleteHost => self.delete_selected_host()?,
            ConfirmAction::DeleteRemoteFile(path) => self.run_remote_file_action(
                format!("Deleted {path}"),
                Some(self.remote_path.clone()),
                {
                    let confirmation = confirm.expected.clone();
                    move |host| remote_fs::delete_remote_confirmed(&host, &path, &confirmation)
                },
            ),
            ConfirmAction::RunCommand(cmd) => self.queue_remote_command(cmd),
            ConfirmAction::FileCommand(cmd) => self.execute_confirmed_file_command(cmd),
            ConfirmAction::PreviewSensitive(path) => self.queue_remote_preview_confirmed(path),
            ConfirmAction::SafeEdit(path) => self.safe_edit_remote_file(path),
        }
        Ok(())
    }

    fn run_remote_file_action<F>(&mut self, label: String, refresh_path: Option<String>, action: F)
    where
        F: FnOnce(SshHost) -> anyhow::Result<String> + Send + 'static,
    {
        let Some(host) = self.current_host().cloned() else { return; };
        self.toast(ToastLevel::Info, label.clone());
        self.queue_task(move || {
            let result = action(host).map_err(|e| e.to_string());
            TaskResult::RemoteAction { label, refresh_path, result }
        });
    }

    fn execute_file_command(&mut self, command: FileCommand) {
        match command {
            FileCommand::Cd(path) => self.open_remote_path(path),
            FileCommand::Mkdir(path) => self.run_remote_file_action(
                format!("Created folder {path}"),
                Some(self.remote_path.clone()),
                move |host| remote_fs::mkdir_remote(&host, &path),
            ),
            FileCommand::Touch(path) => self.run_remote_file_action(
                format!("Created file {path}"),
                Some(self.remote_path.clone()),
                move |host| remote_fs::touch_remote(&host, &path),
            ),
            FileCommand::Rename(old, new) => self.ask_confirmation(
                "Rename remote file",
                format!("Type {old} to rename it to {new}."),
                old.clone(),
                ConfirmAction::FileCommand(FileCommand::Rename(old, new)),
            ),
            FileCommand::DownloadSelected => self.queue_download_selected(),
            FileCommand::Upload(path) => {
                let name = PathBuf::from(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "upload".into());
                let destination = remote_fs::join_remote_path(&self.remote_path, &name);
                self.start_transfer(TransferDirection::Upload, path, destination);
            }
            FileCommand::Chmod(mode, path) => self.ask_confirmation(
                "Change remote permissions",
                format!("Type {path} to run chmod {mode}."),
                path.clone(),
                ConfirmAction::FileCommand(FileCommand::Chmod(mode, path)),
            ),
            FileCommand::Chown(owner, path) => self.ask_confirmation(
                "Change remote owner",
                format!("Type {path} to run chown {owner}."),
                path.clone(),
                ConfirmAction::FileCommand(FileCommand::Chown(owner, path)),
            ),
            FileCommand::Open => self.open_selected_remote_entry(),
            FileCommand::Edit => {
                if let Some(entry) = self.remote_entries.get(self.file_selected) {
                    self.ask_confirmation(
                        "Edit remote file",
                        format!("Type EDIT {} to download, edit, back up, and upload this file.", entry.path),
                        format!("EDIT {}", entry.path),
                        ConfirmAction::SafeEdit(entry.path.clone()),
                    );
                }
            }
            FileCommand::CopyPath => {
                if let Some(entry) = self.remote_entries.get(self.file_selected) {
                    self.toast(ToastLevel::Info, format!("Path: {}", entry.path));
                }
            }
            FileCommand::BookmarkAdd(name) => {
                let host = self.current_host().map(|h| h.alias.clone()).unwrap_or_else(|| "global".into());
                self.config.bookmarks.entry(host).or_default().insert(name, self.remote_path.clone());
                let _ = self.config.save();
                self.toast(ToastLevel::Success, format!("Bookmarked {}", self.remote_path));
            }
            FileCommand::BookmarkJump(name) => {
                let host = self.current_host().map(|h| h.alias.clone()).unwrap_or_else(|| "global".into());
                if let Some(path) = crate::files::bookmarks::bookmarks_for(&self.config, &host).get(&name).cloned() {
                    self.open_remote_path(path);
                } else {
                    self.toast(ToastLevel::Warning, format!("No bookmark named {name}"));
                }
            }
            FileCommand::Unknown(input) => self.toast(ToastLevel::Warning, format!("Unknown file command: {input}")),
        }
    }

    fn execute_confirmed_file_command(&mut self, command: FileCommand) {
        match command {
            FileCommand::Rename(old, new) => self.run_remote_file_action(
                format!("Renamed {old}"),
                Some(self.remote_path.clone()),
                move |host| remote_fs::rename_remote(&host, &old, &new),
            ),
            FileCommand::Chmod(mode, path) => self.run_remote_file_action(
                format!("Changed permissions for {path}"),
                Some(self.remote_path.clone()),
                move |host| remote_fs::chmod_remote(&host, &mode, &path),
            ),
            FileCommand::Chown(owner, path) => self.run_remote_file_action(
                format!("Changed owner for {path}"),
                Some(self.remote_path.clone()),
                move |host| remote_fs::chown_remote(&host, &owner, &path),
            ),
            other => self.execute_file_command(other),
        }
    }

    fn safe_edit_remote_file(&mut self, path: String) {
        let Some(host) = self.current_host().cloned() else { return; };
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
        let tx_path = self.remote_path.clone();
        self.toast(ToastLevel::Info, format!("Opening editor for {path}"));
        self.queue_task(move || {
            let result = remote_fs::safe_edit_with_openssh(host, path.clone(), editor)
                .map(|_| format!("Edited {path}"))
                .map_err(|e| e.to_string());
            TaskResult::RemoteAction { label: "Safe edit finished".into(), refresh_path: Some(tx_path), result }
        });
    }

    fn run_current_command(&mut self) {
        let cmd = if self.command_input.trim().is_empty() {
            "uptime".to_string()
        } else {
            self.command_input.trim().to_string()
        };
        self.submit_remote_command(cmd);
    }

    fn submit_remote_command(&mut self, cmd: String) {
        if cmd.trim().is_empty() {
            self.toast(ToastLevel::Warning, "Type a command first.".into());
            return;
        }
        if is_dangerous_command(&cmd) {
            let expected = format!("RUN {}", cmd);
            self.ask_confirmation(
                "Run dangerous command",
                format!("Type {expected} to run this command on the selected host."),
                expected,
                ConfirmAction::RunCommand(cmd),
            );
        } else {
            self.queue_remote_command(cmd);
        }
    }

    fn queue_remote_command(&mut self, cmd: String) {
        let Some(host) = self.current_host().cloned() else { return; };
        self.view = View::CommandRunner;
        self.command_input = cmd.clone();
        self.command_output = format!("$ {cmd}\nRunning...");
        self.toast(ToastLevel::Info, format!("Running: {cmd}"));
        self.queue_task(move || {
            let result = run_ssh_command_for(&host, &cmd, Duration::from_secs(20), 64 * 1024)
                .map_err(|e| e.to_string());
            TaskResult::RemoteCommand { command: cmd, result }
        });
    }

    fn toggle_tunnel(&mut self) {
        if self.active_tunnel.is_some() {
            self.stop_tunnel();
        } else {
            self.start_tunnel();
        }
    }

    fn start_tunnel(&mut self) {
        if let Some(host)=self.current_host(){
            if self.tunnel.host_alias.trim().is_empty() {
                self.tunnel.host_alias = host.alias.clone();
            }
        }
        if let Err(e) = self.tunnel.validate() {
            self.toast(ToastLevel::Warning, e);
            return;
        }
        let command = self
            .hosts
            .iter()
            .find(|host| host.alias == self.tunnel.host_alias)
            .map(|host| self.tunnel.ssh_command_for_host(host))
            .unwrap_or_else(|| self.tunnel.ssh_command());
        let display = command.display();
        match Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => {
                self.active_tunnel = Some(TunnelSession { command: display.clone(), child });
                self.toast(ToastLevel::Success, format!("Tunnel running: {display}"));
            }
            Err(e) => self.toast(ToastLevel::Error, format!("Could not start tunnel: {e}")),
        }
    }

    fn stop_tunnel(&mut self) {
        let Some(mut session) = self.active_tunnel.take() else {
            self.toast(ToastLevel::Info, "No tunnel is running.".into());
            return;
        };
        let _ = session.child.kill();
        let _ = session.child.wait();
        self.toast(ToastLevel::Info, format!("Stopped tunnel: {}", session.command));
    }

    fn copy_tunnel_command(&mut self) {
        self.toast(ToastLevel::Info, format!("Tunnel command: {}", self.tunnel.command()));
    }

    fn record_recent_host(&mut self, alias: &str) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        if let Some(host) = self.hosts.iter_mut().find(|h| h.alias == alias) {
            host.recent_connection = Some(now);
        }
        self.config.recent_hosts.retain(|h| h != alias);
        self.config.recent_hosts.insert(0, alias.to_string());
        self.config.recent_hosts.truncate(20);
        let _ = self.config.save();
    }
    fn open_host_context(&mut self) { if let Some(h)=self.current_host(){ let title=h.alias.clone(); self.context_menu_selected=0; self.context_menu=Some(ContextMenu{title, items:vec![ ("Connect".into(),ClickTarget::HostConnectButton(self.current_host_index().unwrap_or(0))), ("Files".into(),ClickTarget::HostFilesButton(self.current_host_index().unwrap_or(0))), ("Tunnel".into(),ClickTarget::HostTunnelButton(self.current_host_index().unwrap_or(0))), ("Run Command".into(),ClickTarget::StatusShortcut("run".into())), ("Health".into(),ClickTarget::HostHealthButton(self.current_host_index().unwrap_or(0))), ("Edit".into(),ClickTarget::HostEditButton(self.current_host_index().unwrap_or(0))), ("Delete".into(),ClickTarget::ModalButton("delete-host".into())) ]}); } }
    fn open_file_context(&mut self, path:String) { self.context_menu_selected=0; self.context_menu=Some(ContextMenu{title:path.clone(),items:vec![("Preview".into(),ClickTarget::StatusShortcut("preview".into())),("Edit safely".into(),ClickTarget::StatusShortcut("edit-file".into())),("Download".into(),ClickTarget::StatusShortcut("download".into())),("Rename".into(),ClickTarget::StatusShortcut("rename".into())),("Copy Path".into(),ClickTarget::Breadcrumb(path)),("Delete".into(),ClickTarget::StatusShortcut("delete-file".into()))]}); }

    fn move_context_menu(&mut self, delta: isize) {
        let Some(menu) = self.context_menu.as_ref() else { return; };
        let len = menu.items.len();
        if len == 0 { return; }
        if delta < 0 {
            self.context_menu_selected = self.context_menu_selected.saturating_sub(1);
        } else {
            self.context_menu_selected = (self.context_menu_selected + 1).min(len - 1);
        }
    }

    fn activate_context_menu_selected(&mut self) -> Result<()> {
        let Some(menu) = self.context_menu.as_ref() else { return Ok(()); };
        let Some((_, target)) = menu.items.get(self.context_menu_selected).cloned() else { return Ok(()); };
        self.context_menu = None;
        self.dispatch_click(target)
    }

    fn activate_status_shortcut(&mut self, shortcut: &str) -> Result<()> {
        match shortcut.to_ascii_lowercase().as_str() {
            "/" => { self.mode = Mode::Search; self.search.clear(); },
            "?" => self.view = View::Help,
            "a" => self.open_host_form(HostFormMode::Add),
            "enter" => self.connect_selected()?,
            "s" => self.open_files_home(),
            "t" => { if let Some(h)=self.current_host(){ self.tunnel.host_alias=h.alias.clone(); } self.view=View::Tunnels; },
            "r" | ":" => { self.view=View::CommandRunner; self.command_input="uptime".into(); },
            "run" => { self.view=View::CommandRunner; self.run_current_command(); },
            "h" => self.fetch_health(),
            "l" => { self.logs=storage::read_logs(); self.view=View::Logs; },
            "," | "settings" => { self.view=View::Settings; self.settings_selected=0; },
            "esc" => { self.context_menu=None; self.host_form=None; self.mode=Mode::Normal; if self.view != View::Dashboard { self.view=View::Dashboard; } },
            "tab" => { if self.view==View::Files { if self.files_dual_pane { self.active_file_pane=1-self.active_file_pane; } else { self.files_dual_pane=true; } } },
            "ctrl+p" => { self.mode=Mode::Palette; self.palette_input.clear(); self.palette_selected=0; },
            "preview" => self.preview_selected_remote_entry(),
            "download" => self.queue_download_selected(),
            "delete-file" => self.confirm_delete_selected_file(),
            "edit-file" => {
                if let Some(entry) = self.remote_entries.get(self.file_selected) {
                    self.ask_confirmation(
                        "Edit remote file",
                        format!("Type EDIT {} to download, edit, back up, and upload this file.", entry.path),
                        format!("EDIT {}", entry.path),
                        ConfirmAction::SafeEdit(entry.path.clone()),
                    );
                }
            }
            "rename" => {
                if let Some(entry) = self.remote_entries.get(self.file_selected) {
                    self.mode = Mode::Command;
                    self.command_input = format!("rename {} {}", entry.path, entry.path);
                }
            }
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
        } else if action.contains("setting") {
            self.view = View::Settings;
        } else if action.contains("file") || action == "s" {
            self.open_files_home();
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
    fn test_host_form(&mut self){
        if !self.validate_current_form(){ return; }
        let Some(form)=self.host_form.as_mut() else { return; };
        let Some(host)=form.draft.to_host() else {
            form.test_result=Some("Connection test could not build an SSH command from this form".into());
            return;
        };
        let args=ssh_test_args_for(&host, 5);
        let command=display_command("ssh", &args);
        form.test_result=Some(format!("Testing...\n{command}"));
        self.queue_task(move || {
            let status=Command::new("ssh").args(&args).status();
            let result=match status {
                Ok(s) if s.success()=> "Connection successful".into(),
                Ok(s)=>format!("Connection failed: ssh exited with {s}"),
                Err(e)=>format!("Connection failed: {e}"),
            };
            TaskResult::HostTest { command, result }
        });
    }
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
    fn confirm_delete_host(&mut self){
        if let Some(h)=self.current_host(){
            let expected = h.alias.clone();
            self.ask_confirmation(
                format!("Remove {}?", h.alias),
                format!("Type {expected}. Managed hosts are removed from SSHDeck's managed config; imported hosts are hidden from this view."),
                expected,
                ConfirmAction::DeleteHost,
            );
        }
    }
    fn delete_selected_host(&mut self)->Result<()> {
        let Some(idx)=self.current_host_index() else { return Ok(()); };
        let alias=self.hosts[idx].alias.clone();
        if self.managed_aliases.contains(&alias){
            self.hosts.remove(idx);
            self.managed_aliases.retain(|a|a!=&alias);
            let managed:Vec<_>=self.hosts.iter().filter(|h| self.managed_aliases.contains(&h.alias)).cloned().collect();
            managed_hosts::save_managed_hosts(&managed_hosts::managed_config_path(), &managed)?;
        } else {
            self.hide_aliases.push(alias.clone());
            if !self.config.hidden_imported_hosts.contains(&alias) {
                self.config.hidden_imported_hosts.push(alias.clone());
            }
            self.hosts.remove(idx);
        }
        self.config.hosts.remove(&alias);
        self.config.save()?;
        self.filtered=(0..self.hosts.len()).collect();
        self.selected=self.selected.min(self.filtered.len().saturating_sub(1));
        self.context_menu=None;
        self.toast(ToastLevel::Warning,format!("Removed {alias} from SSHDeck view"));
        Ok(())
    }
    fn toggle_theme(&mut self){ self.config.ui.theme=match self.config.ui.theme.as_str(){"blackout"=>"minimal".into(),"minimal"=>"cyber".into(),_=>"blackout".into()}; self.theme=Theme::named(&self.config.ui.theme); let _=self.config.save(); self.toast(ToastLevel::Info,format!("Theme: {}",self.config.ui.theme)); }
    pub fn settings_ids() -> [&'static str; 8] { ["theme", "animations", "unicode", "nerd_font", "mouse", "show_hidden", "default_local_dir", "config_path"] }
    fn move_setting_down(&mut self) { self.settings_selected = (self.settings_selected + 1).min(Self::settings_ids().len().saturating_sub(1)); }
    fn move_setting_up(&mut self) { self.settings_selected = self.settings_selected.saturating_sub(1); }
    fn activate_selected_setting(&mut self) -> Result<()> { let id=Self::settings_ids().get(self.settings_selected).copied().unwrap_or("theme"); self.activate_setting(id) }
    pub fn activate_setting(&mut self, id:&str) -> Result<()> { match id { "theme" => self.toggle_theme(), "animations" => { self.config.ui.animations=!self.config.ui.animations; self.animator.enabled=self.config.ui.animations; self.toast(ToastLevel::Info,format!("animations {}", if self.config.ui.animations{"on"}else{"off"})); }, "unicode" => { self.config.ui.unicode=!self.config.ui.unicode; self.ascii=!self.config.ui.unicode; self.toast(ToastLevel::Info,format!("unicode {}", if self.config.ui.unicode{"on"}else{"off"})); }, "nerd_font" => { self.config.ui.nerd_font=!self.config.ui.nerd_font; self.toast(ToastLevel::Info,format!("nerd font {}", if self.config.ui.nerd_font{"on"}else{"off"})); }, "mouse" => { self.config.ui.mouse=!self.config.ui.mouse; self.mouse_enabled=self.config.ui.mouse; let mut stdout=std::io::stdout(); if self.mouse_enabled { let _=crossterm::execute!(stdout, EnableMouseCapture); } else { let _=crossterm::execute!(stdout, DisableMouseCapture); } self.toast(ToastLevel::Info,format!("mouse {}", if self.config.ui.mouse{"on"}else{"off"})); }, "show_hidden" => { self.config.files.show_hidden=!self.config.files.show_hidden; self.refresh_local_files(); if self.view==View::Files { self.refresh_remote_files(); } self.toast(ToastLevel::Info,format!("hidden files {}", if self.config.files.show_hidden{"shown"}else{"hidden"})); }, "default_local_dir" => self.toast(ToastLevel::Info,format!("Default local dir: {}", self.config.files.default_local_dir)), "config_path" => self.toast(ToastLevel::Info,format!("Config: {}", self.config.path.display())), _=>{} } self.config.save()?; Ok(()) }
    fn add_include_line(&mut self)->Result<()> { let path=dirs::home_dir().unwrap_or_default().join(".ssh/config"); let line="Include ~/.config/sshdeck/ssh_config"; let changed=managed_hosts::ensure_include_line(&path,line)?; self.context_menu=None; self.toast(ToastLevel::Success, if changed {format!("Added {line} to {} with backup", path.display())} else {"Include line already present".into()}); Ok(()) }

    fn handle_search_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> { match key.code { KeyCode::Esc => { self.search.clear(); self.mode=Mode::Normal; self.filter_hosts(); }, KeyCode::Enter => self.mode=Mode::Normal, KeyCode::Backspace => { self.search.pop(); self.filter_hosts(); }, KeyCode::Char(c) => { self.search.push(c); self.filter_hosts(); }, _=>{} } Ok(()) }
    fn handle_palette_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> {
        const ACTIONS: [&str; 10] = ["Add host","Open files","Build tunnel","Run command","Check health","Duplicate host","Toggle theme","Open settings","Show Include command","Quit"];
        match key.code {
            KeyCode::Esc => self.mode=Mode::Normal,
            KeyCode::Down | KeyCode::Char('j') => self.palette_selected = (self.palette_selected + 1).min(ACTIONS.len() - 1),
            KeyCode::Up | KeyCode::Char('k') => self.palette_selected = self.palette_selected.saturating_sub(1),
            KeyCode::Enter => {
                let q=if self.palette_input.trim().is_empty() {
                    ACTIONS[self.palette_selected].to_string()
                } else {
                    self.palette_input.clone()
                };
                self.run_palette_action(&q)?;
            }
            KeyCode::Backspace=>{self.palette_input.pop();},
            KeyCode::Char(c)=>self.palette_input.push(c),
            _=>{}
        }
        Ok(())
    }
    fn handle_command_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> {
        match key.code {
            KeyCode::Esc => self.mode=Mode::Normal,
            KeyCode::Backspace => { self.command_input.pop(); },
            KeyCode::Enter => {
                let cmd=self.command_input.trim().to_string();
                self.mode=Mode::Normal;
                if self.view == View::Files {
                    self.execute_file_command(parse_command(&cmd));
                } else {
                    self.submit_remote_command(cmd);
                }
            }
            KeyCode::Char(c)=>self.command_input.push(c),
            _=>{}
        }
        Ok(())
    }

    pub fn connect_selected(&mut self)->Result<()> {
        if let Some(h)=self.current_host().cloned(){
            let alias=h.alias.clone();
            let mut args = if self.managed_aliases.contains(&alias) {
                ssh_args_for(&h)
            } else {
                // Imported OpenSSH hosts should connect by alias so ssh can use
                // the user's complete ~/.ssh/config block, including options
                // SSHDeck does not parse yet (ControlMaster, Match, Include,
                // RequestTTY, SetEnv, etc.). This is faster and more faithful
                // than reconstructing a partial command from parsed fields.
                vec!["--".into(), alias.clone()]
            };
            if !args.iter().any(|a| a == "-t" || a == "-tt" || a == "-T") {
                args.insert(0, "-t".into());
            }
            let cmd=display_command("ssh", &args); self.toast(ToastLevel::Info,format!("Connecting: {cmd}"));
            crossterm::terminal::disable_raw_mode()?;
            let mut stdout = std::io::stdout();
            if self.mouse_enabled { crossterm::execute!(stdout, Show, DisableMouseCapture, crossterm::terminal::LeaveAlternateScreen)?; } else { crossterm::execute!(stdout, Show, crossterm::terminal::LeaveAlternateScreen)?; }
            session_frame::enter_session_frame(&mut stdout, &alias, self.ascii)?;
            let status=Command::new("ssh").args(&args).status();
            session_frame::leave_session_frame(&mut stdout)?;
            if self.mouse_enabled { crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0), Hide, EnableMouseCapture)?; } else { crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0), Hide)?; }
            crossterm::terminal::enable_raw_mode()?;
            self.render_reset_needed = true;
            match status {
                Ok(s) if s.success()=>{
                    self.record_recent_host(&alias);
                    self.toast(ToastLevel::Success,format!("Returned from {alias}"));
                }
                Ok(s)=>self.toast(ToastLevel::Warning,format!("SSH exited with {s}: {cmd}")),
                Err(e)=>self.toast(ToastLevel::Error,format!("Could not start ssh: {e}")),
            }
        }
        Ok(())
    }
    pub fn fetch_health(&mut self){
        let Some(host)=self.current_host().cloned() else { return; };
        let alias = host.alias.clone();
        self.health.uptime=format!("{alias} health check running");
        self.view=View::HostDetail;
        self.toast(ToastLevel::Info,"Checking uptime, disk, memory, services, and Docker.".into());
        self.queue_task(move || {
            let result = crate::ssh::health::run_remote_health(&host, Duration::from_secs(20))
                .map_err(|e| e.to_string());
            TaskResult::Health { host_alias: alias, result }
        });
    }
}

fn add_scroll(v:usize, delta:i16)->usize { if delta < 0 { v.saturating_sub((-delta) as usize) } else { v.saturating_add(delta as usize) } }
