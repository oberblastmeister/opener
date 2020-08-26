mod set;
mod open_or_preview;
mod query;

use anyhow::Result;

use super::ext_mime_path::{parse_addtype, ExtMimePath};

use super::StructOpt;
use super::Runable;
use set::Set;
use open_or_preview::Open;
use query::Query;

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    Open(Open),
    Set(Set),
    Query(Query),
}

impl Runable for SubCommand {
    fn run(self) -> Result<()> {
        match self {
            SubCommand::Open(open) => open.run(),
            SubCommand::Set(add) => add.run(),
            SubCommand::Query(query) => query.run(),
        }
    }
}
