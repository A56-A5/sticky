use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::model::NoteType;
use crate::storage::NoteRepository;

#[derive(Parser, Debug)]
#[command(name = "sticky", version, about = "Native sticky notes for Linux")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create a new note
    New {
        /// Note title
        #[arg(short = 'n', long = "title", default_value = "Untitled")]
        title: String,

        /// Note type
        #[arg(short = 't', long = "type", default_value = "text")]
        note_type: String,
    },

    /// List all notes
    List,

    /// Open a note
    Open {
        /// Note ID
        id: String,
    },

    /// Delete a note
    Delete {
        /// Note ID
        id: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let repository = NoteRepository::default()?;

    match cli.command {
        None => {
            println!("Sticky manager coming soon.");
        }

        Some(Command::New { title, note_type }) => {
            let note_type = parse_note_type(&note_type)?;

            let note = repository.create(title, note_type)?;

            println!("Created note:");
            println!("  ID:    {}", note.id);
            println!("  Title: {}", note.title);
            println!("  Type:  {:?}", note.note_type);
        }

        Some(Command::List) => {
            let notes = repository.list()?;

            if notes.is_empty() {
                println!("No notes.");
                return Ok(());
            }

            for note in notes {
                println!("{}  {:<20} {:?}", note.id, note.title, note.note_type);
            }
        }

        Some(Command::Open { id }) => {
            let note = repository.get(&id)?;

            println!("Note:");
            println!("  ID:    {}", note.id);
            println!("  Title: {}", note.title);
            println!("  Type:  {:?}", note.note_type);
            println!();
            println!("{}", note.body);
        }

        Some(Command::Delete { id }) => {
            let note = repository.get(&id)?;

            repository.delete(&id)?;

            println!("Deleted '{}'.", note.title);
        }
    }

    Ok(())
}

fn parse_note_type(value: &str) -> Result<NoteType> {
    match value.to_lowercase().as_str() {
        "text" => Ok(NoteType::Text),
        "list" => Ok(NoteType::List),
        "markdown" | "md" => Ok(NoteType::Markdown),
        other => anyhow::bail!(
            "unknown note type '{}'. Expected: text, list, or markdown",
            other
        ),
    }
}
