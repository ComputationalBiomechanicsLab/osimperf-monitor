use super::read_config;
use anyhow::Result;
use serde::de::DeserializeOwned;
use std::{
    fs,
    path::{Path, PathBuf},
};

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

// Search for "file_name" in directory and subdirectories and read config.
pub fn collect_configs<C: DeserializeOwned>(root_dir: &Path, file_name: &str) -> Result<Vec<C>> {
    let mut out = Vec::new();
    for config in find_file_by_name(root_dir, file_name)
        .drain(..)
        .map(|p| read_config::<C>(&p))
    {
        out.push(config?);
    }
    Ok(out)
}
