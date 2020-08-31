use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use log::*;
use rayon::prelude::*;
use regex::Regex;
use subprocess::Exec;

use super::Runable;
use super::StructOpt;
use crate::config::OpenConfig;
use crate::config::PossibleCommands;
use crate::config::RegexCommands;
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

        // use open or preview commands
        let (possibilites, possible_regexes) = if self.preview {
            open_config.get_preview()
        } else {
            open_config.get_open()
        };

        let path_string = self
            .path
            .to_str()
            .ok_or(anyhow!("Failed to convert path to string"))?
            .to_owned();

        if run_correct_command_with_fallbacks(possible_regexes, &path_string, &self.path).is_err() {
            info!("Running regex commands failed, trying to run mime commands");

            // determine the mime of the path to compare with the mime commands
            let mime = determine_mime(&self.path)?;
            debug!("Guess: {:?}", mime);

            if run_correct_command_with_fallbacks(possibilites, &mime, &self.path).is_err() {
                info!("Running mime commands failed, trying to use xdg_open");
                xdg_open(&self.path)?;
            }
        }

        Ok(())
    }
}

fn run_correct_command_with_fallbacks<T: PossibleCommands>(
    possible_commands_fallbacks: Vec<T>,
    compare: &T::Compare,
    path: impl AsRef<Path>,
) -> Result<()> {
    for fallback in possible_commands_fallbacks {
        let command = fallback.find_correct_command(compare)?;

        if let Some(cmd) = command {
            match run_shell_command(&cmd, &path) {
                Ok(_) => return Ok(()),
                Err(e) => Err(e)?,
            }
        } else {
            // if there is no command specified, go to the next fall back
            continue;
        }
    }

    // if the code has gotten here, the code has checked all fallbacks and no command was found for
    // the compare. Return an error that no commands were found
    bail!("No command found for compare");
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
