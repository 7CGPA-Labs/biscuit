use gtk::prelude::*;
use gtk::{Box, Label, Orientation, Align};

pub struct StatusBar {
    pub widget: Box,
    pub location_label: Label,
    pub type_label: Label,
}

impl StatusBar {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Horizontal, 20);
        container.set_margin_start(10);
        container.set_margin_end(10);
        container.set_margin_top(2);
        container.set_margin_bottom(2);
        container.add_css_class("dim-label"); // Make it look subtle
        
        let location_label = Label::builder()
            .label("Ln 1, Col 1")
            .halign(Align::Start)
            .build();
            
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
            
        let type_label = Label::builder()
            .label("Markdown")
            .halign(Align::End)
            .build();
            
        container.append(&location_label);
        container.append(&spacer);
        container.append(&type_label);
        
        Self {
            widget: container,
            location_label,
            type_label,
        }
    }
}
