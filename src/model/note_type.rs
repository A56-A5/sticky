use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteType {
    Text,
    List,
    Markdown,
}

impl Default for NoteType {
    fn default() -> Self {
        Self::Text
    }
}
