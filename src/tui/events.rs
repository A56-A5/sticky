use std::io;

use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEventKind,
};

use crate::model::NoteType;

use super::app::App;

pub fn handle_events(app: &mut App) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.should_quit = true;
                }

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

                KeyCode::Char('o')
                | KeyCode::Char('O')
                | KeyCode::Enter => {
                    app.open_selected();
                }

                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if app.refresh_notes() {
                        app.status_message =
                            Some("Refreshed".to_string());
                    }
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    app.next();
                }

                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous();
                }

                _ => {}
            }
        }
    }

    Ok(())
}