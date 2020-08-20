mod config;
mod error;
mod mime_helpers;
mod opt;

use std::path::Path;
use std::process::{self, Command};

use anyhow::{anyhow, bail, Context, Result};
use env_logger::Builder;
use log::*;
use structopt::StructOpt;

use config::Config;
use error::print_error;
use mime_helpers::{filter_matches, get_guess, remove_star_mimes};
use opt::{Opt, SubCommand};

fn run_command(cmd: &str, path: impl AsRef<Path>) -> Result<()> {
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

fn run() -> Result<()> {
    let opt = Opt::from_args();

    start_logger(&opt);

    trace!("{:?}", &opt);

    let mut cfg = Config::load()?;
    debug!("{:?}", &cfg);

    let mimes_and_commands = cfg.get_mime_types();

    match opt.cmd {
        SubCommand::Open { path } => {
            let guess = get_guess(&path)?;
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
                run_command(&command, &path)?;
            }
        }
        _ => (),
    }

    Ok(())
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
