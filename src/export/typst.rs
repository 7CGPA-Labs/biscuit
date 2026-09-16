use std::process::Command;
use crate::export::get_embedded_binary;

pub fn export_to_pdf(input_file: &str, output_file: &str) -> Result<(), String> {
    let typst_bin = get_embedded_binary("typst");
    
    let status = Command::new(typst_bin)
        .args(&["compile", input_file, output_file])
        .status()
        .map_err(|e| e.to_string())?;
        
    if status.success() {
        Ok(())
    } else {
        Err(format!("Typst failed with status: {}", status))
    }
}
