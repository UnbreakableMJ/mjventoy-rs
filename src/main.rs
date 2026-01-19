// SPDX-License-Identifier: GPL-3.0-or-later
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

mod theme;
mod auth;
mod discovery;
mod executor;
mod tui;


use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::sync::mpsc;
use std::time::Duration;
use crate::tui::app::App;
use crate::tui::ui::render;
use crate::tui::input::handle_key;
use crate::executor::{InstallationOptions, PartitionStyle, install_ventoy};


#[derive(Parser)]
#[command(name = "MJVentoy")]
#[command(about = "Industrial MJVentoy Installer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to original Ventoy directory
    #[arg(short, long, default_value = "/home/mj/mjventoy/Ventoy")]
    ventoy: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Install Ventoy to a device (CLI mode)
    Install {
        /// Target device (e.g. /dev/sdb)
        device: String,
        /// Use GPT instead of MBR
        #[arg(short, long)]
        gpt: bool,
        /// Disable secure boot
        #[arg(short = 's', long)]
        no_secure_boot: bool,
        /// Reserved space in MiB
        #[arg(short, long, default_value = "0")]
        reserve: u64,
        /// Dry run mode (don't perform destructive actions)
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// List available devices
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install { device, gpt, no_secure_boot, reserve, dry_run }) => {
            let opts = InstallationOptions {
                device,
                part_style: if gpt { PartitionStyle::GPT } else { PartitionStyle::MBR },
                secure_boot: !no_secure_boot,
                reserve_space_mb: reserve,
                ventoy_path: cli.ventoy,
                dry_run,
            };

            println!("Starting CLI Installation on {}...", opts.device);
            install_ventoy(opts)?;
            println!("Installation finished successfully!");
            Ok(())
        }
        Some(Commands::List) => {
            let disks = discovery::get_disks()?;
            println!("Available Disks:");
            for disk in disks {
                println!("- {} ({}): {}", disk.name, disk.model.as_deref().unwrap_or("Unknown"), disk.size);
            }
            Ok(())
        }
        None => {
            // Default to TUI mode
            run_tui(cli.ventoy)
        }
    }
}

fn run_tui(ventoy_path: String) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and run
    let res = run_app(&mut terminal, ventoy_path);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, ventoy_path: String) -> Result<()>
where
    B::Error: std::error::Error + Send + Sync + 'static,
{
    let (tx, rx) = mpsc::channel();
    let mut app = App::new();
    app.ventoy_path = ventoy_path;
    app.tx = Some(tx); // Give app the sender
    
    // Initial data fetch
    app.auth_tools = auth::get_available_tools();
    if auth::is_root() {
        app.state = crate::tui::app::AppState::DeviceSelection;
        app.devices = discovery::get_disks()?;
    }

    loop {
        terminal.draw(|f| render(f, &app))?;

        // Check for background messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                tui::app::AppMessage::InstallationFinished(res) => {
                    match res {
                        Ok(_) => {
                            app.progress = 1.0;
                            app.logs.push("Installation finished successfully!".to_string());
                        }
                        Err(e) => {
                            app.logs.push(format!("Installation failed: {}", e));
                            app.state = crate::tui::app::AppState::Configuration;
                        }
                    }
                }
                tui::app::AppMessage::ProgressUpdate(p) => {
                    app.progress = p;
                }
            }
        }

        // Non-blocking event poll
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
                handle_key(key, &mut app);
                
                if matches!(app.state, crate::tui::app::AppState::DeviceSelection) && app.devices.is_empty() {
                    app.devices = discovery::get_disks().unwrap_or_default();
                }
            }
        }
    }
}


