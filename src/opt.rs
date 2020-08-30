mod subcommand;
mod ext_mime_path;

use std::path::PathBuf;

use structopt::StructOpt;
use anyhow::Result;

use subcommand::SubCommand;

/// A tool to unify and make easier xdg-mime and xdg-open. Specify commands for mime types in it's
/// toml config file which allows comments. Falls back to xdg-open if nothing is specified
#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    // #[structopt(parse(from_os_str))]
    // pub path: Option<PathBuf>,

    #[structopt(subcommand)]
    // pub subcmd: Option<SubCommand>,
    pub subcmd: SubCommand,
}

pub trait Runable {
    fn run(self) -> Result<()>;
}

impl Runable for Opt {
    fn run(self) -> Result<()> {
        // match self.subcmd {
        //     Some(subcmd) => subcmd.run(),
        //     None => Ok(())
        // }
        self.subcmd.run()
    }
}
