use std::{process::Command, time::Duration};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{animation::Animator, config::{app_config::AppConfig, ssh_config::parse_default_ssh_config, storage}, event::{Event, EventLoop}, files::transfer::TransferQueue, ssh::{command::{is_dangerous_command, ssh_command_for}, health::HealthInfo, host::SshHost, tunnel::{TunnelConfig, TunnelType}}, theme::Theme, ui};

#[derive(Debug, Clone, Copy)] pub struct AppOptions { pub no_animations: bool, pub ascii: bool }
#[derive(Debug, Clone, PartialEq, Eq)] pub enum View { Dashboard, HostDetail, Files, Tunnels, CommandRunner, Logs, Settings, Help }
#[derive(Debug, Clone, PartialEq, Eq)] pub enum Mode { Normal, Search, Visual, Command, Rename, Confirm, Transfer, Palette }
#[derive(Debug, Clone)] pub struct Toast { pub message:String, pub ttl:u8, pub level: ToastLevel }
#[derive(Debug, Clone)] pub enum ToastLevel { Success, Warning, Error, Info }

pub struct App {
    pub config: AppConfig,
    pub hosts: Vec<SshHost>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub view: View,
    pub mode: Mode,
    pub search: String,
    pub command_input: String,
    pub palette_input: String,
    pub logs: Vec<String>,
    pub theme: Theme,
    pub animator: Animator,
    pub ascii: bool,
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
}

