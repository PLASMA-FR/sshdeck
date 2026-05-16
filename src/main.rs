#![allow(dead_code, unused_imports)]

mod animation;
mod app;
mod config;
mod event;
mod files;
mod ssh;
mod theme;
mod ui;
mod views;
mod widgets;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
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
    ascii: bool,
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
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = {
        let config = AppConfig::load_or_default(cli.config)?;
        let mut app = App::new(config, AppOptions { no_animations: cli.no_animations, ascii: cli.ascii })?;
        app.run(&mut terminal)
    };
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
