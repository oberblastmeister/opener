use anyhow::{Context, Result};
use log::*;
use mime::Mime;
use rayon::prelude::*;
use serde_derive::Deserialize;
use std::collections::HashMap;

use super::load_to_string;
use crate::mime_helpers::*;

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
        let open = Possible::new_vec(open);
        let preview = Possible::new_vec(preview);
        OpenConfig { open, preview }
    }
}

/// The actual config that will be parsed into because it uses `Mime`s instead of `String`s. This config
/// is used because it is quick and easy but cannot be used to modify the contents of the toml
/// file, only read them. That is why it is called `OpenConfig` because it is only used to open
/// files and will only need read-only data.
#[derive(Debug)]
pub struct OpenConfig {
    pub open: Vec<Possible>,
    pub preview: Vec<Possible>,
}

impl OpenConfig {
    /// Loads the config
    pub fn load() -> Result<Self> {
        Ok(OpenConfigString::load()?.convert())
    }
}

/// The possible mimes and commands that can be used to open a file
#[derive(Debug)]
pub struct Possible(HashMap<Mime, String>);

impl Possible {
    /// Converts a hashmap of mime strings and commands into a hashmap of mimes and commands. This
    /// function will log the errors using warn! and then discard them.
    pub fn new(map: HashMap<String, String>) -> Possible {
        let converted: HashMap<Mime, String> = map
            .into_par_iter()
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

        Possible(converted)
    }

    /// Creates a new vector of possibles. The first possible is the main one used while the other
    /// ones are just fall backs
    pub fn new_vec(map: Vec<HashMap<String, String>>) -> Vec<Possible> {
        map.into_par_iter()
            .map(|hashmap| Possible::new(hashmap))
            .collect()
    }

    /// Narrows down the possible commands to one according to the mime type given. Then returns the
    /// proper command that was narrowed down.
    pub fn narrow(mut self, mime: &Mime) -> String {
        // first filter them so that only mimes that are equal are kept, including star mimes.
        // application/* == application/pdf is true
        self = self.filter_equal(mime);
        debug!("Matches before narrowing down to 1: {:?}", self);

        if self.matches_not_one() {
            // if there are still more matches, remove the star mimes because each mime type can
            // only have two possible matches, the specific and star. For example application/pdf
            // only has two matches, application/pdf and application/*. If star is removed, the
            // command for application/pdf is left.
            self = self.remove_star_mimes();
        }
        debug!("Matches after narrowing down to 1: {:?}", self);

        // there should only be one match left
        if self.matches_not_one() {
            panic!("BUG: matches length should not be greater than 1. Toml file should have non-repeating strings. After removing stars there can only be one match for each mime type.")
        }

        // there is only one match left so this just returns the command associated with it.
        self.0.into_iter().map(|(_mime, command)| command).collect()
    }

    fn filter_equal(self, mime_match: &Mime) -> Self {
        let map: HashMap<Mime, String> = self
            .0
            .into_par_iter()
            .filter(|(mime, _command)| mime_equal(mime_match, mime))
            .collect();
        Possible(map)
    }

    /// Removes the mimes and commands that have star mimes like text/*
    fn remove_star_mimes(self) -> Self {
        let map: HashMap<Mime, String> = self
            .0
            .into_par_iter()
            .filter(|(mime, _command)| {
                mime.subtype().as_str() != "*" && mime.type_().as_str() != "*"
            })
            .collect();
        Possible(map)
    }

    fn matches_not_one(&self) -> bool {
        self.0.len() > 1
    }
}
