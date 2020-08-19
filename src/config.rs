use std::collections::HashMap;

use anyhow::{Context, Result};
use log::*;
use serde_derive::{Deserialize, Serialize};
use mime::Mime;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    mime_types: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let cfg_name = "opener";

        confy::load("opener").context(format!(
            "Failed to load configuration file {}.toml",
            cfg_name
        ))
    }

    pub fn get_mime_types(&mut self) -> HashMap<Mime, String> {
        let mimes: HashMap<mime::Mime, String> = self
            .mime_types
            .drain()
            .map(|(k, v)| {
                let mime: Result<Mime> = k
                    .parse()
                    .context(format!("Failed to parse mime type from string {}", k));
                mime.map(|m| (m, v))
            })
            .inspect(|r| {
                if let Err(e) = r {
                    warn!("{:?}", e);
                }
            })
            .filter_map(|e| e.ok())
            .collect();
        debug!("{:?}", mimes);
        mimes
    }
}
