use std::path::PathBuf;
use std::convert::TryFrom;

use anyhow::{anyhow, bail, Result};
use mime::Mime;

use crate::mime_helpers::determine_mime;

pub fn parse_addtype(src: &str) -> Result<ExtMimePath, anyhow::Error> {
    ExtMimePath::try_from(src)
}

#[derive(Debug)]
pub enum ExtMimePath {
    Extension(String),
    Mime(Mime),
    Path(PathBuf),
}

impl TryFrom<&str> for ExtMimePath {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        if value.starts_with('.') {
            return Ok(ExtMimePath::Extension(value.chars().skip(1).collect()));
        }

        if let Ok(mime) = value.parse::<Mime>() {
            return Ok(ExtMimePath::Mime(mime));
        }

        let path = PathBuf::from(value);
        if path.exists() {
            return Ok(ExtMimePath::Path(path));
        }

        bail!("The supplied string is not an extension, mime, or path.");
    }
}

impl TryFrom<ExtMimePath> for Mime {
    type Error = anyhow::Error;

    fn try_from(value: ExtMimePath) -> Result<Self> {
        match value {
            ExtMimePath::Extension(ext) => Ok(mime_guess::from_ext(&ext)
                .first()
                .ok_or(anyhow!("No mime type found from extension {}", ext))?),
            ExtMimePath::Mime(mime) => Ok(mime),
            ExtMimePath::Path(path) => determine_mime(path),
        }
    }
}
