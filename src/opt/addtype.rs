use std::path::Path;

use anyhow::{anyhow, bail, Result};
use mime::Mime;

use crate::mime_helpers::determine_mime;

#[derive(Debug)]
pub enum AddType<'a> {
    Extension(String),
    Mime(mime::Mime),
    Path(&'a Path),
}

impl<'a> AddType<'a> {
    pub fn determine(candidate: &'a str) -> Result<Self> {
        if candidate.starts_with('.') {
            return Ok(AddType::Extension(candidate.chars().skip(1).collect()));
        }

        if let Ok(mime) = candidate.parse::<mime::Mime>() {
            return Ok(AddType::Mime(mime));
        }

        let path = Path::new(candidate);
        if path.exists() {
            return Ok(AddType::Path(path));
        }

        bail!("The supplied string is not an extension, mime, or path.");
    }

    pub fn convert_to_mime(&self) -> Result<Mime> {
        match self {
            AddType::Extension(ext) => Ok(mime_guess::from_ext(ext)
                .first()
                .ok_or(anyhow!("No mime type found from extension {}", ext))?),
            AddType::Mime(mime) => Ok(mime.clone()),
            AddType::Path(path) => determine_mime(path),
        }
    }
}
