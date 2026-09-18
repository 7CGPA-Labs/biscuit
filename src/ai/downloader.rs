use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::io::Write;

const MODELS: &[(&str, &str)] = &[
    ("minilm-l6-v2.onnx", "https://huggingface.co/Xenova/all-MiniLM-L6-v2/resolve/main/onnx/model_quantized.onnx"),
    ("t5-small.onnx", "https://huggingface.co/Xenova/t5-small/resolve/main/onnx/model_quantized.onnx"),
    ("smollm2-135m.onnx", "https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct/resolve/main/onnx/model_quantized.onnx"), 
];

pub fn get_models_dir() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "biscuit", "clippytext").unwrap();
    let models_dir = proj_dirs.data_local_dir().join("models");
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir).unwrap();
    }
    models_dir
}

pub fn needs_download() -> bool {
    let models_dir = get_models_dir();
    for (filename, _) in MODELS {
        let model_path = models_dir.join(filename);
        if !model_path.exists() || fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0) < 1000 {
            return true;
        }
    }
    false
}

pub fn check_and_download_models(progress_tx: Sender<f64>, text_tx: Sender<String>) {
    let models_dir = get_models_dir();
    let client = reqwest::blocking::Client::new();

    let mut to_download = Vec::new();
    for (filename, url) in MODELS {
        let model_path = models_dir.join(filename);
        if !model_path.exists() || fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0) < 1000 {
            to_download.push((filename, url, model_path));
        }
    }

    let total_files = to_download.len();
    if total_files == 0 {
        let _ = text_tx.send("All models ready.".to_string());
        let _ = progress_tx.send(1.0);
        return;
    }

    for (i, (filename, url, model_path)) in to_download.iter().enumerate() {
        let _ = text_tx.send(format!("Downloading {}...", filename));
        
        let mut response = match client.get(**url).send() {
            Ok(r) if r.status().is_success() => r,
            _ => {
                let _ = text_tx.send(format!("Failed to connect for {}. Creating dummy.", filename));
                fs::write(&model_path, b"").unwrap();
                continue;
            }
        };

        let total_size = response.content_length().unwrap_or(0) as f64;
        let mut file = fs::File::create(&model_path).unwrap();
        let mut downloaded: u64 = 0;
        let mut buffer = [0; 8192];
        
        loop {
            let bytes_read = match std::io::Read::read(&mut response, &mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            
            file.write_all(&buffer[..bytes_read]).unwrap();
            downloaded += bytes_read as u64;
            
            if total_size > 0.0 {
                let current_file_progress = (downloaded as f64) / total_size;
                let overall_progress = (i as f64 + current_file_progress) / (total_files as f64);
                let _ = progress_tx.send(overall_progress);
            }
        }
    }

    let _ = text_tx.send("Download complete.".to_string());
    let _ = progress_tx.send(1.0);
}
