use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap, Clear, Tabs},
    Frame,
};
use crate::app::{App, CurrentScreen, SearchMode};

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create the layout sections.
    // We can have a header, main content, and footer.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top bar
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    // Render footer first (helper text) based on context
    render_footer(f, app, chunks[2]);

    // Render main content based on state
    match app.current_screen {
        CurrentScreen::Home => render_home(f, app, chunks[1]),
        CurrentScreen::Search => render_search(f, app, chunks[1]),
        CurrentScreen::Options => render_options(f, app, chunks[1]),
        CurrentScreen::Help => render_help(f, app, chunks[1]),
        CurrentScreen::Exiting => {}
    }
}

fn render_home(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Top spacer
            Constraint::Length(8),      // Logo
            Constraint::Length(3),      // Description spacer
            Constraint::Length(11),     // Menu
            Constraint::Min(1),         // Bottom padding
        ])
        .split(area);

    // "FUZZY" in block ASCII art with gradient colors
    let logo_text = vec![
        Line::from(vec![
            Span::styled(" ███████╗", Style::default().fg(Color::Rgb(255, 0, 255))),
            Span::styled("██╗   ██╗", Style::default().fg(Color::Rgb(200, 50, 255))),
            Span::styled("███████╗", Style::default().fg(Color::Rgb(150, 100, 255))),
            Span::styled("███████╗", Style::default().fg(Color::Rgb(100, 150, 255))),
            Span::styled("██╗   ██╗", Style::default().fg(Color::Rgb(50, 200, 255))),
        ]),
        Line::from(vec![
            Span::styled(" ██╔════╝", Style::default().fg(Color::Rgb(255, 0, 255))),
            Span::styled("██║   ██║", Style::default().fg(Color::Rgb(200, 50, 255))),
            Span::styled("╚══███╔╝", Style::default().fg(Color::Rgb(150, 100, 255))),
            Span::styled("╚══███╔╝", Style::default().fg(Color::Rgb(100, 150, 255))),
            Span::styled("╚██╗ ██╔╝", Style::default().fg(Color::Rgb(50, 200, 255))),
        ]),
        Line::from(vec![
            Span::styled(" █████╗  ", Style::default().fg(Color::Rgb(255, 0, 255))),
            Span::styled("██║   ██║", Style::default().fg(Color::Rgb(200, 50, 255))),
            Span::styled("  ███╔╝ ", Style::default().fg(Color::Rgb(150, 100, 255))),
            Span::styled("  ███╔╝ ", Style::default().fg(Color::Rgb(100, 150, 255))),
            Span::styled(" ╚████╔╝ ", Style::default().fg(Color::Rgb(50, 200, 255))),
        ]),
        Line::from(vec![
            Span::styled(" ██╔══╝  ", Style::default().fg(Color::Rgb(255, 0, 255))),
            Span::styled("██║   ██║", Style::default().fg(Color::Rgb(200, 50, 255))),
            Span::styled(" ███╔╝  ", Style::default().fg(Color::Rgb(150, 100, 255))),
            Span::styled(" ███╔╝  ", Style::default().fg(Color::Rgb(100, 150, 255))),
            Span::styled("  ╚██╔╝  ", Style::default().fg(Color::Rgb(50, 200, 255))),
        ]),
        Line::from(vec![
            Span::styled(" ██║     ", Style::default().fg(Color::Rgb(255, 0, 255))),
            Span::styled("╚██████╔╝", Style::default().fg(Color::Rgb(200, 50, 255))),
            Span::styled("███████╗", Style::default().fg(Color::Rgb(150, 100, 255))),
            Span::styled("███████╗", Style::default().fg(Color::Rgb(100, 150, 255))),
            Span::styled("   ██║   ", Style::default().fg(Color::Rgb(50, 200, 255))),
        ]),
        Line::from(vec![
            Span::styled(" ╚═╝     ", Style::default().fg(Color::Rgb(255, 0, 255))),
            Span::styled(" ╚═════╝ ", Style::default().fg(Color::Rgb(200, 50, 255))),
            Span::styled("╚══════╝", Style::default().fg(Color::Rgb(150, 100, 255))),
            Span::styled("╚══════╝", Style::default().fg(Color::Rgb(100, 150, 255))),
            Span::styled("   ╚═╝   ", Style::default().fg(Color::Rgb(50, 200, 255))),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled("Locate files with fuzzy-ls", Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC).fg(Color::Gray))),
    ];

    let logo = Paragraph::new(logo_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(logo, chunks[1]);

    // Menu items with selection highlighting
    let menu_items = [
        ("Search Files", "s", 0),
        ("Options", "o", 1),
        ("Help", "h", 2),
        ("Quit", "q", 3),
    ];

    let menu_text: Vec<Line> = menu_items
        .iter()
        .flat_map(|(label, key, index)| {
            let is_selected = *index == app.home_menu_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let color = match *index {
                0 => Color::Cyan,
                1 => Color::Magenta,
                2 => Color::Green,
                3 => Color::Red,
                _ => Color::White,
            };

            let indicator = if is_selected { "▶ " } else { "  " };
            
            vec![
                Line::from(vec![
                    Span::styled(indicator, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("[{}] {}", key, label), style),
                ]),
                Line::from(Span::raw("")),
            ]
        })
        .collect();

    let menu = Paragraph::new(menu_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(menu, chunks[3]);
}

fn render_search(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input box
            Constraint::Min(1),    // Results
        ])
        .split(area);

    let title = match app.search_mode {
        SearchMode::Fuzzy => "Search (Fuzzy)",
        SearchMode::Regex => "Search (Regex)",
        SearchMode::Exact => "Search (Exact)",
    };

    let input = Paragraph::new(app.search_query.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(title)
        .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(input, chunks[0]);

    if app.results.is_empty() {
        let no_results = Paragraph::new("No results found or type to search.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Results")
            .border_style(Style::default().fg(Color::Gray)));
        f.render_widget(no_results, chunks[1]);
    } else {
        let rows: Vec<Row> = app
            .results
            .iter()
            .enumerate()
            .map(|(i, (score, name, path))| {
                let style = if i == app.selected_result_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else if *score == 0 {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    score.to_string(),
                    name.clone(),
                    path.clone(),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(rows, [Constraint::Length(5), Constraint::Percentage(30), Constraint::Percentage(65)])
            .header(Row::new(vec!["Score", "Name", "Path"]).style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)))
            .block(Block::default().borders(Borders::ALL).title(format!("Results ({})", app.results.len()))
            .border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(table, chunks[1]);
    }
}

fn render_options(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Instructions
            Constraint::Min(1),    // Options list
        ])
        .split(area);

    let instructions = Paragraph::new("Interact: ↑/↓ to select • ←/→/Enter to change • Esc to back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Instructions").border_style(Style::default().fg(Color::Magenta)));
    f.render_widget(instructions, chunks[0]);

    let options_items = vec![
        ("Search Mode", format!("{:?}", app.search_mode)),
        ("Fuzziness Threshold", format!("{:.1}", app.fuzziness_threshold)),
        ("Excluded Extensions", if app.exclude_extensions.is_empty() { "None".to_string() } else { app.exclude_extensions.clone() }),
        ("Focused Extensions", if app.focus_extensions.is_empty() { "None".to_string() } else { app.focus_extensions.clone() }),
    ];

    let rows: Vec<Row> = options_items
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let style = if i == app.options_selected_index {
                if app.is_editing && (i == 2 || i == 3) {
                     Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                     Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::White)
            };
            Row::new(vec![
                Span::raw(*label),
                Span::raw(value.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(40), Constraint::Percentage(60)])
        .block(Block::default().borders(Borders::ALL).title("Settings").border_style(Style::default().fg(Color::Magenta)));
    
    f.render_widget(table, chunks[1]);
}

