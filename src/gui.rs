use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table},
    Terminal,
};
use std::time::Duration;

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
                ].as_ref())
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
                "↑/↓ or j/k: Move  Enter: Open  q/Esc: Quit"
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

