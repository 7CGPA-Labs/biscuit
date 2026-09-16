use crate::ai::worker::AiWorker;
use similar::{ChangeTag, TextDiff};

pub enum Intent {
    FixGrammar,
    InlineCompletion,
    FormatTable,
    Rephrase,
    Unknown,
}

pub fn route_intent(_worker: &AiWorker, _text: &str) -> Intent {
    // In a real implementation, this would tokenize text and run minilm_session
    // to classify intent. Here we mock it based on simple rules or return Unknown.
    Intent::Unknown
}

pub fn fix_grammar(_worker: &AiWorker, original_text: &str) -> String {
    // Mock inference
    // Real implementation uses t5_session
    original_text.to_string()
}

pub fn ghostwrite(_worker: &AiWorker, prefix: &str) -> String {
    // Mock inference
    // Real implementation uses smollm_session
    format!("{} [ghostwritten continuation]", prefix)
}

pub fn compute_diff(original: &str, new: &str) -> Vec<(ChangeTag, String)> {
    let diff = TextDiff::from_words(original, new);
    diff.iter_all_changes()
        .map(|change| (change.tag(), change.value().to_string()))
        .collect()
}
