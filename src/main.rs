mod config;
mod error;
mod mime_helpers;
mod opt;

use std::io::Write;
use std::process;

use anyhow::Result;
use env_logger::Builder;
use log::*;
use structopt::StructOpt;

use error::print_error;
use opt::Opt;
use opt::Runable;

/// Start the logger depending on the verbosity flag
fn start_logger(opt: &Opt) {
    Builder::from_default_env()
        .format(|buf, record| {
            let mut level_style = buf.default_level_style(record.level());
            level_style.set_bold(true);

            writeln!(
                buf,
                "{}: {}",
                level_style.value(record.level()),
                record.args()
            )
        })
        .filter_level(
            opt.verbose
                .log_level()
                .map(|l| l.to_level_filter())
                .unwrap_or(log::LevelFilter::Off),
        )
        .init();
}

/// Run the app
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
