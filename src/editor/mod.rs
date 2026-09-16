pub mod tags;

use gtk::prelude::*;
use gtk::Widget;
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, View};

pub struct Editor {
    pub view: View,
    pub buffer: Buffer,
}

impl Editor {
    pub fn new() -> Self {
        let language_manager = LanguageManager::default();
        let language = language_manager.language("markdown");

        let buffer = Buffer::builder()
            .language(&language.expect("Markdown language not found"))
            .highlight_syntax(true)
            .build();

        let style_manager = sourceview5::StyleSchemeManager::default();
        if let Some(scheme) = style_manager
            .scheme("Adwaita-dark")
            .or_else(|| style_manager.scheme("oblivion"))
        {
            buffer.set_style_scheme(Some(&scheme));
        }

        let view = View::builder()
            .buffer(&buffer)
            .show_line_numbers(true)
            .wrap_mode(gtk::WrapMode::Word)
            .auto_indent(true)
            .insert_spaces_instead_of_tabs(true)
            .tab_width(4)
            .css_classes(["editor"])
            .build();

        Self { view, buffer }
    }

    pub fn widget(&self) -> &Widget {
        self.view.upcast_ref()
    }
}
