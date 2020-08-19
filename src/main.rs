mod config;
mod error;
mod opt;

use std::collections::HashMap;
use std::path::Path;
use std::process::{self, Command};

use anyhow::{bail, Context, Result};
use env_logger::Builder;
use log::*;
use structopt::StructOpt;

use config::Config;
use error::print_error;
use opt::{Opt, SubCommand};

fn run() -> Result<()> {
    let opt = Opt::from_args();

    start_logger(&opt);

    trace!("{:?}", &opt);

    let mut cfg = Config::load()?;
    debug!("{:?}", &cfg);

    let mime_types = cfg.get_mime_types();

    match opt.cmd {
        SubCommand::Open { file } => {
            let correct: HashMap<_, _> = mime_types
                .iter()
                .filter(|(mime, _command)| {
                    let possible_extensions = mime_guess::get_mime_extensions(mime);
                    possible_extensions
                        .expect("Failed to get extensions")
                        .iter()
                        .any(|e| {
                            *e == file
                                .extension()
                                .expect("Could not get file extension")
                                .to_str()
                                .expect("could not convert to string")
                        })
                })
                .collect();

            debug!("match and command is : {:?}", correct);

            correct.iter().take(1).for_each(|(_mime, command)| {
                run_command(command, &file).expect("Failed to run command");
            });
        }
    }

    Ok(())
}

fn run_command(cmd: &str, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let mut child = Command::new(cmd).arg(path).spawn()?;

    let ecode = child.wait()?;

    if !ecode.success() {
        bail!("child command failed")
    }

    Ok(())
}

fn start_logger(opt: &Opt) {
    Builder::from_default_env()
        .filter_level(
            opt.verbose
                .log_level()
                .map(|l| l.to_level_filter())
                .unwrap_or(log::LevelFilter::Off),
        )
        .init();
}

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            print_error(format!("{:?}", e));
            process::exit(1);
        }
    }
}
