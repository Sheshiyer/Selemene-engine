//! Noesis TUI — Terminal User Interface for Selemene Engine
//!
//! Interactive terminal interface for running consciousness calculations,
//! managing profiles, and exploring engine/workflow results.

mod app;
mod screens;
mod utils;
mod widgets;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to file (don't pollute terminal)
    let log_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".noesis")
        .join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let log_file = std::fs::File::create(log_dir.join("tui.log")).ok();
    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_writer(file)
            .with_env_filter("noesis_tui=debug,noesis_sdk=debug")
            .init();
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new().await;
    let result = app.run(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Report errors after terminal is restored
    if let Err(ref e) = result {
        eprintln!("Error: {e:?}");
    }

    result
}
