use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    #[structopt(subcommand)]
    pub cmd: SubCommand
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    Open {
        #[structopt(parse(from_os_str))]
        path: PathBuf
    },
    Add {
        extension_or_mime: String
    },
    Preview {
        #[structopt(parse(from_os_str))]
        path: PathBuf
    }
}
