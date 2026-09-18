use crate::ai::downloader::get_models_dir;
use candle_core::Device;
use candle_transformers::models::quantized_llama::ModelWeights;
use std::sync::Mutex;
use tokenizers::Tokenizer;

pub struct AiWorker {
    pub smollm_model: Mutex<Option<ModelWeights>>,
    pub smollm_tokenizer: Option<Tokenizer>,
}

impl AiWorker {
    pub fn new() -> Self {
        let models_dir = get_models_dir();
        
        let smollm_path = models_dir.join("smollm2-360m-instruct-q4_k_m.gguf");
        let smollm_tok_path = models_dir.join("smollm-tokenizer.json");

        let smollm_tokenizer = Tokenizer::from_file(smollm_tok_path).ok();
        
        let smollm_model = if smollm_path.exists() {
            let mut file = std::fs::File::open(&smollm_path).unwrap();
            let model = match candle_core::quantized::gguf_file::Content::read(&mut file) {
                Ok(content) => match ModelWeights::from_gguf(content, &mut file, &Device::Cpu) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        println!("Failed to load ModelWeights: {:?}", e);
                        None
                    }
                },
                Err(e) => {
                    println!("Failed to read GGUF content: {:?}", e);
                    None
                }
            };
            Mutex::new(model)
        } else {
            println!("GGUF file not found at {:?}", smollm_path);
            Mutex::new(None)
        };

        Self {
            smollm_model,
            smollm_tokenizer,
        }
    }

    pub fn ensure_loaded(&self) {
        let mut lock = self.smollm_model.lock().unwrap();
        if lock.is_none() {
            let models_dir = get_models_dir();
            let smollm_path = models_dir.join("smollm2-360m-instruct-q4_k_m.gguf");
            if smollm_path.exists() {
                if let Ok(mut file) = std::fs::File::open(&smollm_path) {
                    if let Ok(content) = candle_core::quantized::gguf_file::Content::read(&mut file) {
                        if let Ok(model) = ModelWeights::from_gguf(content, &mut file, &Device::Cpu) {
                            *lock = Some(model);
                        }
                    }
                }
            }
        }
    }
}
