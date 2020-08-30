use std::fs::File;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{anyhow, bail, Context, Result};
use directories::ProjectDirs;

const DEFAULT_CONFIG: &'static [u8] = include_bytes!("default_config.toml");
const EXTENSION: &'static str = "toml";
const NAME: &'static str = "opener";
const QUALIFIER: &'static str = "rs";
const ORGANIZATION: &'static str = "";

/// Loads config into string
pub fn load_to_string() -> Result<String> {
    load_to_string_or_default(get_config_path()?)
}

/// Stores string into config
pub fn store_string(s: &str) -> Result<()> {
    let mut f = open_file(get_config_path()?)?;
    f.write_all(s.as_bytes())?;
    Ok(())
}

/// Opens file with the correct options
fn open_file(path: impl AsRef<Path>) -> Result<File> {
    Ok(OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .context("Failed to open file.")?)
}

/// Stores the default config in the specified path.
fn store_default(path: impl AsRef<Path>) -> Result<&'static str> {
    let mut f = open_file(path)?;

    f.write_all(DEFAULT_CONFIG)
        .context("Failed to write to default config")?;

    Ok(str::from_utf8(DEFAULT_CONFIG).expect("BUG: the default config was not utf8"))
}

/// Loads a file to string or creates a default if it does not exist, then returns the default
/// string
fn load_to_string_or_default(path: impl AsRef<Path>) -> Result<String> {
    match File::open(&path) {
        Ok(mut file) => Ok(file.get_string()?),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(store_default(&path)?.to_string()),
        Err(_e) => bail!("General load error"),
    }
}

/// Gets the path of the config file
fn get_config_path() -> Result<PathBuf> {
    let project =
        ProjectDirs::from(QUALIFIER, ORGANIZATION, NAME).ok_or(anyhow!("An error occured"))?;

    let config_dir_str = get_config_dir_str(&project)?;

    let path: PathBuf = [config_dir_str, &format!("{}.{}", NAME, EXTENSION)]
        .iter()
        .collect();

    Ok(path)
}

/// Get the string that represents the config directory for the system
fn get_config_dir_str(project: &ProjectDirs) -> Result<&str> {
    project
        .config_dir()
        .to_str()
        .ok_or(anyhow!("Failed to get config dir str"))
}

/// Something that is able to read a string from
trait CheckedStringRead {
    fn get_string(&mut self) -> Result<String, io::Error>;
}

impl CheckedStringRead for File {
    fn get_string(&mut self) -> Result<String, io::Error> {
        let mut s = String::new();
        self.read_to_string(&mut s)?;
        Ok(s)
    }
}
