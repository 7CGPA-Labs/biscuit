use super::markdown::LintError;

pub fn lint_yaml(text: &str) -> Vec<LintError> {
    let mut errors = Vec::new();
    
    if text.starts_with("---\n") {
        if let Some(end_idx) = text[4..].find("\n---\n") {
            let frontmatter = &text[4..4 + end_idx];
            if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                if let Some(loc) = e.location() {
                    let mut start = 4;
                    let mut lines = frontmatter.lines();
                    for _ in 1..loc.line() {
                        if let Some(l) = lines.next() {
                            start += l.len() + 1;
                        }
                    }
                    start += loc.column().saturating_sub(1);
                    
                    errors.push(LintError {
                        start,
                        end: start + 1,
                        message: format!("YAML Error: {}", e),
                    });
                } else {
                    errors.push(LintError {
                        start: 4,
                        end: 4 + end_idx,
                        message: format!("YAML Error: {}", e),
                    });
                }
            }
        }
    }
    
    errors
}
