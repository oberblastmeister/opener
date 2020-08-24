use anyhow::{bail, Context, Result};
use mime::Mime;
use std::path::Path;

pub fn mime_equal(m1: &Mime, m2: &Mime) -> bool {
    let m1_type = m1.type_().as_str();
    let m1_subtype = m1.subtype().as_str();
    let m2_type = m2.type_().as_str();
    let m2_subtype = m2.subtype().as_str();

    if m1 == m2 {
        return true;
    }

    if m1_type == m2_type {
        if m2_subtype == "*" {
            return true;
        } else if m1_subtype == "*" {
            return true;
        }
    }

    false
}

fn tree_magic_mime(path: impl AsRef<Path>) -> Result<Mime> {
    let path = path.as_ref();
    if !path.exists() {
        bail!("The path {} does not exist", path.display());
    }
    let mime_string = tree_magic::from_filepath(path.as_ref());

    mime_string.parse::<Mime>().context(format!(
        "Failed to parse string {} returned by tree_magic into a mime type",
        mime_string
    ))
}

/// Determines the mime type from the given path. First uses the extension and then uses tree_magic
/// if using the extension failed.
pub fn determine_mime(path: impl AsRef<Path>) -> Result<Mime> {
    Ok(mime_guess::from_path(&path).first_or(tree_magic_mime(&path)?))
}
