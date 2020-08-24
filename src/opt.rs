mod subcommand;
mod addtype;

use structopt::StructOpt;

use subcommand::SubCommand;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}
