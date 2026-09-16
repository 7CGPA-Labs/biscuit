use gtk::prelude::*;
use gtk::{Box, Button, Label, Orientation, Popover};

pub struct ClippyBubble {
    pub widget: Popover,
    pub label: Label,
    pub action_box: Box,
}

impl ClippyBubble {
    pub fn new(parent: &impl IsA<gtk::Widget>) -> Self {
        let widget = Popover::new();
        widget.set_parent(parent);
        widget.set_position(gtk::PositionType::Left);

        let container = Box::new(Orientation::Vertical, 10);
        container.set_margin_start(10);
        container.set_margin_end(10);
        container.set_margin_top(10);
        container.set_margin_bottom(10);

        let label = Label::new(Some("How can I help?"));
        label.set_wrap(true);
        label.set_max_width_chars(30);
        container.append(&label);

        let action_box = Box::new(Orientation::Horizontal, 5);
        container.append(&action_box);

        widget.set_child(Some(&container));

        Self {
            widget,
            label,
            action_box,
        }
    }

    pub fn show_message(&self, text: &str) {
        self.label.set_text(text);

        // Remove old buttons
        while let Some(child) = self.action_box.first_child() {
            self.action_box.remove(&child);
        }

        self.widget.popup();
    }

    pub fn show_actions(
        &self,
        text: &str,
        actions: Vec<(&str, std::boxed::Box<dyn Fn() + 'static>)>,
    ) {
        self.label.set_text(text);

        while let Some(child) = self.action_box.first_child() {
            self.action_box.remove(&child);
        }

        for (label, callback) in actions {
            let btn = Button::with_label(label);
            btn.connect_clicked(move |_| {
                callback();
            });
            self.action_box.append(&btn);
        }

        self.widget.popup();
    }
}
