use std::fs;
use std::path::Path;
use std::io;

pub const CASES_ROOT: &str = "assets/Cases";
pub const DOCUMENTS_ROOT: &str = "assets/documents";

/// Copies a case folder from assets/Cases to assets/documents
/// 
/// # Arguments
/// * `case_name` - The name of the case folder to copy
pub fn load_case(case_name: &str) -> io::Result<()> {
    let source = Path::new(CASES_ROOT).join(case_name);
    let destination = Path::new(DOCUMENTS_ROOT).join(case_name);

    if !source.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Case not found in assets/Cases: {}", case_name),
        ));
    }

    copy_dir_recursive(&source, &destination)
}

/// Recursively copies a directory and its contents
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let destination_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &destination_path)?;
        } else {
            fs::copy(entry.path(), &destination_path)?;
        }
    }
    Ok(())
}
