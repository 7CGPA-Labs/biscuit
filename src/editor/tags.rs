use gtk::prelude::*;
use sourceview5::Buffer;
use gtk::{TextTag, TextTagTable};

pub struct EditorTags {
    pub error_tag: TextTag,
}

impl EditorTags {
    pub fn new(buffer: &Buffer) -> Self {
        let tag_table = buffer.tag_table();
        
        // Create the error tag for red squiggly lines
        let error_tag = TextTag::builder()
            .name("error")
            .underline(gtk::pango::Underline::Error)
            .underline_rgba(&gtk::gdk::RGBA::new(1.0, 0.0, 0.0, 1.0))
            .build();
            
        tag_table.add(&error_tag);

        Self { error_tag }
    }
    
    pub fn apply_error(&self, buffer: &Buffer, start: &gtk::TextIter, end: &gtk::TextIter) {
        buffer.apply_tag(&self.error_tag, start, end);
    }
    
    pub fn clear_errors(&self, buffer: &Buffer) {
        let mut start = buffer.start_iter();
        let mut end = buffer.end_iter();
        buffer.remove_tag(&self.error_tag, &mut start, &mut end);
    }
}
