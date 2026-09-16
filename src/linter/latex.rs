use super::markdown::LintError;

pub fn lint_latex(text: &str) -> Vec<LintError> {
    let mut errors = Vec::new();
    
    // Very basic environment matcher
    let mut env_stack: Vec<(String, usize)> = Vec::new();
    let mut i = 0;
    
    while let Some(idx) = text[i..].find("\\begin{") {
        let abs_idx = i + idx;
        if let Some(end_brace) = text[abs_idx..].find('}') {
            let env_name = &text[abs_idx + 7..abs_idx + end_brace];
            env_stack.push((env_name.to_string(), abs_idx));
            i = abs_idx + end_brace + 1;
        } else {
            break;
        }
    }
    
    i = 0;
    while let Some(idx) = text[i..].find("\\end{") {
        let abs_idx = i + idx;
        if let Some(end_brace) = text[abs_idx..].find('}') {
            let env_name = &text[abs_idx + 5..abs_idx + end_brace];
            if let Some((top_env, top_idx)) = env_stack.pop() {
                if top_env != env_name {
                    errors.push(LintError {
                        start: top_idx,
                        end: abs_idx + end_brace + 1,
                        message: format!("Mismatched LaTeX environment: expected \\end{{{}}}, found \\end{{{}}}", top_env, env_name),
                    });
                }
            } else {
                errors.push(LintError {
                    start: abs_idx,
                    end: abs_idx + end_brace + 1,
                    message: format!("Unmatched LaTeX environment: \\end{{{}}}", env_name),
                });
            }
            i = abs_idx + end_brace + 1;
        } else {
            break;
        }
    }
    
    for (env, idx) in env_stack {
        errors.push(LintError {
            start: idx,
            end: idx + 7 + env.len() + 1,
            message: format!("Unclosed LaTeX environment: \\begin{{{}}}", env),
        });
    }
    
    errors
}
