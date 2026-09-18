use crate::ai::worker::AiWorker;
use candle_core::Tensor;
use similar::{ChangeTag, TextDiff};

#[derive(Debug, PartialEq)]
pub enum Intent {
    FixGrammar,
    Ghostwrite,
    Unknown,
}



fn generate_text(worker: &AiWorker, prompt: &str, max_tokens: usize) -> Result<String, String> {
    worker.ensure_loaded();
    let mut model_lock = worker.smollm_model.lock().unwrap();
    let model = match model_lock.as_mut() {
        Some(m) => m,
        None => return Err("AI model is not loaded yet or failed to load. Please wait for download to finish.".to_string()),
    };
    
    // Load tokenizer locally if not present in worker
    let local_tok;
    let tokenizer = match &worker.smollm_tokenizer {
        Some(t) => t,
        None => {
            let path = crate::ai::downloader::get_models_dir().join("smollm-tokenizer.json");
            if let Ok(t) = tokenizers::Tokenizer::from_file(path) {
                local_tok = t;
                &local_tok
            } else {
                return Err("AI tokenizer is not loaded yet or failed to load.".to_string());
            }
        }
    };

    let tokens = tokenizer.encode(prompt, true).unwrap();
    let mut all_tokens = tokens.get_ids().to_vec();
    
    let eos_token = tokenizer.token_to_id("<|im_end|>").unwrap_or(2);
    
    let mut generated_tokens = Vec::new();
    let repeat_penalty = 1.15f32;
    let repeat_last_n = 64;

    for i in 0..max_tokens {
        let (input_tensor, index_pos) = if i == 0 {
            (Tensor::new(&all_tokens[..], &candle_core::Device::Cpu).unwrap().unsqueeze(0).unwrap(), 0)
        } else {
            let last_token = *all_tokens.last().unwrap();
            (Tensor::new(&[last_token], &candle_core::Device::Cpu).unwrap().unsqueeze(0).unwrap(), all_tokens.len() - 1)
        };

        let logits = model.forward(&input_tensor, index_pos).unwrap();
        let mut logits = logits.squeeze(0).unwrap();
        
        // Apply repetition penalty
        if !generated_tokens.is_empty() {
            let start_idx = generated_tokens.len().saturating_sub(repeat_last_n);
            let ctx = &generated_tokens[start_idx..];
            
            // We use simple CPU iteration for repetition penalty
            let mut logits_vec = logits.to_vec1::<f32>().unwrap();
            for &token in ctx {
                let token_idx = token as usize;
                if token_idx < logits_vec.len() {
                    let logit = logits_vec[token_idx];
                    if logit > 0.0 {
                        logits_vec[token_idx] = logit / repeat_penalty;
                    } else {
                        logits_vec[token_idx] = logit * repeat_penalty;
                    }
                }
            }
            logits = Tensor::new(logits_vec, &candle_core::Device::Cpu).unwrap();
        }
        
        let next_token = logits.argmax(0).unwrap().to_scalar::<u32>().unwrap();

        if next_token == eos_token {
            break;
        }
        
        let decoded_char = tokenizer.decode(&[next_token], true).unwrap_or_default();
        if decoded_char == "\n" && prompt.contains("Fix the grammar") {
            // Early exit for grammar fix if model outputs newline
            break;
        }

        generated_tokens.push(next_token);
        all_tokens.push(next_token);
    }

    let decoded = tokenizer.decode(&generated_tokens, true).unwrap_or_default();
    if decoded.trim().is_empty() {
        Ok(String::new())
    } else {
        Ok(decoded)
    }
}

pub fn fix_grammar(worker: &AiWorker, original_text: &str) -> Result<String, String> {
    let prompt = format!(
        "<|im_start|>system\nYou are a grammar editor. Fix the grammar of the provided text. Output ONLY the corrected text.<|im_end|>\n<|im_start|>user\nText: I goes to the store yesterday.<|im_end|>\n<|im_start|>assistant\nI went to the store yesterday.<|im_end|>\n<|im_start|>user\nText: {}<|im_end|>\n<|im_start|>assistant\n",
        original_text
    );
    
    let generated = generate_text(worker, &prompt, 150)?;
    let trimmed = generated.trim().trim_matches('"').to_string();
    if trimmed.is_empty() {
        Ok(original_text.to_string())
    } else {
        Ok(trimmed)
    }
}

pub fn ghostwrite(worker: &AiWorker, prefix: &str) -> Result<String, String> {
    let prompt = format!(
        "<|im_start|>system\nYou are an AI assistant helping the user write a document.<|im_end|>\n<|im_start|>user\nComplete this thought: {}<|im_end|>\n<|im_start|>assistant\n",
        prefix
    );
    
    let generated = generate_text(worker, &prompt, 150)?;
    let gen_trim = generated.trim();
    let prefix_clean = prefix.trim_end_matches("...").trim_end();
    
    // Some small models start generating by repeating the prefix.
    // If it does, we strip the prefix from the generated text.
    let mut final_generated = gen_trim;
    if final_generated.starts_with(prefix_clean) {
        final_generated = &final_generated[prefix_clean.len()..];
    } else if let Some(first_word) = prefix_clean.split_whitespace().next() {
        // sometimes it repeats just the last few words or the whole sentence without exact punctuation
        // For simple handling, we check if it starts with the same text ignoring case
        if final_generated.to_lowercase().starts_with(&prefix_clean.to_lowercase()) {
             final_generated = &final_generated[prefix_clean.len()..];
        }
    }
    
    Ok(format!("{} {}", prefix_clean, final_generated.trim()))
}

pub fn compute_diff(original: &str, new: &str) -> Vec<(ChangeTag, String)> {
    let diff = TextDiff::from_words(original, new);
    diff.iter_all_changes()
        .map(|change| (change.tag(), change.value().to_string()))
        .collect()
}
