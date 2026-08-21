use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::model::{Note, NoteType};

use super::frontmatter;

pub struct NoteRepository {
    notes_dir: PathBuf,
}

impl NoteRepository {
    pub fn new(notes_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&notes_dir).context("failed to create notes directory")?;

        Ok(Self { notes_dir })
    }

    pub fn default() -> Result<Self> {
        let data_dir = dirs::data_dir().context("could not determine XDG data directory")?;

        Self::new(data_dir.join("sticky").join("notes"))
    }

    pub fn notes_dir(&self) -> &Path {
        &self.notes_dir
    }

    pub fn create(&self, title: impl Into<String>, note_type: NoteType) -> Result<Note> {
        let id = Uuid::new_v4().simple().to_string();

        let note = Note::new(id.clone(), title.into(), note_type);

        let note_dir = self.note_dir(&id);

        fs::create_dir_all(note_dir.join("images")).context("failed to create note directory")?;

        self.write(&note)?;

        Ok(note)
    }

    pub fn get(&self, id: &str) -> Result<Note> {
        let path = self.note_path(id);

        let content =
            fs::read_to_string(&path).with_context(|| format!("failed to read note: {}", id))?;

        frontmatter::deserialize(&content)
    }

    pub fn update(&self, note: &Note) -> Result<()> {
        self.write(note)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let dir = self.note_dir(id);

        if dir.exists() {
            fs::remove_dir_all(&dir).with_context(|| format!("failed to delete note: {}", id))?;
        }

        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();

        for entry in fs::read_dir(&self.notes_dir).context("failed to read notes directory")? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let note_path = path.join("note.md");

            if !note_path.exists() {
                continue;
            }

            let content = fs::read_to_string(&note_path)?;
            let note = frontmatter::deserialize(&content)?;

            notes.push(note);
        }

        notes.sort_by(|a, b| b.updated.cmp(&a.updated));

        Ok(notes)
    }

    fn write(&self, note: &Note) -> Result<()> {
        let dir = self.note_dir(&note.id);

        fs::create_dir_all(dir.join("images")).context("failed to create note directory")?;

        let path = self.note_path(&note.id);
        let content = frontmatter::serialize(note)?;

        let temp_path = path.with_extension("md.tmp");

        fs::write(&temp_path, content).context("failed to write temporary note file")?;

        fs::rename(&temp_path, &path).context("failed to atomically replace note file")?;

        Ok(())
    }

    fn note_dir(&self, id: &str) -> PathBuf {
        self.notes_dir.join(id)
    }

    fn note_path(&self, id: &str) -> PathBuf {
        self.note_dir(id).join("note.md")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NoteType;
    use tempfile::tempdir;

    #[test]
    fn create_and_read_note() {
        let temp = tempdir().unwrap();

        let repo = NoteRepository::new(temp.path().join("notes")).unwrap();

        let mut note = repo.create("Shopping", NoteType::List).unwrap();

        note.body = "- [ ] Milk\n- [ ] Eggs".to_string();

        repo.update(&note).unwrap();

        let loaded = repo.get(&note.id).unwrap();

        assert_eq!(loaded.id, note.id);
        assert_eq!(loaded.title, "Shopping");
        assert_eq!(loaded.note_type, NoteType::List);
        assert_eq!(loaded.body, "- [ ] Milk\n- [ ] Eggs");
    }

    #[test]
    fn delete_note() {
        let temp = tempdir().unwrap();

        let repo = NoteRepository::new(temp.path().join("notes")).unwrap();

        let note = repo.create("Temporary", NoteType::Text).unwrap();

        assert!(repo.note_dir(&note.id).exists());

        repo.delete(&note.id).unwrap();

        assert!(!repo.note_dir(&note.id).exists());
    }

    #[test]
    fn list_notes() {
        let temp = tempdir().unwrap();

        let repo = NoteRepository::new(temp.path().join("notes")).unwrap();

        repo.create("Shopping", NoteType::List).unwrap();
        repo.create("Ideas", NoteType::Markdown).unwrap();
        repo.create("Random", NoteType::Text).unwrap();

        let notes = repo.list().unwrap();

        assert_eq!(notes.len(), 3);
    }
}
