mod config;
mod error;
mod mime_helpers;
mod opt;

use std::borrow::Cow;
use std::path::Path;
use std::process::{self, Command};

use anyhow::{anyhow, bail, Context, Result};
use env_logger::Builder;
use log::*;
use structopt::StructOpt;

use config::Config;
use error::print_error;
use mime_helpers::{filter_matches, get_mime_from_path, remove_star_mimes};
use opt::{Opt, SubCommand};

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

    let cfg = Config::load()?;
    debug!("The config was loaded: {:?}", &cfg);

    opt.subcmd.run(cfg)?;

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