fn render_help(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Help")
        .border_style(Style::default().fg(Color::Green));
        
    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
        Line::from("  Up/Down (k/j) : Move selection"),
        Line::from("  Enter         : Open file"),
        Line::from("  Esc           : Back / Quit"),
        Line::from(""),
        Line::from(Span::styled("Search Modes", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
        Line::from("  Fuzzy         : Standard approximate matching (Damerau-Levenshtein)"),
        Line::from("  Regex         : Regular expression matching"),
        Line::from("  Exact         : Exact string matching"),
        Line::from(""),
        Line::from(Span::styled("Tips", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
        Line::from("  Use 's' to start searching from Home."),
        Line::from("  Use 'o' to view options."),
    ];
    let p = Paragraph::new(help_text).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let (text, color) = match app.current_screen {
        CurrentScreen::Home => ("↑/↓ or j/k: Navigate • Enter: Select • Letter keys: Quick access", Color::Cyan),
        CurrentScreen::Search => ("Type to search • ↑/↓ or j/k: Navigate results • Enter: Open • Esc: Home", Color::Yellow),
        CurrentScreen::Options => ("↑/↓ or j/k: Select • ←/→ or h/l: Change • Enter: Edit • Esc: Home", Color::Magenta),
        CurrentScreen::Help => ("Esc or q: Return to home", Color::Green),
        CurrentScreen::Exiting => ("Goodbye!", Color::Red),
    };
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::Black).bg(color))
        .alignment(Alignment::Center);
    f.render_widget(p, area);
}
