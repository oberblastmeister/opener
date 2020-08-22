use anyhow::{anyhow, bail, Context, Result};
use log::*;
use mime::Mime;
use serde_derive::Deserialize;
use std::collections::HashMap;

use super::load_to_string;

/// What the config will serialize into at first. This will then be converted into `OpenConfig` to
/// use `Mime`s instead of `String`s.
#[derive(Debug, Default, Deserialize)]
struct OpenConfigString {
    open: Vec<HashMap<String, String>>,
    preview: Vec<HashMap<String, String>>,
}

impl OpenConfigString {
    /// Gets the config strings and then deserializes it into `OpenConfigString`
    fn load() -> Result<Self> {
        let cfg_string = load_to_string()?;
        let toml: Self = toml::from_str(&cfg_string)?;
        Ok(toml)
    }

    /// Converts `OpenConfigString` into `OpenConfig`
    fn convert(self) -> OpenConfig {
        let OpenConfigString { open, preview } = self;
        let open = convert_section(open);
        let preview = convert_section(preview);
        OpenConfig { open, preview }
    }
}

/// Converts runs convert_hashmap() on all hashmaps in the vector
fn convert_section(section: Vec<HashMap<String, String>>) -> Vec<HashMap<Mime, String>> {
    section
        .into_iter()
        .map(|hashmap| convert_hashmap(hashmap))
        .collect()
}

/// Converts a hashmap of mime strings and commands into a hashmap of mimes and commands. This
/// function will log the errors using warn! and then discard them.
fn convert_hashmap(map: HashMap<String, String>) -> HashMap<Mime, String> {
    let converted = map
        .into_iter()
        .map(|(mime_str, command)| {
            let mime: Result<Mime> = mime_str.parse().context(format!(
                "Failed to parse mime type from string {}",
                mime_str
            ));
            mime.map(|m| (m, command))
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
    debug!("mime_strs were parsed into mime_types: {:?}", converted);

    converted
}

/// The actual config that will be parsed into because it uses `Mime`s instead of `String`s. This config
/// is used because it is quick and easy but cannot be used to modify the contents of the toml
/// file, only read them. That is why it is called `OpenConfig` because it is only used to open
/// files and will only need read-only data.
pub struct OpenConfig {
    pub open: Vec<HashMap<Mime, String>>,
    pub preview: Vec<HashMap<Mime, String>>,
}

impl OpenConfig {
    /// Loads the config
    pub fn load() -> Result<Self> {
        Ok(OpenConfigString::load()?.convert())
    }
}
