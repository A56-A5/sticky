use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::process::Command;

use crate::model::NoteType;
use crate::storage::NoteRepository;

struct App {
    notes: Vec<crate::model::Note>,
    list_state: ListState,
    repository: NoteRepository,
    should_quit: bool,
    status_message: Option<String>,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let repository = NoteRepository::default()?;
        let notes = repository.list()?;
        
        let mut list_state = ListState::default();
        if !notes.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            notes,
            list_state,
            repository,
            should_quit: false,
            status_message: None,
        })
    }

    fn refresh_notes(&mut self) {
        if let Ok(notes) = self.repository.list() {
            self.notes = notes;
            if self.list_state.selected().is_some() && self.list_state.selected().unwrap() >= self.notes.len() {
                if !self.notes.is_empty() {
                    self.list_state.select(Some(self.notes.len() - 1));
                } else {
                    self.list_state.select(None);
                }
            } else if self.notes.is_empty() {
                self.list_state.select(None);
            }
        }
    }

    fn create_note(&mut self, title: &str, note_type: NoteType) {
        match self.repository.create(title, note_type) {
            Ok(note) => {
                self.status_message = Some(format!("Created note: {}", note.id));
                self.refresh_notes();
                self.open_note_in_gtk(&note.id);
            }
            Err(e) => self.status_message = Some(format!("Error: {}", e)),
        }
    }

    fn delete_selected(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.notes.len() {
                let note = self.notes[selected].clone();
                match self.repository.delete(&note.id) {
                    Ok(_) => {
                        self.status_message = Some(format!("Deleted: {}", note.title));
                        self.refresh_notes();
                    }
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
            }
        }
    }

    fn open_selected(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.notes.len() {
                let note_id = self.notes[selected].id.clone();
                self.open_note_in_gtk(&note_id);
            }
        }
    }

    fn open_note_in_gtk(&self, note_id: &str) {
        if let Ok(exe) = std::env::current_exe() {
            let gtk_exe = exe.with_file_name("sticky-note");
            if gtk_exe.exists() {
                let _ = Command::new(gtk_exe).arg(note_id).spawn();
            } else {
                let _ = Command::new(&exe).arg(note_id).spawn();
            }
        }
    }

    fn next(&mut self) {
        if self.notes.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        if i >= self.notes.len() - 1 {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(Some(i + 1));
        }
    }

    fn previous(&mut self) {
        if self.notes.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        if i == 0 {
            self.list_state.select(Some(self.notes.len() - 1));
        } else {
            self.list_state.select(Some(i - 1));
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('n') => {
                        app.create_note("Untitled", NoteType::Text);
                    }
                    KeyCode::Char('N') => {
                        app.create_note("Untitled", NoteType::List);
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        app.create_note("Untitled", NoteType::Markdown);
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        app.delete_selected();
                    }
                    KeyCode::Char('o') | KeyCode::Char('O') | KeyCode::Enter => {
                        app.open_selected();
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        app.refresh_notes();
                        app.status_message = Some("Refreshed".to_string());
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    let title = Paragraph::new("Sticky Notes Manager")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .notes
        .iter()
        .map(|note| {
            let type_str = match note.note_type {
                NoteType::Text => "TEXT",
                NoteType::List => "LIST",
                NoteType::Markdown => "MD  ",
            };
            let type_color = match note.note_type {
                NoteType::Text => Color::Green,
                NoteType::List => Color::Blue,
                NoteType::Markdown => Color::Magenta,
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", type_str), Style::default().fg(type_color).add_modifier(Modifier::BOLD)),
                Span::styled(&note.title, Style::default().fg(Color::White)),
                Span::styled(format!("  ({})", &note.id[..8]), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let notes_list = List::new(items)
        .block(Block::default()
            .title(format!(" Notes ({}) ", app.notes.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol("► ");

    f.render_stateful_widget(notes_list, chunks[1], &mut app.list_state);

    let help = Paragraph::new(
        Line::from(vec![
            Span::styled(" n ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" New Text  "),
            Span::styled(" N ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" New List  "),
            Span::styled(" m ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" New MD  "),
            Span::styled(" o/Enter ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" Open  "),
            Span::styled(" d ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" Delete  "),
            Span::styled(" r ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" Refresh  "),
            Span::styled(" q/Esc ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" Quit  "),
            Span::styled(" ↑/↓ or j/k ", Style::default().fg(Color::Yellow).bg(Color::Black)),
            Span::raw(" Navigate"),
        ])
    )
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[2]);

    if let Some(msg) = &app.status_message {
        let status_area = Rect {
            x: chunks[1].x + 2,
            y: chunks[1].y + chunks[1].height.saturating_sub(3),
            width: chunks[1].width.saturating_sub(4).min(msg.len() as u16 + 4),
            height: 3,
        };
        let status = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Black).bg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, status_area);
    }
}