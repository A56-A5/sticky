use gtk4::Application;

use super::window::build_window;

pub fn run(app: &Application, note_id: &str) {
    load_css();
    build_window(app, note_id);
}

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

    let display = gtk4::gdk::Display::default()
        .expect("GTK display should exist");

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}