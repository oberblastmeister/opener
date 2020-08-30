use std::convert::TryFrom;

use anyhow::Result;
use log::*;
use mime::Mime;

use super::parse_addtype;
use super::ExtMimePath;
use super::Runable;
use super::StructOpt;
use crate::config::EditConfig;

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
        let array = if self.preview {
            cfg.get_preview()?
        } else {
            cfg.get_open()?
        };

        let mut idx = 0;
        // for each table in open
        while let Some(table) = array.get_mut(idx) {
            debug!("Table: {:?}", table);

            // if there is already a key in the table
            if let Some(value) = table.entry(mime_str).as_value() {
                // check if the value is equal to the command added
                if value.as_str().expect("BUG: command should be a string") == &self.command {
                    // if the value is equal, the command for the associated mime type is already
                    // there, do nothing
                    info!("The mime {} already has a command", mime);
                    // do not create a table, we are adding something that is already there
                    return Ok(());
                } else {
                    // if there is already a key but the value is not the same, skip this table and go
                    // to the next one
                    idx += 1;
                    continue;
                }
            } else {
                // if there is not already the specified mime_str in the table, check just to make sure
                if let Some(value) = table.get(mime_str) {
                    debug!("Option<Item>: {:?}", value);
                    if let Some(_) = value.as_str() {
                        panic!("Hash to be Item: None")
                    }
                }
                // insert pair
                info!("inserting pair into table");
                table[mime_str] = toml_edit::value(self.command);
                // should_append_table is false because there is already a table and it has been
                // inserted in
                cfg.store()?;
                return Ok(());
            }
        }

        // if there are no more tables
        // there must have been no tables to insert in so create a new one
        info!("Appending new table");
        let mut table = toml_edit::Table::new();
        table[mime_str] = toml_edit::value(self.command);
        array.append(table);
        cfg.store()?;

        Ok(())
    }
}
