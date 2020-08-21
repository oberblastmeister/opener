use crate::config::Config;
use crate::mime_helpers::{filter_matches, get_mime_from_path, remove_star_mimes};
use anyhow::{anyhow, bail, Context, Result};
use log::*;
use mime::Mime;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::{self, Command};
use structopt::StructOpt;

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
    pub fn run(self, mut cfg: Config) -> Result<()> {
        let mimes_and_commands = cfg.get_mime_types();

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

fn run_open(path: impl AsRef<Path>, mimes_and_commands: BTreeMap<Mime, &str>) -> Result<()> {
    let guess = get_mime_from_path(&path)?;
    debug!("Guess: {:?}", guess);

    let mut matches = filter_matches(guess, mimes_and_commands);
    debug!("Matches before narrowing down to 1: {:?}", matches);

    if matches.len() > 1 {
        matches = remove_star_mimes(matches);
    }

    debug!("Matches after narrowing down to 1: {:?}", matches);

    if matches.len() > 1 {
        panic!("BUG: matches length should not be greater than 1. Toml file should have non-repeating strings. After removing stars there can only be one match for each mime type.")
    }

    for (_mime, command) in matches {
        run_shell_command(&command, &path)?;
    }

    Ok(())
}

fn run_add(extension_mime_path: String, command: String, mut cfg: Config) -> Result<()> {
    let mime = AddType::determine(&extension_mime_path)?.convert_to_mime()?;
    let mime_str = mime.essence_str();
    let mime_string = mime_str.to_string();
    debug!("Run add is using this config: {:?}", cfg);

    let mut should_append = true;
    for table in cfg.open.iter_mut() {
        debug!("Fall back: {:?}", table);
        if let Some(value) = table.get(mime_str) {
            if value == &command {
                info!("{} already has a command", &extension_mime_path);
                should_append = false;
                break;
            } else {
                continue;
            }
        }
        if table.insert(mime_string.clone(), command.clone()).is_some() {
            panic!("BUG: there should not be that key in table because the table should have been skipped above")
        }
        should_append = false;
        break;
    }

    // if nothing was inserted
    if should_append {
        // there must have been no tables to insert in so create a new one
        let mut map = BTreeMap::new();
        map.insert(mime_string, command);
        cfg.open.push(map);
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

    let ecode = child.wait()?;

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
            AddType::Path(path) => get_mime_from_path(path),
        }
    }
}
