use anyhow::Result;
use chrono::Local;
use gtk4::prelude::*;
use gtk4::gdk;
use gtk4::pango;

use gtk4::{
    TextBuffer,
    TextView,
};

use std::cell::RefCell;
use std::rc::Rc;

use sticky::model::{Note, NoteImage, NoteType};
use sticky::storage::NoteRepository;

use super::resizable_image::ResizableImage;

pub struct Editor {
    text_view: TextView,
    buffer: TextBuffer,
    #[allow(dead_code)]
    note_type: NoteType,
}

impl Editor {
    pub fn new(
        note: &Rc<RefCell<Note>>,
        repository: &Rc<NoteRepository>,
    ) -> Self {
        let note_type = note.borrow().note_type;
        
        let text_view = TextView::new();

        text_view.add_css_class("sticky-editor");

        text_view.set_wrap_mode(
            gtk4::WrapMode::WordChar,
        );

        text_view.set_vexpand(true);
        text_view.set_hexpand(true);

        text_view.set_top_margin(12);
        text_view.set_bottom_margin(12);
        text_view.set_left_margin(14);
        text_view.set_right_margin(14);

        let buffer = text_view.buffer();

        create_format_tags(&buffer);
        create_list_tags(&buffer);

        let body = &note.borrow().body;
        if note_type == NoteType::List {
            let formatted = format_list_body(body);
            let mut iter = buffer.start_iter();
            buffer.insert_markup(&mut iter, &formatted);
        } else {
            buffer.set_text(body);
        }

        setup_note_type_handling(&text_view, &buffer, note_type);

        let note_id = note.borrow().id.clone();
        let repository = repository.clone();

        connect_autosave(&buffer, &note_id, &repository);

        Self {
            text_view,
            buffer,
            note_type,
        }
    }

    pub fn text_view(&self) -> &TextView {
        &self.text_view
    }

    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    #[allow(dead_code)]
    pub fn note_type(&self) -> NoteType {
        self.note_type
    }

    #[allow(dead_code)]
    pub fn get_plain_text(&self) -> String {
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        self.buffer.text(&start, &end, true).to_string()
    }

    #[allow(dead_code)]
    pub fn insert_image(&self, window: &gtk4::ApplicationWindow, note_id: &str) {
        use gtk4::{
            FileChooserAction,
            FileChooserNative,
            ResponseType,
        };
        let chooser = FileChooserNative::new(
            Some("Insert Image"),
            Some(window),
            FileChooserAction::Open,
            Some("Open"),
            Some("Cancel"),
        );

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Images"));
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("image/gif");
        filter.add_mime_type("image/webp");
        chooser.set_filter(&filter);

        let note_id = note_id.to_string();
        let buffer = self.buffer.clone();
        let text_view = self.text_view.clone();

        chooser.connect_response(move |chooser, response| {
            if response != ResponseType::Accept {
                return;
            }

            let Some(file) = chooser.file() else { return; };
            let Some(source_path) = file.path() else { return; };

            let Some(data_dir) = dirs::data_dir() else { return; };
            let images_dir = data_dir.join("sticky").join("notes").join(&note_id).join("images");
            if std::fs::create_dir_all(&images_dir).is_err() { return; }

            let Some(file_name) = source_path.file_name() else { return; };
            let file_name_str = file_name.to_string_lossy().to_string();
            let destination = images_dir.join(file_name);
            if std::fs::copy(&source_path, &destination).is_err() { return; }

            let default_width = 300;
            let default_height = 200;

            let mut iter = buffer.end_iter();
            buffer.insert(&mut iter, &format!("\n[IMAGE:{}|{}x{}]\n", file_name_str, default_width, default_height));

            let resizable = ResizableImage::new(file_name_str, default_width, default_height);
            let anchor = buffer.create_child_anchor(&mut iter);
            text_view.add_child_at_anchor(resizable.widget(), &anchor);
            buffer.insert(&mut buffer.end_iter(), "\n");
        });

        chooser.show();
    }
}

fn create_format_tags(buffer: &TextBuffer) {
    buffer
        .create_tag(
            Some("bold"),
            &[(
                "weight",
                &700i32.to_value(),
            )],
        )
        .expect("failed to create bold tag");

    buffer
        .create_tag(
            Some("italic"),
            &[(
                "style",
                &gtk4::pango::Style::Italic.to_value(),
            )],
        )
        .expect("failed to create italic tag");

    buffer
        .create_tag(
            Some("underline"),
            &[(
                "underline",
                &gtk4::pango::Underline::Single.to_value(),
            )],
        )
        .expect("failed to create underline tag");
}

