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

/// Displays the results of the search in a TUI interface.
/// The results are displayed in a table format with columns for the file name and full path.
/// The user can exit the interface by pressing 'q' or 'Esc'.
pub fn display_results_ui(
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

            if potential_hits.is_empty() {
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

