use biscuit::linter::yaml;
use biscuit::linter::latex;

#[test]
fn test_yaml_linter_valid() {
    let text = "---\ntitle: Valid\n---\nHello";
    let errors = yaml::lint_yaml(text);
    assert!(errors.is_empty());
}

#[test]
fn test_yaml_linter_invalid() {
    let text = "---\ntitle: [Invalid\n---\nHello";
    let errors = yaml::lint_yaml(text);
    assert!(!errors.is_empty());
}

#[test]
fn test_latex_linter_valid() {
    let text = "\\begin{equation}\nE = mc^2\n\\end{equation}";
    let errors = latex::lint_latex(text);
    assert!(errors.is_empty());
}

#[test]
fn test_latex_linter_mismatched() {
    let text = "\\begin{equation}\nE = mc^2\n\\end{align}";
    let errors = latex::lint_latex(text);
    assert!(!errors.is_empty());
}

#[test]
fn test_latex_linter_unclosed() {
    let text = "\\begin{equation}\nE = mc^2";
    let errors = latex::lint_latex(text);
    assert!(!errors.is_empty());
}
