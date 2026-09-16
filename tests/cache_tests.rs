use biscuit::ai::models::compute_diff;

#[test]
fn test_diff_computation() {
    let original = "This is the original text.";
    let new = "This is the new and improved text.";

    let diff = compute_diff(original, new);
    assert!(!diff.is_empty());
}
