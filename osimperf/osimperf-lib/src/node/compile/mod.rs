mod cmake;
mod config;
mod progress;

use std::{fs::rename, time::Duration};

use anyhow::{anyhow, Context, Result};
pub use cmake::*;
pub use config::CMakeConfig;
use log::{info, warn};
pub use progress::ProgressStreamer;

use crate::{erase_folder, BuildFolder, Folder, State};

use self::cmake::CMakeCmds;

use super::{status::Status, Focus, Id, Source};

pub fn run_cmake_compilation<'a>(
    id: Id<'a>,
    source: Source<'a>,
    build: &BuildFolder,
    config: &CMakeConfig,
    progress: &mut ProgressStreamer,
    state: &State,
) -> anyhow::Result<State> {
    let mut out = state.clone();
    for (i, s) in state
        .get()
        .iter()
        .enumerate()
        .filter(|(i, s)| s.should_compile())
    {
        let focus = Focus::from(i);
        let install_dir = id.path().join(focus.to_str());
        erase_folder(&install_dir)
            .with_context(|| format!("failed to erase install dir: {:?}", install_dir))?;

        if let Ok(cmd) = CMakeCmds::new(&id, &source, build, config, focus) {
            erase_folder(&build.path()?.join(focus.to_str()))
                .with_context(|| format!("failed to erase build dir"))?;

            let output = cmd
                .run(progress)
                .with_context(|| format!("cmake failed: {:#?}", cmd.print_pretty()));

            out.set(focus, Status::from_output(output));
        }
    }
    Ok(out)
}
