use anyhow::Context;
use std::fmt::Debug;
use log::trace;
use std::{
    fs::rename,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{read_config, write_config};

fn get_temp_file(path: &Path) -> PathBuf {
    path.parent().unwrap().join("temp-conf")
}

pub trait NodeFile: Serialize + DeserializeOwned + Debug {
    fn path_to_self(&self) -> PathBuf;

    fn try_write(&self) -> anyhow::Result<()> {
        let temp = get_temp_file(&self.path_to_self());
        // write to temp.
        trace!("writin to remporary file {:?}", temp);
        write_config(&temp, &self).context("failed to write node file to temporary")?;
        // Move to self.
        trace!("moving temporary to {:?}", self.path_to_self());
        rename(&temp, self.path_to_self())?;
        Ok(())
    }

    fn try_read(&mut self) -> anyhow::Result<()> {
        let temp = get_temp_file(&self.path_to_self());
        trace!("read previous node from {:?}", self.path_to_self());

        // Move to temp.
        rename(self.path_to_self(), &temp)?;

        // Read to self.
        *self = read_config::<Self>(&temp)?;

        // Move temp back.
        rename(temp, self.path_to_self())?;

        trace!("read node: {:?}", &self);
        Ok(())
    }
}
