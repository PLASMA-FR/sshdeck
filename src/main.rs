#![allow(dead_code, unused_imports)]

mod animation;
mod app;
mod config;
mod design;
mod event;
mod files;
mod mouse;
mod ssh;
mod theme;
mod ui;
mod views;
mod widgets;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, event::{EnableMouseCapture, DisableMouseCapture}};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppOptions};
use config::app_config::AppConfig;
use config::ssh_config::parse_default_ssh_config;
use ssh::health::DoctorReport;

#[derive(Parser, Debug)]
#[command(name = "sshdeck", version, about = "Termius for the terminal. No cloud. No account. No Electron.")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    no_animations: bool,
    #[arg(long)]
    no_mouse: bool,
    #[arg(long)]
    mouse: bool,
    #[arg(long)]
    ascii: bool,
    #[arg(long)]
    quick: Option<String>,
    #[arg(value_name = "TARGET")]
    target: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Import,
    Doctor,
}

fn main() -> Result<()> {
    color_eyre::install().ok();
    let cli = Cli::parse();
    if let Some(target) = cli.quick.clone().or_else(|| cli.target.clone()) {
        println!("SSHDeck quick connect: {target}");
        let status = std::process::Command::new("ssh").arg(&target).status()?;
        if !status.success() { std::process::exit(status.code().unwrap_or(1)); }
        return Ok(());
    }
    match cli.command {
        Some(Commands::Doctor) => {
            let cfg = AppConfig::load_or_default(cli.config.clone())?;
            let report = DoctorReport::run(&cfg);
            println!("{}", report.render_text());
            Ok(())
        }
        Some(Commands::Import) => {
            let hosts = parse_default_ssh_config().unwrap_or_default();
            let cfg = AppConfig::load_or_default(cli.config.clone())?;
            cfg.save()?;
            println!("Imported {} host(s) from ~/.ssh/config", hosts.len());
            println!("SSHDeck config: {}", cfg.path.display());
            Ok(())
        }
        None => run_tui(cli),
    }
}

fn run_tui(cli: Cli) -> Result<()> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    let config = AppConfig::load_or_default(cli.config.clone())?;
    let mouse_enabled = (cli.mouse || config.ui.mouse) && !cli.no_mouse;
    if mouse_enabled { execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; } else { execute!(stdout, EnterAlternateScreen)?; }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = {
        let mut app = App::new(config, AppOptions { no_animations: cli.no_animations, ascii: cli.ascii, mouse: mouse_enabled })?;
        app.run(&mut terminal)
    };
    disable_raw_mode()?;
    if mouse_enabled { execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?; } else { execute!(terminal.backend_mut(), LeaveAlternateScreen)?; }
    terminal.show_cursor()?;
    result
}
