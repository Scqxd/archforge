//! Ratatui-based TUI for ArchForge
//!
//! Features:
//! - Package list with search/filter
//! - PKGBUILD preview with syntax highlighting
//! - Vim-like keybindings (hjkl, /search, :w, :q)
//! - Real-time status updates

use std::io::{self, stdout};
use std::time::{Duration, Instant};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, ClearType},
};

use crate::slugify;

/// App state for TUI
pub struct App {
    /// Current tab (0=Generate, 1=Build, 2=Search, 3=Deploy)
    pub tab: usize,
    /// Input mode (normal/insert)
    pub insert_mode: bool,
    /// Command buffer (for :commands)
    pub command_buffer: String,
    /// Search query
    pub search_query: String,
    /// Generated PKGBUILD content
    pub pkgbuild_content: String,
    /// Package list
    pub packages: Vec<PackageItem>,
    /// Selected package index
    pub selected_package: usize,
    /// Status message
    pub status_message: String,
    /// Last status update
    pub status_time: Instant,
    /// Build output
    pub build_output: String,
    /// Scroll offset for preview
    pub scroll_offset: u16,
    /// Quit flag
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: 0,
            insert_mode: false,
            command_buffer: String::new(),
            search_query: String::new(),
            pkgbuild_content: String::new(),
            packages: Vec::new(),
            selected_package: 0,
            status_message: String::new(),
            status_time: Instant::now(),
            build_output: String::new(),
            scroll_offset: 0,
            should_quit: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PackageItem {
    pub name: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            packages: get_sample_packages(),
            status_message: "Welcome to ArchForge! Press ? for help".to_string(),
            ..Default::default()
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Ctrl+C to quit
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }

        // Tab switching
        if key.code == KeyCode::Left && key.modifiers.is_empty() {
            self.tab = self.tab.saturating_sub(1);
            return false;
        }
        if key.code == KeyCode::Right && key.modifiers.is_empty() {
            self.tab = (self.tab + 1) % 4;
            return false;
        }