impl App {
    pub fn new(config: AppConfig, options: AppOptions) -> Result<Self> {
        let mut hosts = parse_default_ssh_config().unwrap_or_default();
        for h in &mut hosts { if let Some(meta)=config.hosts.get(&h.alias){ h.tags=meta.tags.clone(); h.group=meta.group.clone(); h.favorite=meta.favorite; h.notes=meta.notes.clone(); } }
        let theme = Theme::named(&config.ui.theme);
        let mut app=Self{ filtered:(0..hosts.len()).collect(), hosts, selected:0, view:View::Dashboard, mode:Mode::Normal, search:String::new(), command_input:String::new(), palette_input:String::new(), logs:storage::read_logs(), theme, animator:Animator::new(config.ui.animations && !options.no_animations), ascii:options.ascii || !config.ui.unicode, should_quit:false, toast:None, health:HealthInfo::empty(), tunnel:TunnelConfig{tunnel_type:TunnelType::Local, host_alias:String::new(), bind_address:None, local_port:8080, target_host:Some("localhost".into()), target_port:Some(80)}, remote_path:"~".into(), local_path:config.files.default_local_dir.clone(), files_dual_pane:false, active_file_pane:1, selected_files:0, transfer_queue:TransferQueue::default(), command_output:String::new(), config };
        app.toast(ToastLevel::Success, format!("Imported {} host(s)", app.hosts.len()));
        Ok(app)
    }
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> { let mut events=EventLoop::new(Duration::from_millis(80)); while !self.should_quit { terminal.draw(|f| ui::draw(f, self))?; match events.next()? { Event::Tick=>self.on_tick(), Event::Resize(_,_)=>{}, Event::Key(k)=>self.handle_key(k)? } } Ok(()) }
    pub fn current_host(&self)->Option<&SshHost>{ self.filtered.get(self.selected).and_then(|i| self.hosts.get(*i)) }
    pub fn on_tick(&mut self){ self.animator.tick(); if let Some(t)=self.toast.as_mut(){ if t.ttl>0 { t.ttl-=1; } else { self.toast=None; } } }
    pub fn toast(&mut self, level:ToastLevel, message:String){ storage::append_log(&message); self.logs.push(format!("{}", message)); self.toast=Some(Toast{message,ttl:40,level}); }
    fn filter_hosts(&mut self){ if self.search.is_empty(){ self.filtered=(0..self.hosts.len()).collect(); } else { let m=SkimMatcherV2::default(); let mut scored:Vec<(i64,usize)>=self.hosts.iter().enumerate().filter_map(|(i,h)| m.fuzzy_match(&h.search_blob(), &self.search).map(|s|(s,i))).collect(); scored.sort_by(|a,b| b.0.cmp(&a.0)); self.filtered=scored.into_iter().map(|(_,i)|i).collect(); } self.selected=self.selected.min(self.filtered.len().saturating_sub(1)); }
    fn move_down(&mut self){ if self.selected + 1 < self.filtered.len(){ self.selected+=1; } }
    fn move_up(&mut self){ self.selected=self.selected.saturating_sub(1); }
    pub fn handle_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> {
        if self.mode==Mode::Search { return self.handle_search_key(key); }
        if self.mode==Mode::Palette { return self.handle_palette_key(key); }
        if self.mode==Mode::Command { return self.handle_command_key(key); }
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
            KeyCode::Char('s') => { self.view=View::Files; self.remote_path="~".into(); },
            KeyCode::Char('t') => { if let Some(h)=self.current_host(){ self.tunnel.host_alias=h.alias.clone(); } self.view=View::Tunnels; },
            KeyCode::Char('r') => { self.view=View::CommandRunner; self.command_input="uptime".into(); },
            KeyCode::Char('h') => self.fetch_health(),
            KeyCode::Char('l') => { self.logs=storage::read_logs(); self.view=View::Logs; },
            KeyCode::Char('a') => self.toast(ToastLevel::Info,"Add host: create managed entry in ~/.config/sshdeck/ssh_config (MVP placeholder)".into()),
            KeyCode::Char('e') => self.toast(ToastLevel::Info,"Edit host metadata in ~/.config/sshdeck/config.toml".into()),
            KeyCode::Char('d') => self.toast(ToastLevel::Warning,"Delete requires confirmation; not destructive in MVP".into()),
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
    fn handle_search_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> { match key.code { KeyCode::Esc => { self.search.clear(); self.mode=Mode::Normal; self.filter_hosts(); }, KeyCode::Enter => self.mode=Mode::Normal, KeyCode::Backspace => { self.search.pop(); self.filter_hosts(); }, KeyCode::Char(c) => { self.search.push(c); self.filter_hosts(); }, _=>{} } Ok(()) }
    fn handle_palette_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> { match key.code { KeyCode::Esc => self.mode=Mode::Normal, KeyCode::Enter => { let q=self.palette_input.to_ascii_lowercase(); self.mode=Mode::Normal; if q.contains("file"){self.view=View::Files}else if q.contains("tunnel"){self.view=View::Tunnels}else if q.contains("health"){self.fetch_health()}else if q.contains("quit"){self.should_quit=true}else{self.toast(ToastLevel::Info,"Command palette action selected".into())} }, KeyCode::Backspace=>{self.palette_input.pop();}, KeyCode::Char(c)=>self.palette_input.push(c), _=>{} } Ok(()) }
    fn handle_command_key(&mut self, key:crossterm::event::KeyEvent)->Result<()> { match key.code { KeyCode::Esc=>self.mode=Mode::Normal, KeyCode::Backspace=>{self.command_input.pop();}, KeyCode::Enter=>{ let cmd=self.command_input.clone(); self.mode=Mode::Normal; if is_dangerous_command(&cmd){ self.toast(ToastLevel::Warning,"Dangerous command blocked pending explicit confirmation".into()); } else { self.command_output=format!("$ {}
(command mode parsed; remote execution uses ssh in command runner)", cmd); self.toast(ToastLevel::Success,format!("Command accepted: {cmd}")); } }, KeyCode::Char(c)=>self.command_input.push(c), _=>{} } Ok(()) }
    pub fn connect_selected(&mut self)->Result<()> { if let Some(h)=self.current_host(){ let alias=h.alias.clone(); let cmd=ssh_command_for(h); self.toast(ToastLevel::Info,format!("Connecting: {cmd}")); crossterm::terminal::disable_raw_mode()?; crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?; let _=Command::new("ssh").arg(&alias).status(); crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?; crossterm::terminal::enable_raw_mode()?; self.toast(ToastLevel::Success,format!("Returned from {alias}")); } Ok(()) }
    pub fn fetch_health(&mut self){ if let Some(h)=self.current_host(){ self.health.uptime=format!("{} health check queued", h.alias); self.view=View::HostDetail; self.toast(ToastLevel::Info,"Health commands: uptime, df -h, free -h, uname -a".into()); } }
}
