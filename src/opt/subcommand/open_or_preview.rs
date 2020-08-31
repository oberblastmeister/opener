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

        let path_str = self
            .path
            .to_str()
            .ok_or(anyhow!("Failed to convert path to string"))?;

        for possible_regex in possible_regexes {
            let narrowed: Vec<(Regex, String)> = possible_regex
                .into_par_iter()
                .map(|(regex, command)| (Regex::new(&regex), command))
                // TODO, needs to add warning if regex doesn't work
                // filter errors
                .filter_map(|(res, value)| res.ok().map(|regex| (regex, value)))
                // filter the pairs that match the path str
                .filter(|(regex, _command)| regex.is_match(path_str))
                .collect();
        }

        // wheather and of the commands specified in config file was run succesfully
        let mut command_successful = false;
        for possible in possibilites {
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
