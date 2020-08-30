use std::collections::HashMap;

use anyhow::{Context, Result};
use log::*;
use mime::Mime;
use rayon::prelude::*;
use regex::Regex;
use serde_derive::Deserialize;

use super::load_to_string;
use crate::mime_helpers::*;

type PossibleStrings = HashMap<String, String>;

/// What the config will serialize into at first. This will then be converted into `OpenConfig` to
/// use `Mime`s instead of `String`s.
#[derive(Debug, Default, Deserialize)]
struct OpenConfigString {
    open: Vec<PossibleStrings>,
    open_regex: Vec<PossibleStrings>,
    preview: Vec<PossibleStrings>,
    preview_regex: Vec<PossibleStrings>,
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
        let OpenConfigString {
            open,
            open_regex,
            preview,
            preview_regex,
        } = self;
        let open = PossibleMimes::new_vec(open);
        let preview = PossibleMimes::new_vec(preview);
        OpenConfig {
            open,
            open_regex,
            preview,
            preview_regex,
        }
    }
}

/// The actual config that will be parsed into because it uses `Mime`s instead of `String`s. This config
/// is used because it is quick and easy but cannot be used to modify the contents of the toml
/// file, only read them. That is why it is called `OpenConfig` because it is only used to open
/// files and will only need read-only data.
#[derive(Debug)]
pub struct OpenConfig {
    pub open: Vec<PossibleMimes>,
    pub open_regex: Vec<PossibleStrings>,
    pub preview: Vec<PossibleMimes>,
    pub preview_regex: Vec<PossibleStrings>,
}

impl OpenConfig {
    /// Loads the config
    pub fn load() -> Result<Self> {
        Ok(OpenConfigString::load()?.convert())
    }
}

/// The possible mimes and commands that can be used to open a file
#[derive(Debug)]
pub struct PossibleMimes(HashMap<Mime, String>);

impl PossibleMimes {
    /// Converts a hashmap of mime strings and commands into a hashmap of mimes and commands. This
    /// function will log the errors using warn! and then discard them.
    pub fn new(map: PossibleStrings) -> PossibleMimes {
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

        PossibleMimes(converted)
    }

    /// Creates a new vector of possibles. The first possible is the main one used while the other
    /// ones are just fall backs
    pub fn new_vec(map: Vec<PossibleStrings>) -> Vec<PossibleMimes> {
        map.into_par_iter()
            .map(|hashmap| PossibleMimes::new(hashmap))
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
        PossibleMimes(map)
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
        PossibleMimes(map)
    }

    fn matches_not_one(&self) -> bool {
        self.0.len() > 1
    }
}

struct PossibleRegexes(HashMap<String, String>);

impl Narrowable for PossibleRegexes {
    type Compare = String;

    /// Compare is the string filename. It is narrowing down which regex is possibleregexes matches
    /// the filename.
    fn narrow(self, compare: String) -> Result<String> {
        let commands: Vec<String> = self
            .0
            .into_par_iter()
            .map(|(regex_string, command)| Regex::new(&regex_string).map(|regex| (regex, command)))
            .inspect(|result| {
                if let Err(e) = result {
                    warn!("Failed to create regex: {}", e);
                }
            })
            .filter_map(|result| result.ok())
            .filter(|(regex, _command)| regex.is_match(&compare))
            .map(|(_regex, command)| command)
            .collect();
        if commands.len() > 1 {
            Ok(choose_with_rofi(&commands)?)
        } else {
            Ok(commands.into_iter().collect::<String>())
        }
    }
}

/// Choose the command with rofi
fn choose_with_rofi(commands: &[String]) -> Result<String> {
    todo!()
}

/// Something that can be narrowed down and return a command
trait Narrowable {
    type Compare;

    /// Narrow down something according to what is compared against each item
    fn narrow(self, compare: Self::Compare) -> Result<String>;
}
