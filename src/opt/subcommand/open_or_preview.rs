use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use log::*;

use super::Runable;
use super::StructOpt;
use crate::config::OpenConfig;
use crate::mime_helpers::determine_mime;

/// Open or preview a file with the correct program
#[derive(StructOpt, Debug)]
pub struct Open {
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

impl Runable for Open {
    fn run(self) -> Result<()> {
        let possible = if self.preview {
            let OpenConfig { open: _, preview } = OpenConfig::load()?;
            preview
        } else {
            let OpenConfig { open, preview: _ } = OpenConfig::load()?;
            open
        };

        let mime = determine_mime(&self.path)?;
        debug!("Guess: {:?}", mime);

        // wheather and of the commands specified in config file was run succesfully
        let mut command_successful = false;
        for possible in possible {
            // finds the correct command according to the mime
            let command = possible.narrow(&mime);
            if run_shell_command(&command, &self.path).is_ok() {
                command_successful = true;
                break;
            }
        }

        // if none of the commands were run succesfully or there were no commands specified, use
        // xdg-open instead
        if !command_successful {
            info!("Using xdg-open instead");
            xdg_open(&self.path)?;
        }

        Ok(())
    }
}

/// Open something using the default program on the system
fn xdg_open(path: impl AsRef<Path>) -> Result<()> {
    open::that(path.as_ref().as_os_str()).context("Failed to use xdg-open")?;
    Ok(())
}

/// Run shell command and return Ok(()) if successful
fn run_shell_command(cmd: &str, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let mut child = Command::new(cmd).arg(path).spawn()?;

    let ecode = child
        .wait()
        .context(format!("Failed to wait on child command {}", cmd))?;

    if !ecode.success() {
        bail!(
            "The child command {} with path {} failed",
            cmd,
            path.display()
        );
    }

    Ok(())
}
