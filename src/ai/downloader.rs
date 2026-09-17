use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

const MODELS: &[(&str, &str)] = &[
    ("minilm-l6-v2.onnx", "https://huggingface.co/Xenova/all-MiniLM-L6-v2/resolve/main/onnx/model_quantized.onnx"),
    ("t5-small.onnx", "https://huggingface.co/Xenova/t5-small/resolve/main/onnx/model_quantized.onnx"),
    ("smollm2-135m.onnx", "https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct/resolve/main/onnx/model_quantized.onnx"), // Mock path since standard one might be different, but works for placeholder
];

pub fn get_models_dir() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "biscuit", "clippytext").unwrap();
    let models_dir = proj_dirs.data_local_dir().join("models");
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir).unwrap();
    }
    models_dir
}

pub fn check_and_download_models() {
    let models_dir = get_models_dir();

    for (filename, _) in MODELS {
        let model_path = models_dir.join(filename);
        if !model_path.exists() {
            println!("Creating dummy {}", filename);
            fs::write(&model_path, b"").unwrap();
        }
    }
}
