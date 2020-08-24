use std::path::{PathBuf, Path};
use std::process::Command;

use anyhow::{Context, Result, bail};
use log::*;

use crate::config::{Possible, OpenConfig};
use crate::mime_helpers::determine_mime;
use super::SubCommand;
use super::StructOpt;
use super::Runable;

#[derive(StructOpt, Debug)]
pub struct Open {
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    #[structopt(short, long)]
    pick: bool,
}

impl OpenOrPreview for Open {
    fn get_possible() -> Result<Vec<Possible>> {
        let OpenConfig { open, preview: _ } = OpenConfig::load()?;
        Ok(open)
    }
}

impl Runable for Open {
    fn run(&self) -> Result<()> {
        Self::run_open_or_preview(&self.path, self.pick)
    }
}

#[derive(StructOpt, Debug)]
pub struct Preview {
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    #[structopt(short, long)]
    pick: bool,
}

impl OpenOrPreview for Preview {
    fn get_possible() -> Result<Vec<Possible>> {
        let OpenConfig { open: _, preview } = OpenConfig::load()?;
        Ok(preview)
    }
}

impl Runable for Preview {
    fn run(&self) -> Result<()> {
        Self::run_open_or_preview(&self.path, self.pick)
    }
}

trait OpenOrPreview {
    fn get_possible() -> Result<Vec<Possible>>;
    fn run_open_or_preview(path: impl AsRef<Path>, pick: bool) -> Result<()> {
        let possible = Self::get_possible()?;

        let mime = determine_mime(&path)?;
        debug!("Guess: {:?}", mime);

        // wheather and of the commands specified in config file was run succesfully
        let mut command_successful = false;
        for possible in possible {
            // finds the correct command according to the mime
            let command = possible.narrow(&mime);
            if run_shell_command(&command, &path).is_ok() {
                command_successful = true;
                break;
            }
        }

        // if none of the commands were run succesfully or there were no commands specified, use
        // xdg-open instead
        if !command_successful {
            info!("Using xdg-open instead");
            xdg_open(&path)?;
        }

        Ok(())
    }
}

fn xdg_open(path: impl AsRef<Path>) -> Result<()> {
    open::that(path.as_ref().as_os_str()).context("Failed to use xdg-open")?;
    Ok(())
}

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
