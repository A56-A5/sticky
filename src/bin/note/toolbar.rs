use gtk4::prelude::*;

use gtk4::{
    ApplicationWindow,
    Box,
    Button,
    Orientation,
};

use std::cell::RefCell;
use std::rc::Rc;

use sticky::model::Note;
use sticky::model::NoteType;

use super::editor::Editor;

pub fn build_toolbar(
    window: &ApplicationWindow,
    editor: &Editor,
    note: &Rc<RefCell<Note>>,
) -> Box {
    let toolbar =
        Box::new(Orientation::Horizontal, 2);

    toolbar.set_margin_top(3);
    toolbar.set_margin_bottom(3);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);

    toolbar.add_css_class("sticky-toolbar");

    let bold_button =
        Button::with_label("B");

    bold_button.add_css_class("format-button");
    bold_button.set_tooltip_text(Some("Bold"));

    {
        let editor = editor.clone_for_button();

        bold_button.connect_clicked(
            move |_| {
                editor.apply_bold();
            },
        );
    }

    let italic_button =
        Button::with_label("I");

    italic_button.add_css_class("format-button");
    italic_button.set_tooltip_text(Some("Italic"));

    {
        let editor = editor.clone_for_button();

        italic_button.connect_clicked(
            move |_| {
                editor.apply_italic();
            },
        );
    }

    let underline_button =
        Button::with_label("U");

    underline_button
        .add_css_class("format-button");

    underline_button
        .set_tooltip_text(Some("Underline"));

    {
        let editor = editor.clone_for_button();

        underline_button.connect_clicked(
            move |_| {
                editor.apply_underline();
            },
        );
    }

    let list_button =
        Button::with_label("≡");

    list_button.add_css_class("format-button");
    list_button.set_tooltip_text(Some("List"));

    {
        let editor = editor.clone_for_button();

        list_button.connect_clicked(
            move |_| {
                editor.insert_list_item();
            },
        );
    }

    let image_button =
        Button::with_label("▧");

    image_button
        .add_css_class("format-button");

    image_button
        .set_tooltip_text(Some("Insert image"));

    let window = window.clone();
    let note_id = note.borrow().id.clone();
    let editor_handle = editor.clone_for_button();

    image_button.connect_clicked(
        move |_| {
            editor_handle.insert_image(&window, &note_id);
        },
    );

    let preview_button = Button::with_label("👁");
    preview_button.add_css_class("format-button");
    preview_button.set_tooltip_text(Some("Toggle Markdown Preview"));
    
    let note_type = note.borrow().note_type;
    if note_type == NoteType::Markdown {
        let editor_handle = editor.clone_for_button();
        let text_view = editor.text_view().clone();
        let preview_button_clone = preview_button.clone();
        
        let is_preview = RefCell::new(false);
        let original_body = RefCell::new(String::new());
        
        preview_button.connect_clicked(move |_| {
            let mut preview = is_preview.borrow_mut();
            if !*preview {
                let text = editor_handle.get_plain_text();
                *original_body.borrow_mut() = text.clone();
                
                let html = markdown::to_html(&text);
                let markup = html_to_pango_markup(&html);
                
                text_view.set_editable(false);
                text_view.set_cursor_visible(false);
                let buffer = text_view.buffer();
                let mut iter = buffer.start_iter();
                buffer.delete(&mut iter, &mut buffer.end_iter());
                buffer.insert_markup(&mut iter, &markup);
                
                preview_button_clone.set_label("✎");
                *preview = true;
            } else {
                let text = original_body.borrow().clone();
                text_view.set_editable(true);
                text_view.set_cursor_visible(true);
                let buffer = text_view.buffer();
                let mut iter = buffer.start_iter();
                buffer.delete(&mut iter, &mut buffer.end_iter());
                buffer.insert(&mut iter, &text);
                
                preview_button_clone.set_label("👁");
                *preview = false;
            }
        });
        
        toolbar.append(&preview_button);
    }

    toolbar.append(&bold_button);
    toolbar.append(&italic_button);
    toolbar.append(&underline_button);
    toolbar.append(&list_button);
    toolbar.append(&image_button);

    toolbar
}

fn html_to_pango_markup(html: &str) -> String {
    let mut result = String::new();
    let mut in_paragraph = false;
    
    for line in html.lines() {
        let line = line.trim();
        if line.is_empty() {
            if in_paragraph {
                result.push_str("\n\n");
            }
            continue;
        }
        
        // Convert HTML tags to Pango markup
        let mut processed = line.to_string();
        
        // Headings
        processed = processed.replace("<h1>", "<span size=\"xx-large\" weight=\"bold\">")
            .replace("</h1>", "</span>")
            .replace("<h2>", "<span size=\"x-large\" weight=\"bold\">")
            .replace("</h2>", "</span>")
            .replace("<h3>", "<span size=\"large\" weight=\"bold\">")
            .replace("</h3>", "</span>")
            .replace("<h4>", "<span size=\"medium\" weight=\"bold\">")
            .replace("</h4>", "</span>")
            .replace("<h5>", "<span size=\"small\" weight=\"bold\">")
            .replace("</h5>", "</span>")
            .replace("<h6>", "<span size=\"x-small\" weight=\"bold\">")
            .replace("</h6>", "</span>");
        
        // Basic formatting
        processed = processed.replace("<strong>", "<b>")
            .replace("</strong>", "</b>")
            .replace("<b>", "<b>")
            .replace("</b>", "</b>")
            .replace("<em>", "<i>")
            .replace("</em>", "</i>")
            .replace("<i>", "<i>")
            .replace("</i>", "</i>")
            .replace("<u>", "<u>")
            .replace("</u>", "</u>")
            .replace("<code>", "<tt>")
            .replace("</code>", "</tt>")
            .replace("<pre>", "<tt>")
            .replace("</pre>", "</tt>");
        
        // Links - just show the text
        while let Some(start) = processed.find("<a href=") {
            if let Some(end) = processed[start..].find('>') {
                let link_end = processed[start..].find("</a>").map(|e| start + e + 4).unwrap_or(processed.len());
                let link_text = &processed[start + end + 1..link_end].to_string();
                processed.replace_range(start..link_end, &link_text);
            } else {
                break;
            }
        }
        
        // Line breaks
        processed = processed.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
        
        // List items
        processed = processed.replace("<li>", "\n• ").replace("</li>", "");
        processed = processed.replace("<ul>", "").replace("</ul>", "");
        processed = processed.replace("<ol>", "").replace("</ol>", "");
        
        // Paragraphs
        processed = processed.replace("<p>", "").replace("</p>", "\n\n");
        
        if !in_paragraph && !processed.is_empty() {
            in_paragraph = true;
        }
        
        result.push_str(&processed);
    }
    
    // Clean up multiple newlines
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }
    
    result.trim().to_string()
}
