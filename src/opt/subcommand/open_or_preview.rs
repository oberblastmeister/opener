use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use log::*;
use subprocess::Exec;

use super::Runable;
use super::StructOpt;
use crate::config::OpenConfig;
use crate::config::PossibleCommands;
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
            info!("Running preview");
            open_config.get_preview()
        } else {
            info!("Running open");
            open_config.get_open()
        };

        let path_str = self
            .path
            .to_str()
            .ok_or(anyhow!("Failed to convert path to string"))?;

        if run_correct_command_with_fallbacks(possible_regexes, path_str, &self.path).is_err() {
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
            match run_shell_command_with_path(&cmd, &path) {
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
fn run_shell_command_with_path(cmd: &str, path: impl AsRef<Path>) -> Result<()> {
    debug!("part command is `{}`", cmd);

    //convert path to string
    let path = path
        .as_ref()
        .to_str()
        .ok_or(anyhow!("Failed to convert path to string"))?;
    // add path to end of command with space if there is no $f
    let cmd = if cmd.find("$f").is_none() {
        format!("{} {}", cmd, path)
    } else {
        // else replace $f with the path
        cmd.replace("$f", path)
    };
    debug!("full command is `{}`", &cmd);

    let exit_status = Exec::shell(&cmd)
        .detached()
        .popen()?
        .wait_timeout(Duration::from_secs(5))?
        .ok_or(anyhow!("No exit status, command took too long"))?;

    if !exit_status.success() {
        bail!("The child command {}", &cmd,);
    }

    Ok(())
}
