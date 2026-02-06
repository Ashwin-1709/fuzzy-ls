use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Terminal,
};
use std::time::Duration;
use std::time::Instant;
use crate::search;

fn flush_input_events() -> std::io::Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read();
    }
    Ok(())
}

/// Displays the results of the search in a TUI interface.
/// The results are displayed in a table format with columns for the file name and full path.
/// The user can exit the interface by pressing 'q' or 'Esc'.
pub fn display_results_ui(
    potential_hits: Vec<(u32, String, String)>,
    default_editor_command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected_index: usize = 0;
    let num_results = potential_hits.len();

    // Flush input events before starting the main loop
    flush_input_events()?;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Layout for the table and help line
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(2), // For help line
                ]
                .as_ref())
                .split(size);

            if potential_hits.is_empty() {
                let no_results = Paragraph::new(Span::styled(
                    "No results found.",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                f.render_widget(no_results, chunks[0]);
            } else {
                // Table rows
                let rows: Vec<Row> = potential_hits
                    .iter()
                    .enumerate()
                    .map(|(index, (score, file_name, full_path))| {
                        let mut style = if *score == 0 {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Blue)
                        };
                        if index == selected_index {
                            style = style.bg(Color::Yellow).fg(Color::Black);
                        }
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

            // Help/instructions line
            let help = Paragraph::new(Span::raw(
                "↑/↓ or j/k: Move  Enter: Open  q/Esc: Quit",
            ));
            f.render_widget(help, chunks[1]);
        })?;

        // Flush any remaining input events to prevent key repeat issues on Windows
        // Also add a small delay to prevent rapid key processing
        flush_input_events()?;
        std::thread::sleep(Duration::from_millis(10));

        // Handle user input for navigation
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => break, // Exit
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected_index + 1 < num_results {
                        selected_index += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Enter => {
                    if num_results > 0 {
                        let (_score, _file_name, full_path) = &potential_hits[selected_index];
                        open_in_new_terminal(default_editor_command, &[full_path])
                            .expect("Failed to open file in the editor.");
                        break;
                    }
                }
                _ => {}
            }
        }

        // Flush any remaining input events to prevent key repeat issues on Windows
        // Also add a small delay to prevent rapid key processing
        flush_input_events()?;
        std::thread::sleep(Duration::from_millis(10));
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

fn open_in_new_terminal(command: &str, args: &[&str]) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    let terminal_cmd = "cmd";
    #[cfg(target_os = "windows")]
    let terminal_args = &["/c", "start", command];

    #[cfg(target_os = "linux")]
    let terminal_cmd = "gnome-terminal";
    #[cfg(target_os = "linux")]
    let terminal_args = &["--", command];

    #[cfg(target_os = "macos")]
    let terminal_cmd = "open";
    #[cfg(target_os = "macos")]
    let terminal_args = &["-a", "Terminal", command];

    let mut cmd = std::process::Command::new(terminal_cmd);
    cmd.args(terminal_args);
    cmd.args(args);
    cmd.spawn()?;
    Ok(())
}

