use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    Terminal,
};
use std::time::{Duration, Instant};
use crate::search;
use regex::Regex;

pub enum CurrentScreen {
    Home,
    Search,
    Options,
    Help,
    Exiting,
}

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchMode {
    Fuzzy,
    Regex,
    Exact,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub search_query: String,
    pub search_mode: SearchMode,
    pub results: Vec<(u32, String, String)>, // score, name, path
    pub selected_result_index: usize,
    pub all_files: Vec<(String, String)>,
    pub exit: bool,
    
    // Options
    pub focus_extensions: String, // Comma separated
    pub exclude_extensions: String, // Comma separated
    pub fuzziness_threshold: f32, // 0.0 to 1.0 (though logic uses int calculation usually)

    // Internal state
    pub last_input_time: Instant,
    pub debounce_duration: Duration,
    pub dirty: bool,
    
    // UI state
    pub options_selected_index: usize,
    pub is_editing: bool,
    pub home_menu_index: usize,
}

impl App {
    pub fn new(all_files: Vec<(String, String)>) -> App {
        App {
            current_screen: CurrentScreen::Home,
            search_query: String::new(),
            search_mode: SearchMode::Fuzzy,
            results: Vec::new(),
            selected_result_index: 0,
            all_files,
            exit: false,
            focus_extensions: String::new(),
            exclude_extensions: String::new(),
            fuzziness_threshold: 0.4,
            last_input_time: Instant::now(),
            debounce_duration: Duration::from_millis(200),
            dirty: true,
            options_selected_index: 0,
            is_editing: false,
            home_menu_index: 0,
        }
    }

    pub fn rescan_files(&mut self) {
        let mut exclude_set = BTreeSet::new();
        for ext in self.exclude_extensions.split(',') {
            let trimmed = ext.trim();
            if !trimmed.is_empty() {
                exclude_set.insert(trimmed.to_string());
            }
        }

        let mut focus_set = BTreeSet::new();
        for ext in self.focus_extensions.split(',') {
            let trimmed = ext.trim();
            if !trimmed.is_empty() {
                focus_set.insert(trimmed.to_string());
            }
        }

        // Note: This is synchronous/blocking. For very large trees, might pause UI.
        self.all_files = search::walk_directory(exclude_set, focus_set);
        self.perform_search();
    }

