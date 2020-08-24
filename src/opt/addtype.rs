use std::path::PathBuf;
use std::convert::TryFrom;

use anyhow::{anyhow, bail, Result};
use mime::Mime;

use crate::mime_helpers::determine_mime;

pub fn parse_addtype(src: &str) -> Result<AddType, anyhow::Error> {
    AddType::try_from(src)
}

#[derive(Debug)]
pub enum AddType {
    Extension(String),
    Mime(Mime),
    Path(PathBuf),
}

impl TryFrom<&str> for AddType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        if value.starts_with('.') {
            return Ok(AddType::Extension(value.chars().skip(1).collect()));
        }

        if let Ok(mime) = value.parse::<mime::Mime>() {
            return Ok(AddType::Mime(mime));
        }

        let path = PathBuf::from(value);
        if path.exists() {
            return Ok(AddType::Path(path));
        }

        bail!("The supplied string is not an extension, mime, or path.");
    }
}

impl AddType {
    pub fn convert_to_mime(&self) -> Result<Mime> {
        match self {
            AddType::Extension(ext) => Ok(mime_guess::from_ext(&ext)
                .first()
                .ok_or(anyhow!("No mime type found from extension {}", ext))?),
            AddType::Mime(mime) => Ok(mime.clone()),
            AddType::Path(path) => determine_mime(path),
        }
    }
}
