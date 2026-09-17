use pulldown_cmark::{Event, Parser};

pub struct LintError {
    pub start: usize,
    pub end: usize,
    pub message: String,
}

pub fn lint_markdown(text: &str) -> Vec<LintError> {
    let mut errors = Vec::new();
    let parser = Parser::new(text);

    // Just a simple example: finding broken links or malformed blocks
    // In pulldown-cmark, if a link is malformed, it often just parses as text.
    // For this mock implementation, we'll look for unmatched Markdown blocks
    // or placeholder errors just to satisfy the architecture.

    // Real implementation would track open/close tags and ensure correctness,
    // or check links if they resolve.
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Html(html) if html.contains("TODO:") => {
                errors.push(LintError {
                    start: range.start,
                    end: range.end,
                    message: "TODO comment found".to_string(),
                });
            }
            // More comprehensive linting could go here
            _ => {}
        }
    }

    errors
}
