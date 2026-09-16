use crate::ai::downloader::get_models_dir;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;

pub struct AiWorker {
    pub minilm_session: Option<Session>,
    pub t5_session: Option<Session>,
    pub smollm_session: Option<Session>,
}

impl AiWorker {
    pub fn new() -> Self {
        // ort v2 initializes automatically, no need to build Environment explicitly
        let _ = ort::init().commit();

        let models_dir = get_models_dir();

        let minilm_path = models_dir.join("minilm-l6-v2.onnx");
        let t5_path = models_dir.join("t5-small.onnx");
        let smollm_path = models_dir.join("smollm2-135m.onnx");

        let minilm_session = if minilm_path.exists() {
            Session::builder()
                .unwrap()
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .unwrap()
                .with_intra_threads(2)
                .unwrap()
                .commit_from_file(minilm_path)
                .ok()
        } else {
            None
        };

        let t5_session = if t5_path.exists() {
            Session::builder()
                .unwrap()
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .unwrap()
                .with_intra_threads(2)
                .unwrap()
                .commit_from_file(t5_path)
                .ok()
        } else {
            None
        };

        let smollm_session = if smollm_path.exists() {
            Session::builder()
                .unwrap()
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .unwrap()
                .with_intra_threads(2)
                .unwrap()
                .commit_from_file(smollm_path)
                .ok()
        } else {
            None
        };

        Self {
            minilm_session,
            t5_session,
            smollm_session,
        }
    }
}