    pub fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|f| crate::ui::ui(f, self))?;

            // Poll for events with a small timeout to keep UI responsive
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key);
                    }
                }
            }
            
            // Handle debounced search if needed
            if self.dirty && self.last_input_time.elapsed() >= self.debounce_duration {
                self.perform_search();
                self.dirty = false;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.current_screen {
            CurrentScreen::Home => match key.code {
                KeyCode::Char('s') => self.current_screen = CurrentScreen::Search,
                KeyCode::Char('o') => self.current_screen = CurrentScreen::Options,
                KeyCode::Char('h') => self.current_screen = CurrentScreen::Help,
                KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.home_menu_index < 3 { // 4 menu items (0-3)
                        self.home_menu_index += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.home_menu_index > 0 {
                        self.home_menu_index -= 1;
                    }
                }
                KeyCode::Enter => {
                    match self.home_menu_index {
                        0 => self.current_screen = CurrentScreen::Search,
                        1 => self.current_screen = CurrentScreen::Options,
                        2 => self.current_screen = CurrentScreen::Help,
                        3 => self.exit = true,
                        _ => {}
                    }
                }
                _ => {}
            },
            CurrentScreen::Search => match key.code {
                KeyCode::Esc => self.current_screen = CurrentScreen::Home,
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.reset_search_state();
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.reset_search_state();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.results.is_empty() && self.selected_result_index + 1 < self.results.len() {
                        self.selected_result_index += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_result_index > 0 {
                        self.selected_result_index -= 1;
                    }
                }
                KeyCode::Enter => {
                    if !self.results.is_empty() {
                         // TODO: Open file logic. For now just print or do nothing in TUI loop?
                         // actually we probably want to return the selected file to main to open it?
                         // Or we can handle it here if we want to stay in the app. 
                         // For now, let's assume we want to open and maybe exit or stay? 
                         // Let's implement open functionality later, for now just print to stdout on exit?
                         // Or spawn process.
                         if let Some(file) = self.results.get(self.selected_result_index) {
                             crate::editor::open_file_in_terminal("nvim", &file.2).ok(); // Hardcoded nvim for now, fix later
                         }
                    }
                }
                _ => {}
            },
            CurrentScreen::Options => {
                if self.is_editing {
                    match key.code {
                        KeyCode::Enter => {
                            self.is_editing = false;
                            // Trigger rescan if we edited extensions
                            if self.options_selected_index == 2 || self.options_selected_index == 3 {
                                self.rescan_files();
                            }
                        }
                        KeyCode::Esc => {
                            self.is_editing = false;
                            // Discard changes? Currently we edit in place, so accept.
                            // To discard, we'd need a temp buffer.
                            if self.options_selected_index == 2 || self.options_selected_index == 3 {
                                self.rescan_files();
                            }
                        }
                        KeyCode::Backspace => {
                            let target = if self.options_selected_index == 2 {
                                &mut self.exclude_extensions
                            } else {
                                &mut self.focus_extensions
                            };
                            target.pop();
                        }
                        KeyCode::Char(c) => {
                             let target = if self.options_selected_index == 2 {
                                &mut self.exclude_extensions
                            } else {
                                &mut self.focus_extensions
                            };
                            target.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => self.current_screen = CurrentScreen::Home,
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.options_selected_index < 3 { // 4 options total (0-3)
                                self.options_selected_index += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.options_selected_index > 0 {
                                self.options_selected_index -= 1;
                            }
                        }
                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                        match self.options_selected_index {
                            0 => { // Search Mode
                                self.search_mode = match self.search_mode {
                                    SearchMode::Fuzzy => SearchMode::Regex,
                                    SearchMode::Regex => SearchMode::Exact,
                                    SearchMode::Exact => SearchMode::Fuzzy,
                                };
                            }
                            1 => { // Fuzziness
                                if self.fuzziness_threshold < 0.9 {
                                    self.fuzziness_threshold += 0.1;
                                }
                            }
                            2 | 3 => {
                                // Enable editing
                                self.is_editing = true;
                            }
                            _ => {}
                        }
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            match self.options_selected_index {
                                0 => { // Search Mode (Cycle backwards)
                                self.search_mode = match self.search_mode {
                                    SearchMode::Fuzzy => SearchMode::Exact,
                                    SearchMode::Regex => SearchMode::Fuzzy,
                                    SearchMode::Exact => SearchMode::Regex,
                                };
                                }
                                1 => { // Fuzziness
                                if self.fuzziness_threshold > 0.1 {
                                    self.fuzziness_threshold -= 0.1;
                                }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            },
            CurrentScreen::Help => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.current_screen = CurrentScreen::Home,
                _ => {}
            },
            CurrentScreen::Exiting => {}
        }
    }

    fn reset_search_state(&mut self) {
        self.selected_result_index = 0;
        self.dirty = true;
        self.last_input_time = Instant::now();
    }

    fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            self.results.clear();
            return;
        }

        match self.search_mode {
            SearchMode::Fuzzy => {
                if let Ok(results) = search::score_batch(
                    &self.search_query,
                    &self.all_files,
                    search::FuzzySearchAlgorithm::DamerauLevenshtein,
                ) {
                    self.results = results;
                }
            }
            SearchMode::Regex => {
                if let Ok(re) = Regex::new(&self.search_query) {
                    let mut matches: Vec<(u32, String, String)> = self.all_files
                        .iter()
                        .filter_map(|(name, path)| {
                            if re.is_match(name) {
                                Some((0, name.clone(), path.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();
                    // Optional: Sort by name or path since score is 0?
                    // matches.sort_by(|a, b| a.1.cmp(&b.1));
                    self.results = matches;
                } else {
                     // Invalid regex, maybe show error or just empty results
                     self.results.clear();
                }
            }
            SearchMode::Exact => {
                 let matches: Vec<(u32, String, String)> = self.all_files
                    .iter()
                    .filter_map(|(name, path)| {
                        if name == &self.search_query {
                            Some((0, name.clone(), path.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                 self.results = matches;
            }
        }
        
        // Truncate for performance if too many
        if self.results.len() > 1000 {
            self.results.truncate(1000);
        }
    }
}
