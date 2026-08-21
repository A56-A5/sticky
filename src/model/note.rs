use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use super::NoteType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub note_type: NoteType,
    pub created: DateTime<FixedOffset>,
    pub updated: DateTime<FixedOffset>,
    pub body: String,
}

impl Note {
    pub fn new(id: String, title: String, note_type: NoteType) -> Self {
        let now = chrono::Local::now().fixed_offset();

        Self {
            id,
            title,
            note_type,
            created: now,
            updated: now,
            body: String::new(),
        }
    }
}