fn create_list_tags(buffer: &TextBuffer) {
    let font_desc = pango::FontDescription::from_string("Monospace 16");
    
    buffer
        .create_tag(
            Some("list-bullet"),
            &[
                ("font-desc", &font_desc.to_value()),
            ],
        )
        .expect("failed to create list-bullet tag");

    buffer
        .create_tag(
            Some("list-checkbox"),
            &[
                ("font-desc", &font_desc.to_value()),
            ],
        )
        .expect("failed to create list-checkbox tag");
}

fn format_list_body(body: &str) -> String {
    let mut result = String::new();
    for (idx, line) in body.lines().enumerate() {
        if idx > 0 {
            result.push('\n');
        }
        if line.starts_with("- [ ] ") {
            let content = &line[6..];
            result.push_str(&format!("☐  {}", content));
        } else if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
            let content = &line[6..];
            result.push_str(&format!("☑  {}", content));
        } else if line.starts_with("- ") {
            let content = &line[2..];
            result.push_str(&format!("•  {}", content));
        } else if line.starts_with("* ") {
            let content = &line[2..];
            result.push_str(&format!("•  {}", content));
        } else if let Some(stripped) = line.strip_prefix("1. ") {
            let content = stripped;
            result.push_str(&format!("1.  {}", content));
        } else {
            result.push_str(line);
        }
    }
    result
}

fn setup_note_type_handling(text_view: &TextView, buffer: &TextBuffer, note_type: NoteType) {
    match note_type {
        NoteType::List => {
            setup_list_handling(text_view, buffer);
        }
        NoteType::Markdown => {
            setup_markdown_handling(text_view, buffer);
        }
        NoteType::Text => {}
    }
}

fn setup_list_handling(text_view: &TextView, buffer: &TextBuffer) {
    let controller = gtk4::EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let buffer = buffer.clone();
    
    controller.connect_key_pressed(move |_, keyval, _keycode, state| {
        if keyval == gdk::Key::Return && !state.contains(gdk::ModifierType::SHIFT_MASK) {
            let iter = buffer.iter_at_mark(&buffer.get_insert());
            let mut line_start = iter.clone();
            line_start.set_line_offset(0);
            let line_text = buffer.text(&line_start, &iter, true).to_string();
            
            if line_text.starts_with("☐  ") {
                let mut iter = buffer.end_iter();
                buffer.insert(&mut iter, "\n☐  ");
                return glib::Propagation::Stop;
            }
            if line_text.starts_with("☑  ") {
                let mut iter = buffer.end_iter();
                buffer.insert(&mut iter, "\n☐  ");
                return glib::Propagation::Stop;
            }
            if line_text.starts_with("•  ") {
                let mut iter = buffer.end_iter();
                buffer.insert(&mut iter, "\n•  ");
                return glib::Propagation::Stop;
            }
            if line_text.starts_with("1.  ") {
                let mut iter = buffer.end_iter();
                buffer.insert(&mut iter, "\n2.  ");
                return glib::Propagation::Stop;
            }
            let num_match = regex::Regex::new(r"^(\d+)\.\s+").unwrap();
            if let Some(caps) = num_match.captures(&line_text) {
                if let Ok(n) = caps[1].parse::<usize>() {
                    let mut iter = buffer.end_iter();
                    buffer.insert(&mut iter, &format!("\n{}.  ", n + 1));
                    return glib::Propagation::Stop;
                }
            }
        }
        glib::Propagation::Proceed
    });
    
    text_view.add_controller(controller);
}

fn setup_markdown_handling(text_view: &TextView, buffer: &TextBuffer) {
    let buffer = buffer.clone();
    
    let controller = gtk4::EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    
    controller.connect_key_pressed(move |_, keyval, _keycode, state| {
        if keyval == gdk::Key::Return && !state.contains(gdk::ModifierType::SHIFT_MASK) {
            let iter = buffer.iter_at_mark(&buffer.get_insert());
            let mut line_start = iter.clone();
            line_start.set_line_offset(0);
            let line_text = buffer.text(&line_start, &iter, true).to_string();
            
            if line_text.starts_with("# ") || line_text.starts_with("## ") || 
               line_text.starts_with("### ") || line_text.starts_with("- ") ||
               line_text.starts_with("* ") || line_text.starts_with("1. ") {
                let mut iter = buffer.end_iter();
                buffer.insert(&mut iter, "\n");
                return glib::Propagation::Stop;
            }
        }
        glib::Propagation::Proceed
    });
    
    text_view.add_controller(controller);
}

fn extract_images_from_text(text: &str) -> Vec<NoteImage> {
    let mut images = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("[IMAGE:") && line.ends_with("]") {
            let content = &line[7..line.len()-1];
            let parts: Vec<&str> = content.split('|').collect();
            if parts.len() == 2 {
                let file_name = parts[0].to_string();
                let size_parts: Vec<&str> = parts[1].split('x').collect();
                if size_parts.len() == 2 {
                    if let (Ok(width), Ok(height)) = (size_parts[0].parse::<i32>(), size_parts[1].parse::<i32>()) {
                        images.push(NoteImage { path: file_name, width, height });
                    }
                }
            }
        }
    }
    images
}

