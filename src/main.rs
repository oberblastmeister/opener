mod config;
mod error;
mod mime_helpers;
mod opt;

use std::process;

use anyhow::Result;
use env_logger::Builder;
use log::*;
use structopt::StructOpt;

use error::print_error;
use opt::Opt;
use opt::Runable;

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

    opt.run()?;

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