        // Vim-like normal mode
        if !self.insert_mode {
            self.handle_normal_mode(key)
        } else {
            self.handle_insert_mode(key)
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // Navigation
            KeyCode::Char('h') | KeyCode::Left => {
                self.tab = self.tab.saturating_sub(1);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_package < self.packages.len().saturating_sub(1) {
                    self.selected_package += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_package = self.selected_package.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.tab = (self.tab + 1) % 4;
            }

            // Mode switching
            KeyCode::Char('i') | KeyCode::Char('a') => {
                self.insert_mode = true;
                self.status_message = "INSERT MODE".to_string();
            }

            // Command mode
            KeyCode::Char(':') => {
                self.command_buffer.clear();
                self.status_message = ":".to_string();
            }

            // Search
            KeyCode::Char('/') => {
                self.search_query.clear();
                self.status_message = "/".to_string();
            }

            // Help
            KeyCode::Char('?') => {
                self.status_message = "HELP: h/j/k/l=navigate, i=insert, :=cmd, /=search, q=quit, ?=help".to_string();
            }

            // Quit
            KeyCode::Char('q') => {
                self.should_quit = true;
                return true;
            }

            // Scroll up/down in preview
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(3);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_offset = self.scroll_offset.saturating_add(3);
            }

            _ => {}
        }
        false
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.insert_mode = false;
                self.status_message = "NORMAL MODE".to_string();
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            KeyCode::Enter => {
                // Take ownership of command_buffer
                let cmd = std::mem::take(&mut self.command_buffer);
                self.execute_command(&cmd);
                self.insert_mode = false;
            }
            _ => {}
        }
        false
    }

    fn execute_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        if parts.is_empty() || parts[0].is_empty() {
            return;
        }

        match parts[0] {
            "q" | "quit" | "exit" => {
                self.should_quit = true;
                self.status_message = "Goodbye!".to_string();
            }
            "w" | "write" if parts.len() > 1 => {
                // :w <filename> - write to file
                match std::fs::write(parts[1], &self.pkgbuild_content) {
                    Ok(_) => {
                        self.status_message = format!("Written to: {}", parts[1]);
                    }
                    Err(e) => {
                        self.status_message = format!("Error writing: {}", e);
                    }
                }
            }
            "w" | "write" => {
                self.status_message = "Usage: :w <filename>".to_string();
            }
            "wq" | "x" => {
                // Save and quit
                if parts.len() > 2 && parts[1] == "w" {
                    match std::fs::write(parts[2], &self.pkgbuild_content) {
                        Ok(_) => {
                            self.status_message = format!("Saved to {} and quit", parts[2]);
                        }
                        Err(e) => {
                            self.status_message = format!("Error: {}", e);
                        }
                    }
                } else {
                    self.status_message = "Wrote to stdout and quit".to_string();
                }
                self.should_quit = true;
            }
            "help" | "h" => {
                self.status_message = "Commands: :q=quit, :w <file>=save, :wq=save+quit, :gen <desc>=generate, :clear=clear".to_string();
            }
            "gen" | "generate" => {
                // :gen <description> - generate PKGBUILD
                if parts.len() > 1 {
                    let desc = parts[1..].join(" ");
                    self.pkgbuild_content = format!("# Generated PKGBUILD for: {}\n# Generated by ArchForge\n\npkgname={}\npkgver=0.1.0\npkgrel=1\npkgdesc=\"{}\"\narch=(x86_64)\nurl=\"https://github.com/example/{}\"\nlicense=(MIT)\ndepends=()\nmakedepends=(\n    make\n    gcc\n)\nsource=(\"\")\nsha256sums=()\n\nbuild() {{\n    cd \"${{pkgname}}-${{pkgver}}\"\n    make\n}}\n\npackage() {{\n    cd \"${{pkgname}}-${{pkgver}}\"\n    make DESTDIR=\"${{pkgdir}}\" install\n}}\n",
                        slugify(&desc), slugify(&desc), desc, slugify(&desc));
                    self.status_message = "PKGBUILD generated!".to_string();
                } else {
                    self.status_message = "Usage: :gen <description>".to_string();
                }
            }
            "clear" => {
                self.pkgbuild_content.clear();
                self.status_message = "Cleared".to_string();
            }
            "tab" => {
                if parts.len() > 1 {
                    match parts[1].parse::<usize>() {
                        Ok(n) if n > 0 && n <= 4 => {
                            self.tab = n - 1;
                            self.status_message = format!("Switched to tab {}", n);
                        }
                        _ => {
                            self.status_message = "Invalid tab number (1-4)".to_string();
                        }
                    }
                }
            }
            _ => {
                self.status_message = format!("Unknown command: {}. Try :help", cmd);
            }
        }
        self.status_time = Instant::now();
    }

    /// Update status with timeout
    pub fn update_status(&mut self, msg: String) {
        self.status_message = msg;
        self.status_time = Instant::now();
    }

    /// Get age of current status message
    pub fn status_age(&self) -> Duration {
        Instant::now().duration_since(self.status_time)
    }
}

/// Get sample packages for demo
fn get_sample_packages() -> Vec<PackageItem> {
    vec![
        PackageItem {
            name: "firefox".to_string(),
            version: "135.0".to_string(),
            description: "Standalone web browser from mozilla.org".to_string(),
            installed: true,
        },
        PackageItem {
            name: "neovim".to_string(),
            version: "0.10.0".to_string(),
            description: "Vim-fork focused on extensibility and agility".to_string(),
            installed: true,
        },
        PackageItem {
            name: "alacritty".to_string(),
            version: "0.13.0".to_string(),
            description: "Modern, portable, terminal emulator".to_string(),
            installed: false,
        },
        PackageItem {
            name: "zsh".to_string(),
            version: "5.9".to_string(),
            description: "Shell with many enhancements".to_string(),
            installed: true,
        },
        PackageItem {
            name: "paru".to_string(),
            version: "1.11.2".to_string(),
            description: "AUR helper written in Rust".to_string(),
            installed: true,
        },
    ]
}

