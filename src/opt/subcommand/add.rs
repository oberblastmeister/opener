use anyhow::Result;
use log::*;

use super::parse_addtype;
use super::AddType;
use super::Runable;
use super::StructOpt;
use crate::config::EditConfig;

#[derive(StructOpt, Debug)]
pub struct Add {
    #[structopt(parse(try_from_str = parse_addtype))]
    addtype: AddType,

    command: String,
}

impl Runable for Add {
    fn run(&self) -> Result<()> {
        let mut cfg = EditConfig::load()?;
        let mime = self.addtype.convert_to_mime()?;
        let mime_str = mime.essence_str();
        debug!("Run add is using this config: {}", cfg.to_string());

        let mut should_append_table = true;
        let mut idx = 0;
        // for each table in open
        while let Some(table) = cfg.get_open()?.get_mut(idx) {
            debug!("Table: {:?}", table);

            // if there is already a key in the table
            if let Some(value) = table.entry(mime_str).as_value() {
                // check if the value is equal to the command added
                if value.as_str().expect("BUG: command should be a string") == &self.command {
                    // if the value is equal, the command for the associated mime type is already
                    // there, do nothing
                    info!("The mime {} already has a command", mime);
                    // do not create a table, we are adding something that is already there
                    should_append_table = false;
                    break;
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
                table[mime_str] = toml_edit::value(self.command.clone());
                // should_append_table is false because there is already a table and it has been
                // inserted in
                should_append_table = false;
                break;
            }
        }

        // if there are no more tables
        if should_append_table {
            // there must have been no tables to insert in so create a new one
            let mut table = toml_edit::Table::new();
            table[mime_str] = toml_edit::value(self.command.clone());
            cfg.get_open()?.append(table);
        }
        cfg.store()?;

        Ok(())
    }
}
