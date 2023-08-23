use anyhow::{ensure, Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{self, json};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub fn read_config<T: DeserializeOwned>(path: &Path) -> Result<T> {
    ensure!(path.exists(), format!("Path to config file does not exist: {:?}", path));
    let mut file = File::options().read(true).open(path)?;
    let mut serialized = String::new();
    let _len = file.read_to_string(&mut serialized)?;
    let config: T = serde_json::from_str(&serialized)
        .context("error parsing configuration file.")
        .context(format!("file: {:?}", path))?;
    Ok(config)
}

pub fn write_config<T: Serialize>(path: &Path, config: &T) -> Result<()> {
    let serialized = json!(config);
    let mut file = File::create(path)?;
    let string = format!("{}", serde_json::to_string_pretty(&serialized).unwrap());
    file.write_all(string.as_bytes())?;
    Ok(())
}

pub fn write_default_config<T: Serialize + Default>(path: &Path) -> Result<()> {
    let default = T::default();
    write_config::<T>(path, &default)?;
    Ok(())
}
