use std::process::Command;

use ratatui::widgets::ListState;

use crate::model::NoteType;
use crate::storage::NoteRepository;

pub struct App {
    pub(crate) notes: Vec<crate::model::Note>,
    pub(crate) list_state: ListState,
    repository: NoteRepository,
    pub(crate) should_quit: bool,
    pub(crate) status_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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

    pub fn refresh_notes(&mut self) -> bool {
        let selected_note_id = self
            .list_state
            .selected()
            .and_then(|index| self.notes.get(index))
            .map(|note| note.id.clone());

        match self.repository.list() {
            Ok(notes) => {
                self.notes = notes;

                if self.notes.is_empty() {
                    self.list_state.select(None);
                    return true;
                }

                if let Some(note_id) = selected_note_id {
                    if let Some(new_index) =
                        self.notes.iter().position(|note| note.id == note_id)
                    {
                        self.list_state.select(Some(new_index));
                        return true;
                    }
                }

                self.list_state.select(Some(0));

                true
            }

            Err(e) => {
                self.status_message = Some(format!("Refresh error: {}", e));
                false
            }
        }
    }

    pub fn create_note(&mut self, title: &str, note_type: NoteType) {
        match self.repository.create(title, note_type) {
            Ok(note) => {
                self.status_message =
                    Some(format!("Created note: {}", note.id));

                self.refresh_notes();

                self.open_note_in_gtk(&note.id);
            }

            Err(e) => {
                self.status_message =
                    Some(format!("Error: {}", e));
            }
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.notes.len() {
                let note = self.notes[selected].clone();

                match self.repository.delete(&note.id) {
                    Ok(_) => {
                        self.status_message =
                            Some(format!("Deleted: {}", note.title));

                        self.refresh_notes();
                    }

                    Err(e) => {
                        self.status_message =
                            Some(format!("Error: {}", e));
                    }
                }
            }
        }
    }

    pub fn open_selected(&mut self) {
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
                let _ = Command::new(gtk_exe)
                    .arg(note_id)
                    .spawn();
            } else {
                let _ = Command::new(&exe)
                    .arg(note_id)
                    .spawn();
            }
        }
    }

    pub fn next(&mut self) {
        if self.notes.is_empty() {
            return;
        }

        let i = self.list_state.selected().unwrap_or(0);

        if i >= self.notes.len() - 1 {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(Some(i + 1));
        }
    }

    pub fn previous(&mut self) {
        if self.notes.is_empty() {
            return;
        }

        let i = self.list_state.selected().unwrap_or(0);

        if i == 0 {
            self.list_state.select(Some(self.notes.len() - 1));
        } else {
            self.list_state.select(Some(i - 1));
        }
    }
}