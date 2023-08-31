use crate::common::visit_dirs;
use crate::{erase_folder, ResultsFolder};
use anyhow::Result;
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

pub fn setup_context(setup_dir: &Path, context_dir: &Path) -> Result<()> {
    // Erase scratch dir.
    erase_folder(&context_dir)?;

    // Copy all files from setup directory.
    trace!("setup dir = {:?}", setup_dir);
    visit_dirs(setup_dir, &|entry| {
        filter_copy_file_to_context(entry, context_dir)
    })?;

    Ok(())
}
