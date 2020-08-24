mod add;
mod open_or_preview;
mod query;

use std::path::PathBuf;

use anyhow::Result;

use super::addtype::{parse_addtype, AddType};

use super::StructOpt;
use super::Runable;
use add::Add;
use open_or_preview::{Preview, Open};
use query::Query;

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    Open(Open),
    Add(Add),
    Preview(Preview),
    Query(Query),
}

impl Runable for SubCommand {
    fn run(self) -> Result<()> {
        match self {
            SubCommand::Open(open) => open.run(),
            SubCommand::Add(add) => add.run(),
            SubCommand::Preview(preview) => preview.run(),
            SubCommand::Query(query) => query.run(),
        }
    }
}
