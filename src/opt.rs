mod subcommand;
mod ext_mime_path;

use structopt::StructOpt;
use anyhow::Result;

use subcommand::SubCommand;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}

pub trait Runable {
    fn run(self) -> Result<()>;
}

impl Runable for Opt {
    fn run(self) -> Result<()> {
        self.subcmd.run()
    }
}