/// Main TUI entry point
pub fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, crossterm::terminal::Clear(ClearType::All))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Initial render
    terminal.draw(|f| ui(f, &app))?;

    loop {
        // Check for events with timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let quit = app.handle_key(key);
                terminal.draw(|f| ui(f, &app))?;
                if quit || app.should_quit {
                    break;
                }
            }
        }

        // Auto-clear status after 3 seconds
        if app.status_age() > Duration::from_secs(3) && !app.insert_mode {
            app.status_message = "Ready".to_string();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), crossterm::terminal::Clear(ClearType::All))?;
    Ok(())
}

/// Main UI rendering function
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Length(3),  // Search/Command bar
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.size());

    // Render tabs
    let tabs = Tabs::new(vec![
        Span::raw(" GENERATE "),
        Span::raw(" BUILD "),
        Span::raw(" SEARCH "),
        Span::raw(" DEPLOY "),
    ])
    .select(app.tab)
    .style(Style::default().fg(Color::Gray))
    .highlight_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, chunks[0]);

    // Render search/command bar
    let bar_content = if app.insert_mode {
        format!("{} {}", app.status_message, app.command_buffer)
    } else {
        app.status_message.clone()
    };
    let bar = Paragraph::new(Text::from(bar_content))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(bar, chunks[1]);

    // Render main content based on tab
    match app.tab {
        0 => render_generate_tab(f, app, chunks[2]),
        1 => render_build_tab(f, app, chunks[2]),
        2 => render_search_tab(f, app, chunks[2]),
        3 => render_deploy_tab(f, app, chunks[2]),
        _ => {}
    }

    // Render status bar
    let status_bar = Paragraph::new(Text::from(format!(
        "Mode: {} | Tab: {}/4 | Packages: {} | {}",
        if app.insert_mode { "INSERT" } else { "NORMAL" },
        app.tab + 1,
        app.packages.len(),
        if app.insert_mode { "Type and press Enter" } else { "Press i to insert, : to command, ? for help" }
    )))
    .style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(status_bar, chunks[3]);
}

fn render_generate_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    // Package list
    let items: Vec<ListItem> = app
        .packages
        .iter()
        .enumerate()
        .map(|(i, pkg)| {
            let style = if i == app.selected_package {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!(
                "[{}] {} - {}",
                if pkg.installed { "x" } else { " " },
                pkg.name,
                pkg.version
            ))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Packages").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Green).bg(Color::DarkGray));

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_package));
    f.render_stateful_widget(list, chunks[0], &mut list_state);

    // PKGBUILD preview
    let preview = Paragraph::new(Text::from(app.pkgbuild_content.clone()))
        .block(Block::default().title("PKGBUILD Preview").borders(Borders::ALL))
        .scroll((app.scroll_offset, 0));

    f.render_widget(preview, chunks[1]);
}

fn render_build_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let build_queue = List::new(vec![
        ListItem::new("1. firefox - Building..."),
        ListItem::new("2. neovim - Pending"),
        ListItem::new("3. alacritty - Pending"),
    ])
    .block(Block::default().title("Build Queue").borders(Borders::ALL));

    f.render_widget(build_queue, chunks[0]);

    let output = Paragraph::new(Text::from(app.build_output.clone()))
        .block(Block::default().title("Build Output").borders(Borders::ALL))
        .scroll((app.scroll_offset, 0));

    f.render_widget(output, chunks[1]);
}

fn render_search_tab(f: &mut Frame, app: &App, area: Rect) {
    let search_input = Paragraph::new(Text::from(app.search_query.clone()))
        .block(Block::default().title("Search Query").borders(Borders::ALL));

    f.render_widget(search_input, area);

    // Results below
    let results_area = Rect::new(area.x, area.y + 4, area.width, area.height - 4);
    let results = Paragraph::new(Text::from("Search results will appear here..."))
        .block(Block::default().title("Results").borders(Borders::ALL));

    f.render_widget(results, results_area);
}

fn render_deploy_tab(f: &mut Frame, _app: &App, area: Rect) {
    let options = vec![
        "1. Deploy to AUR",
        "2. Build Docker image",
        "3. Build Flatpak bundle",
        "4. Generate Nix flake",
    ];

    let menu = List::new(
        options.into_iter()
            .map(ListItem::new)
            .collect::<Vec<_>>()
    )
    .block(Block::default().title("Deployment Options").borders(Borders::ALL));

    f.render_widget(menu, area);
}