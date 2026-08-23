use gtk4::prelude::*;

use gtk4::{
    Application,
    ApplicationWindow,
    Box,
    Button,
    Entry,
    Orientation,
    Popover,
    ScrolledWindow,
};

use std::cell::RefCell;
use std::rc::Rc;

use sticky::model::NoteType;
use sticky::storage::repository::NoteRepository;

use super::editor::Editor;
use super::images::load_images_into_buffer;
use super::toolbar::build_toolbar;

pub fn build_window(app: &Application, note_id: &str) {
    let repository =
        NoteRepository::default()
            .expect("Could not initialize note repository");

    let note = repository
        .get(note_id)
        .expect("Could not read note");

let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("Sticky — {}", note.title))
        .default_width(360)
        .default_height(400)
        .decorated(false)
        .build();

    let note = Rc::new(RefCell::new(note));
    let repository = Rc::new(repository);

    let root = Box::new(Orientation::Vertical, 0);
    root.add_css_class("sticky-root");

    let header = build_header(
        app,
        &window,
        &note,
    );

    let editor = Editor::new(
        &note,
        &repository,
    );

    let scrolled = ScrolledWindow::new();
    scrolled.set_child(Some(editor.text_view()));
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);

    let toolbar = build_toolbar(
        &window,
        &editor,
        &note,
    );

    root.append(&header);
    root.append(&scrolled);
    root.append(&toolbar);

    window.set_child(Some(&root));
    window.present();

    load_images_into_buffer(editor.buffer(), note_id, editor.text_view());
}

fn build_header(
    app: &Application,
    window: &ApplicationWindow,
    note: &Rc<RefCell<sticky::model::Note>>,
) -> Box {
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
        let repository =
            NoteRepository::default()
                .expect("Could not initialize note repository");

        let new_note = repository
            .create("Untitled", NoteType::Text)
            .expect("Could not create note");

        println!("Created note: {}", new_note.id);

        let exe = std::env::current_exe()
            .expect("Could not find executable");

        std::process::Command::new(exe)
            .arg(&new_note.id)
            .spawn()
            .expect("Could not open new note");

        let _ = &app_for_add;
    });

    let title = Entry::new();
    title.set_text(&note.borrow().title);
    title.set_hexpand(true);
    title.add_css_class("sticky-title");
    title.set_has_frame(false);

    let repository_for_title =
        NoteRepository::default()
            .expect("Could not initialize note repository");

    let note_id_for_title =
        note.borrow().id.clone();

    title.connect_activate(move |entry| {
        let mut note = repository_for_title
            .get(&note_id_for_title)
            .expect("Could not read note");

        note.title = entry.text().to_string();
        note.updated = chrono::Local::now().fixed_offset();

        repository_for_title
            .update(&note)
            .expect("Could not save note");

        println!("Saved title: {}", note.title);
    });

    let menu_button = Button::with_label("☰");
    menu_button.add_css_class("icon-button");
    menu_button.set_tooltip_text(Some("Note menu"));

    let popover = Popover::new();
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
    let note_id_for_delete =
        note.borrow().id.clone();

    delete_button.connect_clicked(move |_| {
        let repository =
            NoteRepository::default()
                .expect("Could not initialize note repository");

        repository
            .delete(&note_id_for_delete)
            .expect("Could not delete note");

        window_for_delete.close();

        println!(
            "Deleted note {}",
            note_id_for_delete
        );
    });

    let close_button = Button::with_label("×");
    close_button.add_css_class("icon-button");
    close_button.set_tooltip_text(Some("Close"));

    let window_for_close = window.clone();

    close_button.connect_clicked(move |_| {
        window_for_close.close();
    });

    header.append(&add_button);
    header.append(&title);
    header.append(&menu_button);
    header.append(&close_button);

    header
}