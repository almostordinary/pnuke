use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::io::{Result, stdout};
use std::time::Duration;

use crate::engine::{get_all_listeners, nuke_port};
use crate::types::ProcessInfo;

pub struct App {
    pub listeners: Vec<ProcessInfo>,
    pub filtered: Vec<ProcessInfo>,
    pub table_state: TableState,
    pub filter_query: String,
    pub is_filtering: bool,
    pub toast: Option<(String, bool)>, // (message, is_success)
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let listeners = get_all_listeners().unwrap_or_default();
        let filtered = listeners.clone();
        let mut table_state = TableState::default();
        if !filtered.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            listeners,
            filtered,
            table_state,
            filter_query: String::new(),
            is_filtering: false,
            toast: None,
            should_quit: false,
        }
    }

    pub fn refresh(&mut self) {
        self.listeners = get_all_listeners().unwrap_or_default();
        self.apply_filter();
    }

    pub fn apply_filter(&mut self) {
        if self.filter_query.is_empty() {
            self.filtered = self.listeners.clone();
        } else {
            let q = self.filter_query.to_lowercase();
            self.filtered = self
                .listeners
                .iter()
                .filter(|p| {
                    p.port.to_string().contains(&q)
                        || p.name.to_lowercase().contains(&q)
                        || p.protocol.to_lowercase().contains(&q)
                        || p.pid.to_string().contains(&q)
                })
                .cloned()
                .collect();
        }

        if self.filtered.is_empty() {
            self.table_state.select(None);
        } else {
            let selected = self.table_state.selected().unwrap_or(0);
            if selected >= self.filtered.len() {
                self.table_state.select(Some(self.filtered.len() - 1));
            } else {
                self.table_state.select(Some(selected));
            }
        }
    }

    pub fn next(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.filtered.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn nuke_selected(&mut self) {
        if let Some(proc_info) = self
            .table_state
            .selected()
            .and_then(|idx| self.filtered.get(idx))
        {
            let port = proc_info.port;
            let results = nuke_port(port, false);
            let successes = results.iter().filter(|r| r.success).count();

            if successes > 0 {
                self.toast = Some((
                    format!(
                        "✔ Successfully nuked port {} ({} process{})",
                        port,
                        successes,
                        if successes > 1 { "es" } else { "" }
                    ),
                    true,
                ));
            } else {
                let err = results
                    .first()
                    .map(|r| r.message.clone())
                    .unwrap_or_else(|| "Failed to terminate".to_string());
                self.toast = Some((format!("✖ Error nuking port {}: {}", port, err), false));
            }

            self.refresh();
        }
    }
}

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(5),    // Table
                    Constraint::Length(3), // Footer / Toast
                ])
                .split(f.area());

            // Header
            let filter_display = if app.is_filtering {
                format!(" [Filter: {}_]", app.filter_query)
            } else if !app.filter_query.is_empty() {
                format!(" [Filter: {}]", app.filter_query)
            } else {
                String::new()
            };

            let header_text = vec![Line::from(vec![
                Span::styled(
                    " ⚡ PNUKE ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    "Sub-millisecond Port Cockpit",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" ({} listeners detected)", app.filtered.len()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    filter_display,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])];

            let header_block = Paragraph::new(header_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(header_block, chunks[0]);

            // Table
            let rows: Vec<Row> = app
                .filtered
                .iter()
                .map(|p| {
                    let port_cell = Cell::from(p.port.to_string()).style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    );
                    let proto_cell =
                        Cell::from(p.protocol.clone()).style(Style::default().fg(Color::Yellow));
                    let pid_cell =
                        Cell::from(p.pid.to_string()).style(Style::default().fg(Color::Magenta));
                    let name_cell = Cell::from(p.name.clone()).style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    );
                    let ip_cell = Cell::from(format!("{}:{}", p.ip, p.port))
                        .style(Style::default().fg(Color::DarkGray));

                    Row::new(vec![port_cell, proto_cell, pid_cell, name_cell, ip_cell])
                })
                .collect();

            let header_cells = ["PORT", "PROTO", "PID", "PROCESS NAME", "ADDRESS"]
                .iter()
                .map(|h| {
                    Cell::from(*h).style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                });
            let table_header = Row::new(header_cells).bottom_margin(1);

            let table = Table::new(
                rows,
                [
                    Constraint::Length(8),
                    Constraint::Length(8),
                    Constraint::Length(10),
                    Constraint::Length(25),
                    Constraint::Min(20),
                ],
            )
            .header(table_header)
            .block(
                Block::default()
                    .title(" Active Listening Ports ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 44, 52))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

            f.render_stateful_widget(table, chunks[1], &mut app.table_state);

            // Footer / Toast
            let footer_content = if let Some((msg, success)) = &app.toast {
                Line::from(vec![
                    Span::styled(
                        if *success { " ✔ " } else { " ✖ " },
                        Style::default()
                            .fg(Color::Black)
                            .bg(if *success { Color::Green } else { Color::Red })
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        msg.clone(),
                        Style::default()
                            .fg(if *success { Color::Green } else { Color::Red })
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else if app.is_filtering {
                Line::from(vec![
                    Span::styled(
                        " SEARCH MODE ",
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Type to filter | Press [Enter] or [Esc] to return to navigation"),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        " [↑/↓] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Navigate  "),
                    Span::styled(
                        " [Enter/k] ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Nuke Port  "),
                    Span::styled(
                        " [/] ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Filter  "),
                    Span::styled(
                        " [r] ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Refresh  "),
                    Span::styled(
                        " [q] ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Quit"),
                ])
            };

            let footer_block = Paragraph::new(vec![footer_content]).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(footer_block, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            if app.is_filtering {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        app.is_filtering = false;
                    }
                    KeyCode::Backspace => {
                        app.filter_query.pop();
                        app.apply_filter();
                    }
                    KeyCode::Char(c) => {
                        app.filter_query.push(c);
                        app.apply_filter();
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Char('/') => {
                        app.is_filtering = true;
                    }
                    KeyCode::Char('r') => {
                        app.toast = None;
                        app.refresh();
                    }
                    KeyCode::Enter | KeyCode::Char('x') | KeyCode::Char('d') => {
                        app.nuke_selected();
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
