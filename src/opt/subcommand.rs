mod set;
mod open_or_preview;
mod query;

use anyhow::Result;

use super::ext_mime_path::{parse_addtype, ExtMimePath};

use super::StructOpt;
use super::Runable;
use set::SetOptions;
use open_or_preview::OpenOptions;
use query::QueryOptions;

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Open or preview a file with the correct program
    Open(OpenOptions),

    /// Set the correct command for an extension, mime, or path
    Set(SetOptions),

    /// Query for mime types or extensions
    Query(QueryOptions),
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
