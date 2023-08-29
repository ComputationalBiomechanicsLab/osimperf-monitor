use super::read_config;
use anyhow::{ensure, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

// Search for "file_name" in directory and subdirectories.
pub fn find_file_by_name(root_dir: &Path, file_name: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(root_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                result.extend(find_perf_test_setup_files(&entry_path));
            } else if entry_path.file_name() == Some(TEST_SETUP_FILE_NAME.as_ref()) {
                result.push(entry_path);
            }
        }
    }
    result
}

// Search for "file_name" in directory and subdirectories and read config.
pub fn collect_configs<C: Deserialize>(root_dir: &Path, file_name: &str) -> Result<Vec<C>> {
    Ok(
        Some(find_file_by_name(root_dir, file_name).map(|p| read_config::<C>(p)))
            .filter(|vec| vec.iter().all(|x| x.is_ok()))
            .context("failed to parse all config files")?
            .map(|x| x.unwrap()),
    )
}
