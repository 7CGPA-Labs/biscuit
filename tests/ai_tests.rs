use biscuit::ai::worker::AiWorker;
use biscuit::ai::models;
use std::sync::Arc;

#[test]
fn test_fix_grammar_paragraph() {
    let worker = Arc::new(AiWorker::new());
    
    // Check if download is needed
    if biscuit::ai::downloader::needs_download() {
        let (ptx, _) = std::sync::mpsc::channel();
        let (ttx, _) = std::sync::mpsc::channel();
        biscuit::ai::downloader::check_and_download_models(ptx, ttx);
    }
    
    let original = "Their is many mistake in this sentences, it dont looks good at all. I was going to the store yesterday but I forgets my wallet at home so I has to go back. When I getted back, the store was already close for the night and I didn't got to buy anything I needs. It were a very bad day for me and I is very sad.";
    let res = models::fix_grammar(&worker, original).unwrap();
    println!("GRAMMAR RESULT:\n>{}<", res);
}
