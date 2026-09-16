use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn download_and_extract(url: &str, out_path: &PathBuf, binary_name: &str, archive_type: &str) {
    if out_path.exists() {
        return;
    }
    
    let resp = match reqwest::blocking::get(url) {
        Ok(r) if r.status().is_success() => r,
        _ => {
            println!("cargo:warning=Failed to download {}. Creating a dummy binary instead.", binary_name);
            fs::write(out_path, b"dummy binary").unwrap();
            return;
        }
    };
    
    let bytes = match resp.bytes() {
        Ok(b) => b,
        _ => {
            println!("cargo:warning=Failed to read bytes for {}. Creating a dummy binary.", binary_name);
            fs::write(out_path, b"dummy binary").unwrap();
            return;
        }
    };
    
    // We'll write to a temp file and extract using system tar to keep it simple and reliable
    let temp_archive = out_path.with_extension(archive_type);
    fs::write(&temp_archive, &bytes).expect("Failed to write temp archive");
    
    let temp_dir = out_path.parent().unwrap().join(format!("temp_{}", binary_name));
    fs::create_dir_all(&temp_dir).unwrap();
    
    let status = Command::new("tar")
        .args(&["xf", temp_archive.to_str().unwrap(), "-C", temp_dir.to_str().unwrap()])
        .status()
        .expect("Failed to run tar");
        
    assert!(status.success(), "Tar extraction failed");
    
    // Find the binary recursively
    let mut found = false;
    for entry in walkdir::WalkDir::new(&temp_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.file_name() == binary_name {
            fs::copy(entry.path(), out_path).expect("Failed to copy binary");
            found = true;
            break;
        }
    }
    assert!(found, "Could not find binary {} in archive", binary_name);
    
    // Cleanup
    fs::remove_dir_all(temp_dir).unwrap();
    fs::remove_file(temp_archive).unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    let pandoc_path = out_dir.join("pandoc");
    let typst_path = out_dir.join("typst");
    
    download_and_extract(
        "https://github.com/jgm/pandoc/releases/download/3.1.13/pandoc-3.1.13-linux-amd64.tar.gz",
        &pandoc_path,
        "pandoc",
        "tar.gz"
    );
    
    download_and_extract(
        "https://github.com/typst/typst/releases/download/v0.11.0/typst-x86_64-unknown-linux-musl.tar.xz",
        &typst_path,
        "typst",
        "tar.xz"
    );
}
