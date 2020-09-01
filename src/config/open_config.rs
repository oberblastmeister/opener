use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use log::*;
use mime::Mime;
use rayon::prelude::*;
use regex::Regex;
use serde_derive::Deserialize;

use super::load_to_string;

type StringCommands = HashMap<String, String>;

/// What the config will serialize into at first. This will then be converted into `OpenConfig` to
/// use `Mime`s instead of `String`s.
#[derive(Debug, Default, Deserialize)]
struct OpenConfigString {
    open: Vec<StringCommands>,
    open_regex: Vec<RegexCommands>,
    preview: Vec<StringCommands>,
    preview_regex: Vec<RegexCommands>,
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
        let open = MimeCommands::new_vec(open);
        let preview = MimeCommands::new_vec(preview);
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
    pub open: Vec<MimeCommands>,
    pub open_regex: Vec<RegexCommands>,
    pub preview: Vec<MimeCommands>,
    pub preview_regex: Vec<RegexCommands>,
}

impl OpenConfig {
    /// Loads the config
    pub fn load() -> Result<Self> {
        Ok(OpenConfigString::load()?.convert())
    }

    pub fn get_preview(self) -> (Vec<MimeCommands>, Vec<RegexCommands>) {
        match self {
            OpenConfig {
                preview,
                preview_regex,
                ..
            } => (preview, preview_regex),
        }
    }

    pub fn get_open(self) -> (Vec<MimeCommands>, Vec<RegexCommands>) {
        match self {
            OpenConfig {
                open, open_regex, ..
            } => (open, open_regex),
        }
    }
}

/// Something that can be narrowed down and return a command
pub trait PossibleCommands {
    type Compare;

    /// Narrow down something according to what is compared against each item and return the
    /// command associated with that item
    fn find_correct_command(self, compare: &Self::Compare) -> Result<Option<String>>;
}

/// The possible mimes and commands that can be used to open a file
#[derive(Debug)]
pub struct MimeCommands(HashMap<Mime, String>);

impl MimeCommands {
    /// Converts a hashmap of mime strings and commands into a hashmap of mimes and commands. This
    /// function will log the errors using warn! and then discard them.
    pub fn new(map: StringCommands) -> MimeCommands {
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

        MimeCommands(converted)
    }

    /// Creates a new vector of possibles. The first possible is the main one used while the other
    /// ones are just fall backs
    pub fn new_vec(map: Vec<StringCommands>) -> Vec<MimeCommands> {
        map.into_par_iter()
            .map(|hashmap| MimeCommands::new(hashmap))
            .collect()
    }
}

impl PossibleCommands for MimeCommands {
    type Compare = Mime;

    /// proper command that was narrowed down.
    fn find_correct_command(self, mime: &Mime) -> Result<Option<String>> {
        let mime_type = mime.type_().as_str();
        let mime_subtype = mime.subtype().as_str();

        if mime_type == "*" || mime_subtype == "*" {
            panic!("mime type must not be that")
        }

        let command = self.0.get(mime);
        if command.is_none() {
            let mime_star = format!("{}/*", mime_type).parse::<Mime>()?;
            // let star_command = self.0[&mime_star]
            let star_command = self.0.get(&mime_star);
            Ok(star_command.map(|s| s.to_owned()))
        } else {
            Ok(command.map(|s| s.to_owned()))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegexCommands(HashMap<String, String>);

impl PossibleCommands for RegexCommands {
    type Compare = String;

    /// Compare is the string filename. It is narrowing down which regex is possibleregexes matches
    /// the filename.
    fn find_correct_command(self, compare: &String) -> Result<Option<String>> {
        // filter out all the commands that have a regex that match
        let commands: Vec<String> = self
            .0
            .into_par_iter()
            .map(|(regex_string, command)| Regex::new(&regex_string).map(|regex| (regex, command)))
            // log regex errors
            .inspect(|result| {
                if let Err(e) = result {
                    warn!("Failed to create regex: {}", e);
                }
            })
            // keep successes
            .filter_map(|result| result.ok())
            // filter matches
            .filter(|(regex, _command)| regex.is_match(&compare))
            // only get commands
            .map(|(_regex, command)| command)
            .collect();

        if commands.len() > 1 {
            info!("Commands are being chosen with rofi");
            Ok(Some(choose_with_rofi(&commands)?))
        } else if commands.len() == 0 {
            info!("No regex commands were found");
            Ok(None)
        } else {
            info!("One regex command was found.");
            Ok(Some(commands.into_iter().collect::<String>()))
        }
    }
}

/// Choose the command with rofi
fn choose_with_rofi(commands: &Vec<String>) -> Result<String> {
    match rofi::Rofi::new(commands).run() {
        Ok(choice) => Ok(choice),
        Err(rofi::Error::Interrupted) => bail!("Rofi was interrupted"),
        Err(e) => Err(e)?,
    }
}