/// Interactive incremental search UI.
/// - `all_files`: list of (file_name, full_path) to search over
/// - typing updates results (debounced), Enter opens selected, q/Esc quits.
pub fn display_interactive_ui(
    all_files: Vec<(String, String)>,
    default_editor_command: &str,
    initial_query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut query = initial_query.to_string();
    let debounce = Duration::from_millis(150);
    // Force initial recompute by making last_input sufficiently old.
    let mut last_input = Instant::now() - debounce;
    let mut dirty = true;
    let mut results: Vec<(u32, String, String)> = Vec::new();
    let mut selected_index: usize = 0;
    let max_rows = 1000usize;
    // When there's an initial query, keep a cached copy of the initial
    // scored results. For each edit we first try a cheap prefix-filter
    // over this cache; if that yields no results we fall back to running
    // the full fuzzy scorer across all files.
    let mut initial_results: Option<Vec<(u32, String, String)>> = None;

    flush_input_events()?;

    loop {
        // Recompute if needed (debounced)
        if dirty && last_input.elapsed() >= debounce {
            // Try prefix-filter over cached initial results first (cheap).
            let mut used_prefix = false;
            if let Some(init) = &initial_results {
                if !query.is_empty() {
                    let qlow = query.to_lowercase();
                    let mut filtered: Vec<(u32, String, String)> = init
                        .iter()
                        .filter(|(_score, name, _path)| name.to_lowercase().starts_with(&qlow))
                        .cloned()
                        .collect();
                    if !filtered.is_empty() {
                        filtered.sort_by(|a, b| a.0.cmp(&b.0));
                        if filtered.len() > max_rows {
                            filtered.truncate(max_rows);
                        }
                        results = filtered;
                        used_prefix = true;
                    }
                }
            }

            if !used_prefix {
                // Full fuzzy rescore across all files (parallelized in search::score_batch)
                match search::score_batch(&query, &all_files, search::FuzzySearchAlgorithm::DamerauLevenshtein) {
                    Ok(mut scored) => {
                        if scored.len() > max_rows {
                            scored.truncate(max_rows);
                        }
                        // Save initial scored results if this is the first compute
                        if initial_results.is_none() {
                            initial_results = Some(scored.clone());
                        }
                        results = scored;
                        if selected_index >= results.len() && !results.is_empty() {
                            selected_index = results.len() - 1;
                        }
                    }
                    Err(_) => {
                        results.clear();
                    }
                }
            }
            dirty = false;
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ]
                .as_ref())
                .split(size);

            // Input box
            let input = Paragraph::new(Spans::from(vec![Span::raw(format!("> {}", query))]))
                .block(Block::default().borders(Borders::ALL).title("Filter (type to search)"));
            f.render_widget(input, chunks[0]);

            // Results
            if results.is_empty() {
                let p = Paragraph::new(Span::styled("No results", Style::default().add_modifier(Modifier::BOLD)));
                f.render_widget(p, chunks[1]);
            } else {
                let rows: Vec<Row> = results
                    .iter()
                    .enumerate()
                    .map(|(i, (score, name, path))| {
                        let mut style = if *score == 0 {
                            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Blue)
                        };
                        if i == selected_index {
                            style = style.bg(Color::Yellow).fg(Color::Black);
                        }
                        Row::new(vec![
                            Span::raw((i + 1).to_string()),
                            Span::styled(name.clone(), style),
                            Span::raw(path.clone()),
                        ])
                    })
                    .collect();

                let table = Table::new(rows)
                    .header(Row::new(vec![
                        Span::styled("No.", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("File Name", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("Full Path", Style::default().add_modifier(Modifier::BOLD)),
                    ]))
                    .block(Block::default().borders(Borders::ALL).title("Results"))
                    .widths(&[Constraint::Length(5), Constraint::Percentage(35), Constraint::Percentage(60)]);
                f.render_widget(table, chunks[1]);
            }

            // Footer
            let help = Paragraph::new(Spans::from(vec![Span::raw(
                "↑/↓ j/k: navigate  Enter: open  Ctrl+U: clear  Backspace: delete  q/Esc: quit",
            )]));
            f.render_widget(help, chunks[2]);
        })?;

        // Small sleep to avoid busy loop
        std::thread::sleep(Duration::from_millis(10));

        // Poll for events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                // Only handle actual key presses. Some terminals/platforms emit
                // multiple key event kinds (Press, Release, Repeat). Ignoring
                // non-Press events prevents duplicate character handling.
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !results.is_empty() && selected_index + 1 < results.len() {
                            selected_index += 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected_index > 0 {
                            selected_index -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        if !results.is_empty() {
                            let (_score, _name, full_path) = &results[selected_index];
                            open_in_new_terminal(default_editor_command, &[full_path])?;
                            break;
                        }
                    }
                    KeyCode::Backspace => {
                        query.pop();
                        dirty = true;
                        last_input = Instant::now();
                    }
                    KeyCode::Char('u') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        query.clear();
                        dirty = true;
                        last_input = Instant::now();
                    }
                    KeyCode::Char(c) => {
                        query.push(c);
                        dirty = true;
                        last_input = Instant::now();
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

