use gtk4::prelude::*;
use gtk4::Application;

mod note;

use note::app::run;

const APP_ID: &str = "com.sticky.notes";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let note_id = match args.get(1) {
        Some(id) => id.clone(),
        None => {
            eprintln!("Usage: sticky-note <note-id>");
            std::process::exit(1);
        }
    };

    let application = Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    application.connect_activate(move |app| {
        run(app, &note_id);
    });

    // Give GTK only argv[0].
    // The note ID is captured by the activate callback.
    let program_name = args
        .first()
        .map(String::as_str)
        .unwrap_or("sticky-note");

    application.run_with_args(&[program_name]);
}
