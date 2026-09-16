pub mod latex;
pub mod markdown;
pub mod yaml;

use crate::editor::tags::EditorTags;
use gtk::prelude::*;
use sourceview5::Buffer;
use std::cell::RefCell;
use std::rc::Rc;

pub fn setup_linter(buffer: &Buffer, tags: Rc<EditorTags>) {
    let source_id = Rc::new(RefCell::new(None::<glib::SourceId>));

    buffer.connect_changed(move |buf| {
        let mut id = source_id.borrow_mut();

        if let Some(source) = id.take() {
            source.remove();
        }

        let buf_clone = buf.clone();
        let tags_clone = tags.clone();

        *id = Some(glib::timeout_add_local(
            std::time::Duration::from_millis(200),
            move || {
                lint_buffer(&buf_clone, &tags_clone);
                glib::ControlFlow::Break
            },
        ));
    });
}

fn lint_buffer(buffer: &Buffer, tags: &EditorTags) {
    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
    let text = text.as_str();

    tags.clear_errors(buffer);

    let mut errors = Vec::new();
    errors.extend(yaml::lint_yaml(text));
    errors.extend(latex::lint_latex(text));
    errors.extend(markdown::lint_markdown(text));

    for err in errors {
        let mut start_iter = buffer.start_iter();
        start_iter.set_offset(err.start as i32);

        let mut end_iter = buffer.start_iter();
        end_iter.set_offset(err.end as i32);

        tags.apply_error(buffer, &start_iter, &end_iter);
    }
}
