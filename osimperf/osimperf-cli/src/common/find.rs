use std::path::{Path, PathBuf};
use std::fs;

// Search for "file_name" in directory and subdirectories.
pub fn find_file_by_name(root_dir: &Path, file_name: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(root_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                result.extend(find_file_by_name(&entry_path, file_name));
            } else if entry_path.file_name().and_then(|f| f.to_str()) == Some(file_name) {
                result.push(entry_path);
            }
        }
    }
    result
}
