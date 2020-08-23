use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::{self, Command};
use structopt::StructOpt;

use anyhow::{anyhow, bail, Context, Result};
use log::*;
use mime::Mime;

use crate::config::EditConfig;
use crate::config::OpenConfig;
use crate::config::Possible;
use crate::mime_helpers::{determine_mime, filter_by_mimes, remove_star_mimes};

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    Open {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    Add {
        extension_mime_path: String,
        command: String,
    },
    Preview {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

impl SubCommand {
    pub fn run(self) -> Result<()> {
        match self {
            SubCommand::Open { path } => {
                run_open(path, mimes_and_commands)?;
            }
            SubCommand::Add {
                extension_mime_path,
                command,
            } => {
                run_add(extension_mime_path, command, cfg)?;
            }
            _ => (),
        }

        Ok(())
    }
}

fn run_open(path: impl AsRef<Path>) -> Result<()> {
    // load open open config, we don't need preview because we are only running open
    let OpenConfig { open, preview: _ } = OpenConfig::load()?;

    // The mime type of the path that we are opening
    let mime = determine_mime(path)?;
    debug!("Guess: {:?}", mime);

    // wheather and of the commands specified in config file was run succesfully
    let mut command_successful = false;
    for possible in open {
        // finds the correct command according to the mime
        let command = possible.narrow(mime);
        if run_shell_command(&command, path).is_ok() {
            command_successful = true;
            break;
        }
    }

    // if none of the commands were run succesfully or there were no commands specified, use
    // xdg-open instead
    if !command_successful {
        info!("Using xdg-open instead");
        xdg_open();
    }

    Ok(())
}

fn xdg_open() {
    todo!()
}

fn run_add(extension_mime_path: String, command: String, mut cfg: OpenConfig) -> Result<()> {
    let cfg = EditConfig::load()?;
    let mime = AddType::determine(&extension_mime_path)?.convert_to_mime()?;
    let mime_str = mime.essence_str();
    let mime_string = mime_str.to_string();
    debug!("Run add is using this config: {}", cfg.to_string());

    let mut should_append_table = true;
    let idx = 0;
    // for each table in open
    while let Some(table) = cfg.get_open()?.get_mut(idx) {
        debug!("Table: {:?}", table);

        // if there is already a key in the table
        if let Some(value) = table.entry(mime_str).as_value() {
            // check if the value is equal to the command added
            if value.as_str().expect("BUG: command should be a string") == &command {
                // if the value is equal, the command for the associated mime type is already
                // there, do nothing
                info!("{} already has a command", &extension_mime_path);
                // do not create a table, we are adding something that is already there
                should_append_table = false;
                break;
            } else {
                // if there is already a key but the value is not the same, skip this table and go
                // to the next one
                idx += 1;
                continue;
            }
        } else {
            // if there is not already the specified mime_str in the table, check just to make sure
            if table.get(mime_str).is_some() {
                panic!("BUG: there should not be that key in table because the table should have been skipped above");
            }
            // insert pair
            let mut inserted = table.entry(mime_str);
            table[mime_str] = toml_edit::value(command);
            // should_append_table is false because there is already a table and it has been
            // inserted in
            should_append_table = false;
        }
    }

    // if there are no more tables
    if should_append_table {
        // there must have been no tables to insert in so create a new one
        let mut table = toml_edit::Table::new();
        table[mime_str] = toml_edit::value(command);
        cfg.get_open()?.append(table);
    }
    cfg.store()?;

    Ok(())
}

fn run_preview() {
    todo!()
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

#[derive(Debug)]
enum AddType<'a> {
    Extension(&'a str),
    Mime(mime::Mime),
    Path(&'a Path),
}

impl<'a> AddType<'a> {
    fn determine(candidate: &'a str) -> Result<Self> {
        if candidate.starts_with('.') {
            return Ok(AddType::Extension(candidate));
        }

        if let Ok(mime) = candidate.parse::<mime::Mime>() {
            return Ok(AddType::Mime(mime));
        }

        let path = Path::new(candidate);
        if path.exists() {
            return Ok(AddType::Path(path));
        }

        bail!("The supplied string is not an extension, mime, or path.");
    }

    fn convert_to_mime(&self) -> Result<mime::Mime> {
        match self {
            AddType::Extension(ext) => Ok(mime_guess::from_ext(
                &ext.chars().skip(1).collect::<String>(),
            )
            .first()
            .ok_or(anyhow!("No mime type found from extension {}", ext))?),
            AddType::Mime(mime) => Ok(mime.clone()),
            AddType::Path(path) => determine_mime(path),
        }
    }
}
