use gtk4::prelude::*;

use gtk4::{
    Application,
    ApplicationWindow,
    Box,
    Button,
    FileChooserAction,
    FileChooserNative,
    Orientation,
    ResponseType,
    ScrolledWindow,
    TextView,
};

use std::path::PathBuf;

use std::cell::RefCell;
use std::rc::Rc;

use sticky::storage::repository::NoteRepository;

const APP_ID: &str = "com.sticky.notes";
const AUTOSAVE_DELAY_MS: u64 = 400;

fn load_css() {
    let provider = gtk4::CssProvider::new();

    provider.load_from_data(
        r#"
        window {
            background: #FFF3A6;
        }

        .sticky-root {
            background: #FFF3A6;
        }

        .sticky-header {
            background: #FFF3A6;
            border-bottom: 1px solid #E7D76A;
            padding: 6px 10px;
        }

        .sticky-title {
            color: #292929;
            font-size: 17px;
            font-weight: 700;
        }

        .sticky-editor {
            background: #FFF3A6;
            color: #292929;
            font-size: 16px;
        }

        .sticky-editor text {
            background: #FFF3A6;
            color: #292929;
        }

        .sticky-toolbar {
            background: #FFF3A6;
            border-top: 1px solid #E7D76A;
            padding: 6px 10px;
        }

        .icon-button {
            background: transparent;
            background-image: none;
            border: none;
            box-shadow: none;
            outline: none;
            color: #292929;
            min-width: 32px;
            min-height: 32px;
            padding: 0;
            border-radius: 8px;
        }

        .icon-button:hover {
            background: rgba(80, 70, 20, 0.10);
        }

        .icon-button:active {
            background: rgba(80, 70, 20, 0.16);
        }

        .format-button {
            background: transparent;
            background-image: none;
            border: none;
            box-shadow: none;
            outline: none;
            color: #292929;
            min-width: 34px;
            min-height: 32px;
            padding: 0;
            border-radius: 7px;
        }

        .format-button:hover {
            background: rgba(80, 70, 20, 0.10);
        }

        .format-button:active {
            background: rgba(80, 70, 20, 0.16);
        }

        .menu-popover {
            background: #FFF8C9;
            border: 1px solid #D8C85E;
            border-radius: 10px;
            padding: 5px;
        }

        .menu-button {
            background: transparent;
            background-image: none;
            border: none;
            box-shadow: none;
            color: #292929;
            min-width: 150px;
            min-height: 36px;
            padding: 6px 12px;
            border-radius: 7px;
            text-align: left;
        }

        .menu-button:hover {
            background: #F0E58F;
        }
        
        .sticky-popover contents {
            background: #FFF3A6;
        }

        .sticky-popover button {
            background: transparent;
            color: #222222;
            border: none;
            box-shadow: none;
            font-size: 14px;
            min-width: 120px;
            min-height: 32px;
        }

        .sticky-popover button:hover {
            background: rgba(0, 0, 0, 0.08);
        }
        "#,
    );

    let display = gtk4::gdk::Display::default().expect("GTK display should exist");

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_window(app: &Application, note_id: &str) {
    let repository = NoteRepository::default().expect("Could not initialize note repository");

    let note = repository.get(note_id).expect("Could not read note");

    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("Sticky — {}", note.title))
        .default_width(360)
        .default_height(400)
        .decorated(false)
        .build();

    // Keep the note and repository alive for the lifetime of the window.
    let note = Rc::new(RefCell::new(note));
    let repository = Rc::new(repository);

    // ─────────────────────────────────────────
    // Root
    // ─────────────────────────────────────────

    let root = Box::new(Orientation::Vertical, 0);
    root.add_css_class("sticky-root");

    // ─────────────────────────────────────────
    // Header
    // ─────────────────────────────────────────

    let header = Box::new(Orientation::Horizontal, 4);

    header.set_margin_top(6);
    header.set_margin_bottom(6);
    header.set_margin_start(8);
    header.set_margin_end(8);

    header.add_css_class("sticky-header");

    let add_button = Button::with_label("+");
    add_button.add_css_class("icon-button");
    add_button.set_tooltip_text(Some("New note"));

    let app_for_add = app.clone();

    add_button.connect_clicked(move |_| {
        let repository = NoteRepository::default().expect("Could not initialize note repository");

        let new_note = repository
            .create("Untitled", sticky::model::NoteType::Text)
            .expect("Could not create note");

        println!("Created note: {}", new_note.id);

        let exe = std::env::current_exe().expect("Could not find executable");

        std::process::Command::new(exe)
            .arg(&new_note.id)
            .spawn()
            .expect("Could not open new note");

        let _ = &app_for_add;
    });

    let title = gtk4::Entry::new();
    title.set_text(&note.borrow().title);
    title.set_hexpand(true);
    title.add_css_class("sticky-title");
    title.set_has_frame(false);

    let repository_for_title =
        NoteRepository::default().expect("Could not initialize note repository");

    let note_id_for_title = note.borrow().id.clone();

    title.connect_activate(move |entry| {
        let mut note = repository_for_title
            .get(&note_id_for_title)
            .expect("Could not read note");

        note.title = entry.text().to_string();

        repository_for_title
            .update(&note)
            .expect("Could not save note");

        println!("Saved title: {}", note.title);
    });

    let menu_button = Button::with_label("☰");
    menu_button.add_css_class("icon-button");
    menu_button.set_tooltip_text(Some("Note menu"));

    let popover = gtk4::Popover::new();
    popover.add_css_class("sticky-popover");

    let menu_box = Box::new(Orientation::Vertical, 4);
    menu_box.set_margin_top(8);
    menu_box.set_margin_bottom(8);
    menu_box.set_margin_start(8);
    menu_box.set_margin_end(8);

    let delete_button = Button::with_label("Delete note");
    menu_box.append(&delete_button);

    popover.set_child(Some(&menu_box));
    popover.set_parent(&menu_button);

    menu_button.connect_clicked({
        let popover = popover.clone();

        move |_| {
            popover.popup();
        }
    });

    let window_for_delete = window.clone();
    let note_id_for_delete = note.borrow().id.clone();

    delete_button.connect_clicked(move |_| {
        let repository = NoteRepository::default().expect("Could not initialize note repository");

        repository
            .delete(&note_id_for_delete)
            .expect("Could not delete note");

        window_for_delete.close();

        println!("Deleted note {}", note_id_for_delete);
    });

    let close_button = Button::with_label("×");
    close_button.add_css_class("icon-button");
    close_button.set_tooltip_text(Some("Close"));

    header.append(&add_button);
    header.append(&title);
    header.append(&menu_button);
    header.append(&close_button);

    // ─────────────────────────────────────────
    // Close
    // ─────────────────────────────────────────

    let window_for_close = window.clone();

    close_button.connect_clicked(move |_| {
        window_for_close.close();
    });

    // ─────────────────────────────────────────
    // Editor
    // ─────────────────────────────────────────

    let text_view = TextView::new();
    text_view.add_css_class("sticky-editor");

    text_view.set_wrap_mode(gtk4::WrapMode::WordChar);
    text_view.set_vexpand(true);
    text_view.set_hexpand(true);

    text_view.set_top_margin(12);
    text_view.set_bottom_margin(12);
    text_view.set_left_margin(14);
    text_view.set_right_margin(14);

    let buffer = text_view.buffer();
    let bold_tag = buffer
        .create_tag(Some("bold"), &[("weight", &700i32.to_value())])
        .expect("failed to create bold tag");

    let italic_tag = buffer
        .create_tag(
            Some("italic"),
            &[("style", &gtk4::pango::Style::Italic.to_value())],
        )
        .expect("failed to create italic tag");

    let underline_tag = buffer
        .create_tag(
            Some("underline"),
            &[(
                "underline",
                &gtk4::pango::Underline::Single.to_value(),
            )],
        )
        .expect("failed to create underline tag");

    // Load existing body.
    buffer.set_text(&note.borrow().body);

    // ─────────────────────────────────────────
    // Autosave
    // ─────────────────────────────────────────

    let save_source: Rc<RefCell<Option<gtk4::glib::SourceId>>> = Rc::new(RefCell::new(None));

    let note_for_save = Rc::clone(&note);
    let repository_for_save = Rc::clone(&repository);

    buffer.connect_changed(move |buffer| {
        // Cancel the previous pending save.

        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();

        let note = Rc::clone(&note_for_save);
        let repository = Rc::clone(&repository_for_save);

        let source_id = gtk4::glib::timeout_add_local(
            std::time::Duration::from_millis(AUTOSAVE_DELAY_MS),
            move || {
                {
                    let mut note = note.borrow_mut();

                    note.body = text.clone();

                    if let Err(error) = repository.update(&note) {
                        eprintln!("Failed to save note {}: {:#}", note.id, error);
                    } else {
                        println!("Saved note {}", note.id);
                    }
                }

                gtk4::glib::ControlFlow::Break
            },
        );

        *save_source.borrow_mut() = Some(source_id);
    });

    let scrolled = ScrolledWindow::new();

    scrolled.set_child(Some(&text_view));
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);

    // ─────────────────────────────────────────
    // Toolbar
    // ─────────────────────────────────────────

    let toolbar = Box::new(Orientation::Horizontal, 2);

    toolbar.set_margin_top(3);
    toolbar.set_margin_bottom(3);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);

    toolbar.add_css_class("sticky-toolbar");

    let bold_button = Button::with_label("B");
    let buffer_for_bold = buffer.clone();

    let bold_tag_for_button = bold_tag.clone();

    bold_button.connect_clicked(move |_| {
        if let Some((start, end)) = buffer_for_bold.selection_bounds() {
            buffer_for_bold.apply_tag(
                &bold_tag_for_button,
                &start,
                &end,
            );
        }
    });

    bold_button.add_css_class("format-button");

    let italic_button = Button::with_label("I");
    let buffer_for_italic = buffer.clone();

    let italic_tag_for_button = italic_tag.clone();

    italic_button.connect_clicked(move |_| {
        if let Some((start, end)) = buffer_for_italic.selection_bounds() {
            buffer_for_italic.apply_tag(
                &italic_tag_for_button,
                &start,
                &end,
            );
        }
    });

    italic_button.add_css_class("format-button");

    let underline_button = Button::with_label("U");

    let buffer_for_underline = buffer.clone();
    let underline_tag_for_button = underline_tag.clone();

    underline_button.connect_clicked(move |_| {
        if let Some((start, end)) = buffer_for_underline.selection_bounds() {
            buffer_for_underline.apply_tag(
                &underline_tag_for_button,
                &start,
                &end,
            );
        }
    });
    
    underline_button.add_css_class("format-button");

    let list_button = Button::with_label("≡");
    let buffer_for_list = buffer.clone();

    list_button.connect_clicked({
        let text_view = text_view.clone();

        move |_| {
            let buffer = text_view.buffer();

            let mark = buffer.get_insert();
            let mut iter = buffer.iter_at_mark(&mark);

            iter.set_line_offset(0);

            buffer.insert(&mut iter, "- [ ] ");
        }
    });

    list_button.add_css_class("format-button");
    let image_button = Button::with_label("▧");
    image_button.add_css_class("format-button");
    
    let window_for_image = window.clone();
    let note_id_for_image = note.borrow().id.clone();
    let buffer_for_image = buffer.clone();
    
    image_button.connect_clicked(move |_| {
        let chooser = FileChooserNative::new(
            Some("Insert Image"),
            Some(&window_for_image),
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
    
        let note_id = note_id_for_image.clone();
        let buffer = buffer_for_image.clone();
    
        chooser.connect_response(move |chooser, response| {
            if response != ResponseType::Accept {
                return;
            }
        
            let Some(file) = chooser.file() else {
                return;
            };
        
            let Some(source_path) = file.path() else {
                eprintln!("Could not get image path");
                return;
            };
        
            let Some(data_dir) = dirs::data_dir() else {
                eprintln!("Could not determine data directory");
                return;
            };
        
            let images_dir = data_dir
                .join("sticky")
                .join("notes")
                .join(&note_id)
                .join("images");
        
            if let Err(error) = std::fs::create_dir_all(&images_dir) {
                eprintln!("Could not create images directory: {error}");
                return;
            }
        
            let Some(file_name) = source_path.file_name() else {
                eprintln!("Could not determine image filename");
                return;
            };
        
            let destination = images_dir.join(file_name);
        
            if let Err(error) = std::fs::copy(&source_path, &destination) {
                eprintln!("Could not copy image: {error}");
                return;
            }
        
            let pixbuf = match gtk4::gdk_pixbuf::Pixbuf::from_file(&destination) {
                Ok(pixbuf) => pixbuf,
                Err(error) => {
                    eprintln!("Could not load image: {error}");
                    return;
                }
            };
        
            let image = gtk4::Image::from_pixbuf(Some(&pixbuf));
        
            let mut iter = buffer.end_iter();
        
            buffer.insert_paintable(
                &mut iter,
                &image.paintable().expect("Image has no paintable"),
            );
        
            buffer.insert(&mut iter, "\n");
        
            println!("Inserted image: {}", destination.display());
        });
    
        chooser.show();
    });

    bold_button.set_tooltip_text(Some("Bold"));
    italic_button.set_tooltip_text(Some("Italic"));
    underline_button.set_tooltip_text(Some("Underline"));
    list_button.set_tooltip_text(Some("List"));
    image_button.set_tooltip_text(Some("Insert image"));

    toolbar.append(&bold_button);
    toolbar.append(&italic_button);
    toolbar.append(&underline_button);
    toolbar.append(&list_button);
    toolbar.append(&image_button);

    // ─────────────────────────────────────────
    // Assemble
    // ─────────────────────────────────────────

    root.append(&header);
    root.append(&scrolled);
    root.append(&toolbar);

    window.set_child(Some(&root));

    window.present();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: sticky-note <note-id>");
        std::process::exit(1);
    }

    let note_id = args[1].clone();

    let application = Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    application.connect_activate(move |app| {
        load_css();
        build_window(app, &note_id);
    });

    let program_name = args.first().map(String::as_str).unwrap_or("sticky-note");

    application.run_with_args(&[program_name]);
}
