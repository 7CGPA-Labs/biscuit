use crate::export::get_embedded_binary;
use std::process::Command;

pub fn export_to_docx(input_file: &str, output_file: &str) -> Result<(), String> {
    let pandoc_bin = get_embedded_binary("pandoc");

    let status = Command::new(pandoc_bin)
        .args(&[input_file, "-o", output_file])
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Pandoc failed with status: {}", status))
    }
}
