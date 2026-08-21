use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::model::{Note, NoteType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMeta {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub note_type: NoteType,
    pub created: chrono::DateTime<chrono::FixedOffset>,
    pub updated: chrono::DateTime<chrono::FixedOffset>,
}

impl From<&Note> for NoteMeta {
    fn from(note: &Note) -> Self {
        Self {
            id: note.id.clone(),
            title: note.title.clone(),
            note_type: note.note_type,
            created: note.created,
            updated: note.updated,
        }
    }
}

pub fn serialize(note: &Note) -> Result<String> {
    let meta = NoteMeta::from(note);

    let frontmatter = toml::to_string(&meta).context("failed to serialize note frontmatter")?;

    Ok(format!("---\n{}---\n\n{}", frontmatter, note.body))
}

pub fn deserialize(content: &str) -> Result<Note> {
    let content = content
        .strip_prefix("---\n")
        .context("note is missing frontmatter opening delimiter")?;

    let (frontmatter, body) = content
        .split_once("\n---\n")
        .context("note is missing frontmatter closing delimiter")?;

    let meta: NoteMeta = toml::from_str(frontmatter).context("failed to parse note frontmatter")?;

    Ok(Note {
        id: meta.id,
        title: meta.title,
        note_type: meta.note_type,
        created: meta.created,
        updated: meta.updated,
        body: body.strip_prefix('\n').unwrap_or(body).to_string(),
    })
}
