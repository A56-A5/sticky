use gtk4::prelude::*;

use gtk4::{
    TextBuffer,
    TextView,
};

use std::path::Path;

pub fn load_images_into_buffer(
    buffer: &TextBuffer,
    note_id: &str,
    text_view: &TextView,
) {
    let data_dir = match dirs::data_dir() {
        Some(d) => d,
        None => return,
    };

    let images_dir = data_dir
        .join("sticky")
        .join("notes")
        .join(note_id)
        .join("images");

    if !images_dir.exists() {
        return;
    }

    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true).to_string();
    
    let mut new_text = String::new();
    let mut last_end = 0;
    
    for (line_idx, line) in text.lines().enumerate() {
        let line_start = text[last_end..].find(line).map(|i| last_end + i).unwrap_or(last_end);
        let line_end = line_start + line.len();
        last_end = line_end + 1;
        
        let trimmed = line.trim();
        if trimmed.starts_with("[IMAGE:") && trimmed.ends_with("]") {
            let parts: Vec<&str> = trimmed[7..trimmed.len()-1].split('|').collect();
            if parts.len() == 2 {
                let file_name = parts[0];
                let size_parts: Vec<&str> = parts[1].split('x').collect();
                if size_parts.len() == 2 {
                    if let (Ok(width), Ok(height)) = (size_parts[0].parse::<i32>(), size_parts[1].parse::<i32>()) {
                        let image_path = images_dir.join(file_name).to_string_lossy().to_string();
                        if Path::new(&image_path).exists() {
                            let resizable = super::resizable_image::ResizableImage::new(image_path, width, height);
                            
                            let Some(mut iter) = buffer.iter_at_line(line_idx as i32) else { continue; };
                            let mut line_start_iter = iter.clone();
                            line_start_iter.forward_to_line_end();
                            buffer.delete(&mut iter, &mut line_start_iter);
                            
                            let Some(mut iter) = buffer.iter_at_line(line_idx as i32) else { continue; };
                            buffer.insert(&mut iter, "\n");
                            let anchor = buffer.create_child_anchor(&mut iter);
                            text_view.add_child_at_anchor(resizable.widget(), &anchor);
                            buffer.insert(&mut iter, "\n");
                            
                            let buffer_clone = buffer.clone();
                            let _file_name_clone = file_name.to_string();
                            resizable.connect_size_change(move |fname, w, h| {
                                let text = buffer_clone.text(&buffer_clone.start_iter(), &buffer_clone.end_iter(), true).to_string();
                                let new_text = text.replace(
                                    &format!("[IMAGE:{}|{}x{}]", fname, width, height),
                                    &format!("[IMAGE:{}|{}x{}]", fname, w, h)
                                );
                                if new_text != text {
                                    let start = buffer_clone.start_iter();
                                    let end = buffer_clone.end_iter();
                                    buffer_clone.delete(&mut start.clone(), &mut end.clone());
                                    buffer_clone.insert(&mut buffer_clone.start_iter(), &new_text);
                                }
                            });
                            continue;
                        }
                    }
                }
            }
        }
        new_text.push_str(line);
        new_text.push('\n');
    }
}