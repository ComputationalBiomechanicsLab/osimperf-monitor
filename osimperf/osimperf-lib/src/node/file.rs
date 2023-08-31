use log::{info, trace};
use std::{fmt::Debug, fs::create_dir};
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

use crate::{read_config, write_config};

// fn get_temp_file(path: &Path) -> PathBuf {
//     path.parent().unwrap().join("temp-conf")
// }

pub trait NodeFile: Serialize + DeserializeOwned + Debug {
    const SUBFOLDER_LEVEL: usize;

    fn path_to_self(&self) -> PathBuf;

    fn try_write(&self) -> anyhow::Result<()> {
        // let temp = get_temp_file(&self.path_to_self());
        // // write to temp.
        // trace!("writin to remporary file {:?}", temp);
        // write_config(&temp, &self).context("failed to write node file to temporary")?;
        // // Move to self.
        // trace!("moving temporary to {:?}", self.path_to_self());
        // rename(&temp, self.path_to_self())?;

        write_config::<Self>(&self.path_to_self(), &self)?;
        Ok(())
    }

    fn try_read(&mut self) -> anyhow::Result<()> {
        // let temp = get_temp_file(&self.path_to_self());
        // trace!("read previous node from {:?}", self.path_to_self());

        // // Move to temp.
        // rename(self.path_to_self(), &temp)?;

        // // Read to self.
        // *self = read_config::<Self>(&temp)?;

        // // Move temp back.
        // rename(temp, self.path_to_self())?;

        *self = read_config::<Self>(&self.path_to_self())?;
        trace!("read node: {:?}", &self);
        Ok(())
    }

    fn read_or_write_new(&mut self) -> anyhow::Result<()> {
        for i in 0..Self::SUBFOLDER_LEVEL {
            let mut parent = self.path_to_self();
            let sublvl = Self::SUBFOLDER_LEVEL - i;
            for _ in 0..sublvl {
                parent = parent.parent().unwrap().to_path_buf();
            }
            if parent.exists() {
                continue;
            }
            info!("Creating directory at parent {:?}", parent);
            create_dir(&parent)?;
        }

        if let Ok(_) = self.try_read() {
            // overwrites self.
            info!("found previous node: {:#?}", self);
        } else {
            info!("create new node at {:?}", self.path_to_self());
            self.try_write()?;
        }

        Ok(())
    }
}
