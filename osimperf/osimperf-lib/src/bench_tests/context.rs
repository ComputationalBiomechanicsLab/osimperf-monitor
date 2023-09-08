use crate::common::{find_file_by_name, visit_dirs};
use crate::erase_folder;
use anyhow::{anyhow, Result};
use log::trace;
use std::fs::copy;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

pub fn flattened_path(file: &Path, flat_dir: &Path) -> PathBuf {
    flat_dir.join(file.file_name().unwrap())
}

pub fn copy_file_to_flat(file: &Path, flat_dir: &Path) -> Result<()> {
    let to = flattened_path(file, &flat_dir);
    trace!("Context setup: Copy {:?} to {:?}", file, to);
    let _ = copy(file, &to)?;
    Ok(())
}

pub fn filter_copy_file_to_context(entry: &DirEntry, context: &Path) {
    let file = entry.path();
    if file.file_name().unwrap() != "osimperf-test.conf" {
        copy_file_to_flat(&file, context).unwrap();
    }
}

pub fn setup_context(
    setup_dir: &Path,
    context_dir: &Path,
    required_files: &[String],
    models_dir: &Path,
) -> Result<()> {
    // Erase scratch dir.
    erase_folder(&context_dir)?;

    // Copy all files from setup directory.
    trace!("setup dir = {:?}", setup_dir);
    visit_dirs(setup_dir, &mut |entry| {
        filter_copy_file_to_context(entry, context_dir)
    })?;

    for required_file in required_files.iter() {
        let file = find_modeling_file(required_file, &models_dir)?;
        copy_file_to_flat(&file, context_dir)?;
    }

    Ok(())
}

pub fn find_modeling_file(required_file: &str, search_dir: &Path) -> Result<PathBuf> {
    let mut search_results = find_file_by_name(search_dir, required_file);
    match search_results.len() {
        0 => Err(anyhow!(format!(
            "unable to find required file {} in directory {:?}",
            required_file, search_dir
        )))?,
        1 => return Ok(search_results.drain(..).next().unwrap()),
        len => Err(anyhow!(format!(
            "Found {} files matching {} in directory {:?}",
            len, required_file, search_dir
        )))?,
    }
}
