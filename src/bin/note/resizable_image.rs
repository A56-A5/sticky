use gtk4::prelude::*;
use gtk4::{
    Picture,
    EventControllerMotion,
    GestureDrag,
    Box as GtkBox,
    Orientation,
    DrawingArea,
};

use std::cell::RefCell;
use std::rc::Rc;

const MIN_SIZE: i32 = 50;
const MAX_SIZE: i32 = 600;
const HANDLE_SIZE: f64 = 16.0;

pub struct ResizableImage {
    container: GtkBox,
    picture: Picture,
    #[allow(dead_code)]
    image_path: String,
    file_name: String,
    width: RefCell<i32>,
    height: RefCell<i32>,
    drag_start: RefCell<Option<(f64, f64)>>,
    on_size_change: RefCell<Option<Box<dyn Fn(&str, i32, i32) + 'static>>>,
}

impl ResizableImage {
    pub fn new(image_path: String, initial_width: i32, initial_height: i32) -> Rc<Self> {
        let picture = Picture::for_filename(&image_path);
        picture.set_keep_aspect_ratio(true);
        picture.set_can_shrink(true);

        let file_name = std::path::Path::new(&image_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&image_path)
            .to_string();

        let container = GtkBox::new(Orientation::Vertical, 0);
        container.add_css_class("resizable-image-container");
        
        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&picture));
        container.append(&overlay);

        let resizable = Rc::new(Self {
            container,
            picture: picture.clone(),
            image_path,
            file_name,
            width: RefCell::new(initial_width),
            height: RefCell::new(initial_height),
            drag_start: RefCell::new(None),
            on_size_change: RefCell::new(None),
        });

        resizable.picture.set_width_request(initial_width);
        resizable.picture.set_height_request(initial_height);
        resizable.container.set_size_request(initial_width, initial_height + HANDLE_SIZE as i32);

        let handle = DrawingArea::new();
        handle.set_size_request(HANDLE_SIZE as i32, HANDLE_SIZE as i32);
        handle.add_css_class("resize-handle");
        handle.set_halign(gtk4::Align::End);
        handle.set_valign(gtk4::Align::End);
        overlay.add_overlay(&handle);

        let drag = GestureDrag::new();
        let resizable_for_begin = resizable.clone();
        let handle_for_begin = handle.clone();
        
        drag.connect_drag_begin(move |gesture, x, y| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            let alloc = handle_for_begin.allocation();
            let start_x = alloc.width() as f64 - x;
            let start_y = alloc.height() as f64 - y;
            *resizable_for_begin.drag_start.borrow_mut() = Some((start_x, start_y));
        });

        let resizable_for_update = resizable.clone();
        drag.connect_drag_update(move |_gesture, dx, dy| {
            if let Some((start_x, start_y)) = *resizable_for_update.drag_start.borrow() {
                let new_width = (*resizable_for_update.width.borrow() as f64 + dx + start_x - HANDLE_SIZE).round() as i32;
                let new_height = (*resizable_for_update.height.borrow() as f64 + dy + start_y - HANDLE_SIZE).round() as i32;
                
                let new_width = new_width.clamp(MIN_SIZE, MAX_SIZE);
                let new_height = new_height.clamp(MIN_SIZE, MAX_SIZE);
                
                resizable_for_update.set_size(new_width, new_height);
            }
        });

        let resizable_for_end = resizable.clone();
        drag.connect_drag_end(move |_gesture, _dx, _dy| {
            *resizable_for_end.drag_start.borrow_mut() = None;
        });

        handle.add_controller(drag);

        let motion = EventControllerMotion::new();
        let handle_clone = handle.clone();
        motion.connect_motion(move |_, x, y| {
            let alloc = handle_clone.allocation();
            let near_edge = (alloc.width() as f64 - x).abs() < 20.0 && (alloc.height() as f64 - y).abs() < 20.0;
            if near_edge {
                handle_clone.add_css_class("hover");
            } else {
                handle_clone.remove_css_class("hover");
            }
        });
        handle.add_controller(motion);

        handle.set_draw_func({
            let picture = picture.clone();
            move |_area, cr, width, height| {
                let has_texture = picture.paintable().map(|p| p.current_image()).is_some();
                if !has_texture {
                    return;
                }
                cr.set_source_rgb(0.3, 0.3, 0.3);
                cr.set_line_width(2.0);
                cr.move_to(width as f64 - 12.0, height as f64 - 2.0);
                cr.line_to(width as f64 - 2.0, height as f64 - 2.0);
                cr.line_to(width as f64 - 2.0, height as f64 - 12.0);
                cr.stroke().ok();
            }
        });

        resizable
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn set_size(&self, width: i32, height: i32) {
        let width = width.clamp(MIN_SIZE, MAX_SIZE);
        let height = height.clamp(MIN_SIZE, MAX_SIZE);
        
        *self.width.borrow_mut() = width;
        *self.height.borrow_mut() = height;
        
        self.picture.set_width_request(width);
        self.picture.set_height_request(height);
        self.container.set_size_request(width, height + HANDLE_SIZE as i32);
        
        if let Some(callback) = self.on_size_change.borrow().as_ref() {
            callback(&self.file_name, width, height);
        }
    }

    pub fn connect_size_change<F: Fn(&str, i32, i32) + 'static>(&self, f: F) {
        *self.on_size_change.borrow_mut() = Some(Box::new(f));
    }
}