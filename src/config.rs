use std::collections::HashMap;

use anyhow::{Context, Result};
use log::*;
use toml_edit::Document;
use mime::Mime;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OpenConfig {
    pub open: Vec<HashMap<String, String>>,
    pub preview: Vec<HashMap<String, String>>,
}

impl OpenConfig {
    pub fn create_default()  -> Result<()> {
        confy::store("opener", Self::default())?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let cfg_name = "opener";

        confy::load("opener").context(format!(
            "Failed to load configuration file {}.toml",
            cfg_name
        ))
    }

    pub fn get_mime_types(&mut self) -> HashMap<Mime, &str> {
        let mimes_and_commands: HashMap<mime::Mime, &str> = self
            .open
            .first_mut()
            .unwrap()
            .iter()
            .map(|(mime_str, command)| {
                let mime: Result<Mime> = mime_str.parse().context(format!(
                    "Failed to parse mime type from string {}",
                    mime_str
                ));
                mime.map(|m| (m, &**command))
            })
            // log errors
            .inspect(|r| {
                if let Err(e) = r {
                    warn!("{:?}", e);
                }
            })
            // then ignore errors
            .filter_map(|e| e.ok())
            .collect();
        debug!(
            "mime_strs were parsed into mime_types: {:?}",
            mimes_and_commands
        );
        mimes_and_commands
    }

    pub fn store(self) -> Result<()> {
        confy::store("opener", self)?;
        Ok(())
    }
}

struct EditConfig {
    doc: Document,
}
