use std::path::PathBuf;
use std::process::Command;
use std::path::Path;
use std::io::{stdout, Write};
use anyhow::{anyhow, bail, Context, Result};
use log::*;

use crate::config::EditConfig;
use crate::config::OpenConfig;
use crate::mime_helpers::determine_mime;
use super::addtype::{parse_addtype, AddType};

use super::StructOpt;

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    Open {
        #[structopt(parse(from_os_str))]
        path: PathBuf,

        #[structopt(short, long)]
        pick: bool,
    },
    Add {
        #[structopt(parse(try_from_str = parse_addtype))]
        addtype: AddType,

        command: String,
    },
    Preview {
        #[structopt(parse(from_os_str))]
        path: PathBuf,

        #[structopt(short, long)]
        pick: bool,
    },
    Query {
        #[structopt(parse(try_from_str = parse_addtype))]
        addtype: AddType,
    },
}

impl SubCommand {
    pub fn run(self) -> Result<()> {
        match self {
            SubCommand::Open { ref path, pick } | SubCommand::Preview { ref path, pick } => {
                run_open_or_preview(path, &self, pick)?;
            }
            SubCommand::Add {
                addtype,
                command,
            } => {
                run_add(addtype, command)?;
            }
            SubCommand::Query {
                addtype,
            } => run_query(addtype)?,
            _ => (),
        }

        Ok(())
    }
}

fn run_open_or_preview(path: impl AsRef<Path>, subcmd: &SubCommand, pick: bool) -> Result<()> {
    // make sure subcommand is correct
    let possible = match subcmd {
        SubCommand::Open { .. } => {
            let OpenConfig { open, preview: _ } = OpenConfig::load()?;
            open
        }
        SubCommand::Preview { .. } => {
            let OpenConfig { open: _, preview } = OpenConfig::load()?;
            preview
        }
        _ => panic!("run_open_or_preview must used with the Open or Preview SubCommand"),
    };

    // The mime type of the path that we are opening
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

fn xdg_open(path: impl AsRef<Path>) -> Result<()> {
    open::that(path.as_ref().as_os_str()).context("Failed to use xdg-open")?;
    Ok(())
}

fn run_query(addtype: AddType) -> Result<()> {
    match addtype {
        AddType::Mime(mime) => {
            let extensions = mime_guess::get_mime_extensions(&mime)
                .ok_or(anyhow!("No mime types found for given extension"))?;
            let stdout = stdout();
            let mut stdout = stdout.lock();
            for extension in extensions {
                write!(stdout, ".{} ", extension).unwrap();
            }
            write!(stdout, "\n").unwrap();
            stdout.flush().unwrap();
        }
        AddType::Path(path) => {
            let mime_string = determine_mime(path)?.to_string();
            println!("{}", mime_string);
        }
        AddType::Extension(ext) => {
            let mime_string = mime_guess::from_ext(&ext)
                .first()
                .ok_or(anyhow!("Could not get mime type from extension"))?
                .to_string();
            println!("{}", mime_string);
        }
    }
    Ok(())
}

fn run_add(addtype: AddType, command: String) -> Result<()> {
    let mut cfg = EditConfig::load()?;
    let mime = addtype.convert_to_mime()?;
    let mime_str = mime.essence_str();
    debug!("Run add is using this config: {}", cfg.to_string());

    let mut should_append_table = true;
    let mut idx = 0;
    // for each table in open
    while let Some(table) = cfg.get_open()?.get_mut(idx) {
        debug!("Table: {:?}", table);

        // if there is already a key in the table
        if let Some(value) = table.entry(mime_str).as_value() {
            // check if the value is equal to the command added
            if value.as_str().expect("BUG: command should be a string") == &command {
                // if the value is equal, the command for the associated mime type is already
                // there, do nothing
                info!("The mime {} already has a command", mime);
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
            if let Some(value) = table.get(mime_str) {
                debug!("Option<Item>: {:?}", value);
                if let Some(_) = value.as_str() {
                    panic!("Hash to be Item: None")
                }
            }
            // insert pair
            table[mime_str] = toml_edit::value(command.clone());
            // should_append_table is false because there is already a table and it has been
            // inserted in
            should_append_table = false;
            break;
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
