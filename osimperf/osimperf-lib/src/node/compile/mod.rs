mod cmake;
mod config;
mod progress;

use std::{fs::rename, time::Duration};

use anyhow::{anyhow, Context };
pub use cmake::*;
pub use config::CMakeConfig;
use log::{info, warn};
pub use progress::ProgressStreamer;

use crate::{erase_folder, BuildFolder};

use self::cmake::CMakeCmds;

use super::{Focus, Id, Source};

pub fn run_cmake_compilation<'a>(
    id: Id<'a>,
    source: Source<'a>,
    build: &BuildFolder,
    config: &CMakeConfig,
    progress: &mut ProgressStreamer,
    focus_it: &[Option<Focus>; 3],
) -> anyhow::Result<[anyhow::Result<Duration>; 3]> {
    let mut res = [
        Err(anyhow!("not started compiling")),
        Err(anyhow!("not started compiling")),
        Err(anyhow!("not started compiling")),
    ];
    for i in 0..3 {
        if let Some(focus) = focus_it[i] {
            let install_dir = id.path().join(focus.to_str());
            erase_folder(&install_dir)
                .with_context(|| format!("failed to erase install dir: {:?}", install_dir))?;

            let cmd = CMakeCmds::new(&id, &source, build, config, focus)?;
            println!("RUNN {}", cmd.print_pretty());

            res[i] = cmd
                .run(progress)
                .with_context(|| format!("cmake failed: {:#?}", cmd.print_pretty()));

            if res[i].is_err() {
                warn!("cmake failed with errors:");
                for r in res.iter() {
                    warn!("{:?}", r);
                }
            }
        }
    }
    Ok(res)
}
