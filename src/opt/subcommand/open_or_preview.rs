use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use log::*;
use rayon::prelude::*;
use regex::Regex;
use subprocess::Exec;

use super::Runable;
use super::StructOpt;
use crate::config::Narrowable;
use crate::config::OpenConfig;
use crate::config::PossibleRegexes;
use crate::mime_helpers::determine_mime;

/// Options to use for subcommand open
#[derive(StructOpt, Debug)]
pub struct OpenOptions {
    #[structopt(parse(from_os_str))]
    /// the file to open
    path: PathBuf,

    /// enter interactive mode
    #[structopt(short, long)]
    interactive: bool,

    /// preview the file
    #[structopt(short, long)]
    preview: bool,
}

impl Runable for OpenOptions {
    fn run(self) -> Result<()> {
        let open_config = OpenConfig::load()?;

        let (possibilites, possible_regexes) = if self.preview {
            let OpenConfig {
                preview,
                preview_regex,
                ..
            } = open_config;
            (preview, preview_regex)
        } else {
            let OpenConfig {
                open, open_regex, ..
            } = open_config;
            (open, open_regex)
        };

        let mime = determine_mime(&self.path)?;
        debug!("Guess: {:?}", mime);

        let path_string = self
            .path
            .to_str()
            .ok_or(anyhow!("Failed to convert path to string"))?
            .to_owned();

        if run_narrowable(possible_regexes, &path_string, &self.path).is_err() {
            info!("Running regex commands failed, trying to run mime commands");
            if run_narrowable(possibilites, &mime, &self.path).is_err() {
                info!("Running mime commands failed, trying to use xdg_open");
                xdg_open(&self.path)?;
            }
        }

        Ok(())
    }
}

fn run_narrowable<T: Narrowable>(
    possible_regexes: Vec<T>,
    compare: &T::Compare,
    path: impl AsRef<Path>,
) -> Result<()> {
    for possible_regex in possible_regexes {
        let command = possible_regex.narrow(compare)?;

        if command.is_none() {
            bail!("No command to run");
        }

        if let Some(cmd) = command {
            match run_shell_command(&cmd, &path) {
                Ok(_) => return Ok(()),
                Err(e) => Err(e)?,
            }
        }
    }
    Ok(())
}

/// Open something using the default program on the system
fn xdg_open(path: impl AsRef<Path>) -> Result<()> {
    open::that(path.as_ref().as_os_str()).context("Failed to use xdg-open")?;
    Ok(())
}

/// Run shell command and return Ok(()) if successful
fn run_shell_command(cmd: &str, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let exit_status = Exec::shell(cmd).join()?;

    if !exit_status.success() {
        bail!(
            "The child command {} with path {} failed",
            cmd,
            path.display()
        );
    }

    Ok(())
}

fn rofi_open(commands: &Vec<String>, path: impl AsRef<Path>) -> Result<()> {
    match rofi::Rofi::new(commands).run() {
        Ok(choice) => run_shell_command(&choice, path)?,
        Err(rofi::Error::Interrupted) => info!("Rofi was interrupted"),
        Err(e) => Err(e)?,
    }
    Ok(())
}
