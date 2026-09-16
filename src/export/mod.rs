use std::path::{Path, PathBuf};
use std::fs;
use std::env;

pub mod pandoc;
pub mod typst;

pub fn get_embedded_binary(name: &str) -> PathBuf {
    let mut temp_dir = env::temp_dir();
    temp_dir.push("biscuit_binaries");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).unwrap();
    }
    
    let binary_path = temp_dir.join(name);
    
    // In a full implementation, we'd use include_bytes! from OUT_DIR here
    // For this mockup, we just assume the path exists or touch it
    if !binary_path.exists() {
        fs::write(&binary_path, b"dummy binary").unwrap();
        // Make executable (unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms).unwrap();
        }
    }
    
    binary_path
}