fn connect_autosave(buffer: &TextBuffer, note_id: &str, repository: &Rc<NoteRepository>) {
    let buffer = buffer.clone();
    let note_id = note_id.to_string();
    let repository = repository.clone();

    buffer.connect_changed(move |buffer| {
        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();

        let note_id = note_id.clone();
        let repository = repository.clone();

        glib::timeout_add_local_once(
            std::time::Duration::from_millis(400),
            move || {
                let result = (|| -> Result<()> {
                    let mut note = repository.get(&note_id)?;

                    note.body = text.clone();
                    note.images = extract_images_from_text(&text);
                    note.updated = Local::now().fixed_offset();

                    repository.update(&note)?;

                    Ok(())
                })();

                if let Err(err) = result {
                    eprintln!("Failed to autosave note {}: {:#}", note_id, err);
                }
            },
        );
    });
}
impl Editor {
    pub fn clone_for_button(&self) -> EditorHandle {
        EditorHandle {
            buffer: self.buffer.clone(),
            text_view: self.text_view.clone(),
        }
    }
}

#[derive(Clone)]
pub struct EditorHandle {
    buffer: TextBuffer,
    text_view: TextView,
}

impl EditorHandle {
    pub fn apply_bold(&self) {
        let Some((start, end)) =
            self.buffer.selection_bounds()
        else {
            return;
        };

        let Some(tag) =
            self.buffer.tag_table().lookup("bold")
        else {
            return;
        };

        self.buffer.apply_tag(
            &tag,
            &start,
            &end,
        );
    }

    pub fn apply_italic(&self) {
        let Some((start, end)) =
            self.buffer.selection_bounds()
        else {
            return;
        };

        let Some(tag) =
            self.buffer.tag_table().lookup("italic")
        else {
            return;
        };

        self.buffer.apply_tag(
            &tag,
            &start,
            &end,
        );
    }

    pub fn apply_underline(&self) {
        let Some((start, end)) =
            self.buffer.selection_bounds()
        else {
            return;
        };

        let Some(tag) =
            self.buffer.tag_table().lookup("underline")
        else {
            return;
        };

        self.buffer.apply_tag(
            &tag,
            &start,
            &end,
        );
    }

    pub fn insert_list_item(&self) {
        let mark = self.buffer.get_insert();
        let mut iter =
            self.buffer.iter_at_mark(&mark);

        iter.set_line_offset(0);

        self.buffer.insert(
            &mut iter,
            "- [ ] ",
        );
    }

    pub fn get_plain_text(&self) -> String {
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        self.buffer.text(&start, &end, true).to_string()
    }

    pub fn insert_image(&self, window: &gtk4::ApplicationWindow, note_id: &str) {
        use gtk4::{
            FileChooserAction,
            FileChooserNative,
            ResponseType,
        };
        let chooser = FileChooserNative::new(
            Some("Insert Image"),
            Some(window),
            FileChooserAction::Open,
            Some("Open"),
            Some("Cancel"),
        );

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Images"));
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("image/gif");
        filter.add_mime_type("image/webp");
        chooser.set_filter(&filter);

        let note_id = note_id.to_string();
        let buffer = self.buffer.clone();
        let text_view = self.text_view.clone();

        chooser.connect_response(move |chooser, response| {
            if response != ResponseType::Accept {
                return;
            }

            let Some(file) = chooser.file() else { return; };
            let Some(source_path) = file.path() else { return; };

            let Some(data_dir) = dirs::data_dir() else { return; };
            let images_dir = data_dir.join("sticky").join("notes").join(&note_id).join("images");
            if std::fs::create_dir_all(&images_dir).is_err() { return; }

            let Some(file_name) = source_path.file_name() else { return; };
            let file_name_str = file_name.to_string_lossy().to_string();
            let destination = images_dir.join(file_name);
            if std::fs::copy(&source_path, &destination).is_err() { return; }

            let default_width = 300;
            let default_height = 200;

            let mut iter = buffer.end_iter();
            buffer.insert(&mut iter, &format!("\n[IMAGE:{}|{}x{}]\n", file_name_str, default_width, default_height));

            let resizable = ResizableImage::new(file_name_str, default_width, default_height);
            let anchor = buffer.create_child_anchor(&mut iter);
            text_view.add_child_at_anchor(resizable.widget(), &anchor);
            buffer.insert(&mut buffer.end_iter(), "\n");
        });

        chooser.show();
    }
}