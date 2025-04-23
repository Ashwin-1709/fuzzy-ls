extern crate clap;
mod editor;
mod search;
use clap::{ArgAction, Parser};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use regex::Regex;
use std::{collections::BTreeSet, io::Write};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table},
    Terminal,
};
#[derive(Parser)]
#[clap(
    name = "ffs",
    version = "0.1.0",
    about = "Fuzzy file search command line tool.",
    author = "Ashwin Pugalia"
)]
struct Cli {
    /// Query string used for the search.
    #[clap(
        help = "Query used for the search. Default search mode is fuzzy search within recursive directories."
    )]
    query: String,

    /// Use query as a regex pattern.
    #[clap(short, long, action = ArgAction::SetTrue, help = "Query is a regex pattern and the search is performed using the regex.")]
    regex: bool,

    /// Use query as an exact pattern.
    #[clap(short = 'p', long, action = ArgAction::SetTrue, help = "Exact pattern matching is done for the query.")]
    exact: bool,

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

fn display_results_ui(
    potential_hits: Vec<(u32, String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Layout for the table
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            if (potential_hits.is_empty()) {
                let no_results = Paragraph::new(Span::styled(
                    "No results found.",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                f.render_widget(no_results, chunks[0]);
            } else {
                // Table ows
                let rows: Vec<Row> = potential_hits
                    .iter()
                    .enumerate()
                    .map(|(index, (score, file_name, full_path))| {
                        let style = if *score == 0 {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Blue)
                        };
                        Row::new(vec![
                            Span::raw((index + 1).to_string()),
                            Span::styled(file_name.clone(), style),
                            Span::raw(full_path.clone()),
                        ])
                    })
                    .collect();

                // Table widget
                let table = Table::new(rows)
                    .header(Row::new(vec![
                        Span::styled("No.", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("File Name", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("Full Path", Style::default().add_modifier(Modifier::BOLD)),
                    ]))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Search Results"),
                    )
                    .widths(&[
                        Constraint::Length(5),
                        Constraint::Percentage(30),
                        Constraint::Percentage(65),
                    ]);

                f.render_widget(table, chunks[0]);
            }
        })?;

        // Wait for user input to exit
        if let Event::Key(key_event) = event::read()? {
            if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Esc {
                break; // Exit on 'q' or 'Esc' key
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    if args.regex && args.exact {
        return Err("Both regex and exact flags cannot be set together.".into());
    }
    let mut exclude_extension_set: BTreeSet<String> = BTreeSet::new();
    let mut focus_extension_set: BTreeSet<String> = BTreeSet::new();
    args.exclude.into_iter().for_each(|ext| {
        exclude_extension_set.insert(ext.to_string());
    });
    args.focus.into_iter().for_each(|ext| {
        focus_extension_set.insert(ext.to_string());
    });
    let files = search::walk_directory(exclude_extension_set, focus_extension_set);
    let mut potential_hits: Vec<(u32, String, String)> = Vec::new();
    if args.exact {
        for (file_name, full_path) in files {
            if file_name == args.query {
                potential_hits.push((0, file_name, full_path));
            }
        }
    } else if args.regex {
        let pattern: Regex = match Regex::new(&args.query) {
            Ok(pattern) => pattern,
            Err(error) => return Err(error.into()),
        };
        for (file_name, full_path) in files {
            match pattern.captures(&file_name) {
                Some(caps) => {
                    if caps
                        .get(0)
                        .map_or(false, |matched| matched.as_str() == file_name)
                    {
                        potential_hits.push((0, file_name, full_path));
                    }
                }
                None => continue,
            }
        }
    } else {
        let mut ranked_files: Vec<(u32, String, String)> = Vec::new();
        for (file_name, full_path) in files {
            match search::score_fuzzy_search(
                args.query.clone(),
                file_name.clone(),
                search::FuzzySearchAlgorithm::DamerauLevenshtein,
            ) {
                Ok(score) => ranked_files.push((score, file_name.clone(), full_path)),
                Err(error) => return Err(error.into()),
            };
        }
        ranked_files.sort_by(|a, b| a.0.cmp(&b.0));
        let threshold: u32 = match args.query.len() {
            0..=4 => (args.query.len() as f32 * 0.25).ceil() as u32,
            5..=10 => (args.query.len() as f32 * 0.35).ceil() as u32,
            _ => (args.query.len() as f32 * 0.45).ceil() as u32,
        };
        for (score, file_name, full_path) in ranked_files {
            if score <= threshold {
                potential_hits.push((score, file_name, full_path));
            } else {
                break;
            }
        }
    }
    if cfg!(feature = "open_in_editor") {
        if potential_hits.is_empty() {
            println!("No files found.");
        } else {
            println!("{} files found:", potential_hits.len());
            let mut file_number: usize = 1;
            for (score, file_name, full_path) in potential_hits.clone() {
                if score == 0 {
                    println!(
                        "{}. \x1b[32m{}\x1b[0m - {}",
                        file_number, file_name, full_path
                    ); // Green color for score 0
                } else {
                    println!(
                        "{}. \x1b[34m{}\x1b[0m - {}",
                        file_number, file_name, full_path
                    ); // Blue color for other scores
                }
                file_number += 1;
            }
            return editor::experimental_open_files(
                args.default_editor_command,
                file_number,
                potential_hits,
            );
        }
    }
    return display_results_ui(potential_hits);
}
