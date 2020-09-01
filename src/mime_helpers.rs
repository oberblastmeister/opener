use anyhow::{bail, Context, Result};
use mime::Mime;
use std::path::Path;

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
