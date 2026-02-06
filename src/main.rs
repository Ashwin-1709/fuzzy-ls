extern crate clap;
mod app;
mod editor;
mod search;
mod ui;

use clap::{ArgAction, Parser};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::collections::BTreeSet;
use std::io;

#[derive(Parser, Debug)]
#[clap(
    name = "fuzzy-ls",
    version = "1.3.0",
    about = "Fuzzy file search TUI.",
    author = "Ashwin Pugalia"
)]
struct Cli {
    /// Initial query string.
    #[clap(help = "Initial query used for the search.")]
    query: Option<String>,

    /// Exclude files of specific extensions.
    #[clap(
        short = 'e',
        long,
        help = "Exclude files of specific extensions.",
        value_name = ".ext",
        num_args = 0..,
    )]
    exclude: Vec<String>,

    /// Focus search on specific set of extensions.
    #[clap(
        short = 'f',
        long,
        help = "Focus search on specific set of extensions. In case both exclude and focus are provided, focus takes precedence.",
        value_name = ".ext",
        num_args = 0..,
    )]
    focus: Vec<String>,

    /// Default code editor to open the files.
    #[clap(
        short = 'd',
        long,
        help = "Default editor to open files. By default the files are opened in neovim.",
        value_name = "nvim",
        default_value = "nvim"
    )]
    default_editor_command: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    
    // Prepare extension sets
    let mut exclude_extension_set: BTreeSet<String> = BTreeSet::new();
    let mut focus_extension_set: BTreeSet<String> = BTreeSet::new();
    args.exclude.into_iter().for_each(|ext| {
        exclude_extension_set.insert(ext);
    });
    args.focus.into_iter().for_each(|ext| {
        focus_extension_set.insert(ext);
    });

    // Scan directory
    // Note: This scanning is synchronous and might take time for large folders.
    // In the future, we should probably do this in a thread or background task within the App.
    let all_files = search::walk_directory(exclude_extension_set, focus_extension_set);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = app::App::new(all_files);
    
    // Set initial state from args if present
    if let Some(q) = args.query {
        app.search_query = q;
        app.current_screen = app::CurrentScreen::Search;
    }
    
    // Run app
    let res = app.run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
