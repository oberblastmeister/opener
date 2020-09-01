use std::convert::TryFrom;

use anyhow::Result;
use log::*;
use mime::Mime;

use super::parse_addtype;
use super::ExtMimePath;
use super::Runable;
use super::StructOpt;
use crate::config::{StreamingIteratorMut, EditConfig};

/// Options to use for subcommand set
#[derive(StructOpt, Debug)]
pub struct SetOptions {
    /// can be a file extension, mime, or path
    #[structopt(parse(try_from_str = parse_addtype))]
    ext_mime_path: ExtMimePath,

    /// the command to add for the extension, path, or mime type
    command: String,

    /// weather to set preview instead of setting the open command
    #[structopt(long, short)]
    preview: bool,
}

impl Runable for SetOptions {
    fn run(self) -> Result<()> {
        let mut cfg = EditConfig::load()?;
        debug!("Run add is using this config:\n{}", cfg.to_string());

        let mime = Mime::try_from(self.ext_mime_path)?;
        let mime_str = mime.essence_str();
        let mut array = if self.preview {
            cfg.get_preview_iter_mut()?
        } else {
            cfg.get_open_iter_mut()?
        };

        // for each table in open
        while let Some(table) = array.next() {
            debug!("Table: {:?}", table);

            // if there is already a key in the table
            if let Some(value) = table.entry(mime_str).as_value() {
                // check if the value is equal to the command that we are going to add
                if value.as_str().expect("BUG: command should be a string") == &self.command {

                    // if the value is equal, the command for the associated mime type is already
                    // there, do nothing
                    info!("The mime {} already has a command", mime);

                    // do not create a table, we are adding something that is already there
                    return Ok(());
                } else {
                    // if there is already a key but the value is not the same, skip this table and go
                    // to the next one
                    continue;
                }
            } else {
                // if there is not already a key in the table
                info!("inserting pair into table");
                table[mime_str] = toml_edit::value(self.command);
                cfg.store()?;
                return Ok(());
            }
        }

        // if there are no more tables
        // there must have been no tables to insert in so create a new one
        info!("Appending new table");
        array.append_single_entry_table(mime_str, self.command);
        cfg.store()?;

        Ok(())
    }
}
