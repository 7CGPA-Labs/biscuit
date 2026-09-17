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
            println!(
                "cargo:warning=Failed to download {}. Creating a dummy binary instead.",
                binary_name
            );
            fs::write(out_path, b"dummy binary").unwrap();
            return;
        }
    };

    let bytes = match resp.bytes() {
        Ok(b) => b,
        _ => {
            println!(
                "cargo:warning=Failed to read bytes for {}. Creating a dummy binary.",
                binary_name
            );
            fs::write(out_path, b"dummy binary").unwrap();
            return;
        }
    };

    // We'll write to a temp file and extract using system tar to keep it simple and reliable
    let temp_archive = out_path.with_extension(archive_type);
    fs::write(&temp_archive, &bytes).expect("Failed to write temp archive");

    let temp_dir = out_path
        .parent()
        .unwrap()
        .join(format!("temp_{}", binary_name));
    fs::create_dir_all(&temp_dir).unwrap();

    let status = Command::new("tar")
        .args(&[
            "xf",
            temp_archive.to_str().unwrap(),
            "-C",
            temp_dir.to_str().unwrap(),
        ])
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

fn download_katex(out_dir: &PathBuf) {
    let katex_dir = out_dir.join("katex");
    let version_file = out_dir.join("katex_version.txt");
    let tag_name = "v0.16.11"; // hardcoded version

    if version_file.exists() && katex_dir.exists() {
        if let Ok(cached) = fs::read_to_string(&version_file) {
            if cached == tag_name {
                return; // Already up to date
            }
        }
    }

    let tarball_url = format!(
        "https://github.com/KaTeX/KaTeX/releases/download/{}/katex.tar.gz",
        tag_name
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("biscuit-build")
        .build()
        .unwrap();

    let bytes = match client.get(&tarball_url).send() {
        Ok(r) if r.status().is_success() => r.bytes().unwrap(),
        _ => {
            println!("cargo:warning=Failed to download KaTeX. Creating dummy files.");
            fs::create_dir_all(katex_dir.join("katex/contrib")).unwrap();
            fs::write(katex_dir.join("katex/katex.min.css"), b"").unwrap();
            fs::write(katex_dir.join("katex/katex.min.js"), b"").unwrap();
            fs::write(katex_dir.join("katex/contrib/auto-render.min.js"), b"").unwrap();
            return;
        }
    };

    let temp_archive = out_dir.join("katex.tar.gz");
    fs::write(&temp_archive, &bytes).unwrap();

    if katex_dir.exists() {
        fs::remove_dir_all(&katex_dir).unwrap();
    }
    fs::create_dir_all(&katex_dir).unwrap();

    let status = Command::new("tar")
        .args(&[
            "xf",
            temp_archive.to_str().unwrap(),
            "-C",
            katex_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run tar");

    assert!(status.success(), "Tar extraction failed for KaTeX");

    fs::remove_file(temp_archive).unwrap();
    fs::write(version_file, tag_name).unwrap();
}

fn build_latex_renderer() {
    let renderer_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("latex_renderer");
    if !renderer_dir.exists() {
        return;
    }

    // Run npm install
    let status = Command::new("npm")
        .current_dir(&renderer_dir)
        .args(&["install"])
        .status()
        .expect("Failed to run npm install");
    assert!(status.success(), "npm install failed in latex_renderer");

    // Run npm run build
    let status = Command::new("npm")
        .current_dir(&renderer_dir)
        .args(&["run", "build"])
        .status()
        .expect("Failed to run npm run build");
    assert!(status.success(), "npm run build failed in latex_renderer");

    // Tar the dist folder to OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tar_path = out_dir.join("latex_renderer.tar.gz");
    let status = Command::new("tar")
        .current_dir(renderer_dir.join("dist"))
        .args(&["-czf", tar_path.to_str().unwrap(), "."])
        .status()
        .expect("Failed to create tar archive");
    assert!(status.success(), "Failed to create latex_renderer.tar.gz");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=latex_renderer/src/index.js");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let pandoc_path = out_dir.join("pandoc");
    let typst_path = out_dir.join("typst");

    download_and_extract(
        "https://github.com/jgm/pandoc/releases/download/3.1.13/pandoc-3.1.13-linux-amd64.tar.gz",
        &pandoc_path,
        "pandoc",
        "tar.gz",
    );

    download_and_extract(
        "https://github.com/typst/typst/releases/download/v0.11.0/typst-x86_64-unknown-linux-musl.tar.xz",
        &typst_path,
        "typst",
        "tar.xz"
    );

    download_katex(&out_dir);
    build_latex_renderer();
}
