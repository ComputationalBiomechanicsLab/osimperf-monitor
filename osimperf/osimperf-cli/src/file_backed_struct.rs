use log::{trace, warn};
use std::path::PathBuf;
use std::{fmt::Debug, fs::create_dir};

use serde::{de::DeserializeOwned, Serialize};

use crate::common::{read_json, write_json};
use crate::Ctxt;

pub trait FileBackedStruct: Serialize + DeserializeOwned + Debug {
    fn path_to_self(&self, context: &Ctxt) -> PathBuf;

    fn try_write(&self, context: &Ctxt) -> anyhow::Result<()> {
        write_json::<Self>(&self.path_to_self(context), &self)?;
        Ok(())
    }

    fn try_read(&mut self, context: &Ctxt) -> anyhow::Result<()> {
        *self = read_json::<Self>(&self.path_to_self(context))?;
        trace!("read node: {:?}", &self);
        Ok(())
    }

    fn read_or_write_new(&mut self, context: &Ctxt) -> anyhow::Result<()> {
        // Check if file already exists, and overwrite self.
        if let Ok(_) = self.try_read(context) {
            trace!("found previous node: {:#?}", self);
            return Ok(());
        }

        // Create new file.
        let parent = self.path_to_self(context).parent().unwrap().to_path_buf();
        if !parent.exists() {
            trace!("Creating directory at parent {:?}", parent);
            create_dir(parent)?;
        }

        trace!("create new file at {:?}", self.path_to_self(context));
        self.try_write(context)?;

        Ok(())
    }
}
