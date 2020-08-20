use anyhow::{bail, Context, Result};
use mime::Mime;
use std::collections::HashMap;
use std::path::Path;

fn test_mime_equal(m1: &Mime, m2: &Mime) -> bool {
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

/// Compares to mime types. If they are equal returns Some(mime type). If one contains a star it
/// returns the more specific of the two mime types. For example "text/*" compared with
/// "text/plain" will yeild Some("text/plain") because that is the more specific mime_type. If the
/// mime types are equal it will just return one of the mime types. If the mime types are not equal
/// in any way this will return None.
fn compare_mimes(m1: Mime, m2: Mime) -> Option<Mime> {
    let m1_type = m1.type_().as_str();
    let m1_subtype = m1.subtype().as_str();
    let m2_type = m2.type_().as_str();
    let m2_subtype = m2.subtype().as_str();

    if m1 == m2 {
        return Some(m1);
    }

    if m1_type == m2_type {
        if m2_subtype == "*" {
            return Some(m1);
        } else if m1_subtype == "*" {
            return Some(m2);
        }
    }

    None
}

pub fn get_guess(path: impl AsRef<Path>) -> Result<Mime> {
    Ok(mime_guess::from_path(&path)
        .first()
        .unwrap_or(tree_magic_mime(&path)?))
}

pub fn filter_matches(
    mime_match: Mime,
    mimes_and_commands: HashMap<Mime, String>,
) -> HashMap<Mime, String> {
    mimes_and_commands
        .into_iter()
        .filter(|(mime, _command)| test_mime_equal(&mime_match, mime))
        .collect()
}

pub fn remove_star_mimes(mimes_and_commands: HashMap<Mime, String>) -> HashMap<Mime, String> {
    mimes_and_commands
        .into_iter()
        .filter(|(mime, _command)| mime.subtype().as_str() != "*" && mime.type_().as_str() != "*")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_compare_mimes(m1: &str, m2: &str, expected: Option<&str>) {
        let m1 = m1.parse().expect("Failed to parse mime type from string");
        let m2 = m2.parse().expect("Failed to parse mime type from string");
        let expected = expected.map(|e| {
            e.parse::<Mime>()
                .expect("Failed to parse mime type from string")
        });
        assert_eq!(compare_mimes(m1, m2), expected);
    }

    #[test]
    fn compare_mimes_equal_test() {
        test_compare_mimes("text/plain", "text/plain", Some("text/plain"));
    }

    #[test]
    fn compaire_mimes_star_test() {
        test_compare_mimes("application/pdf", "application/*", Some("application/pdf"))
    }

    #[test]
    fn compare_mimes_equal_star_test() {
        test_compare_mimes("image/*", "image/*", Some("image/*"))
    }

    #[test]
    fn compare_mimes_type_nonequal_test() {
        test_compare_mimes("image/*", "application/*", None)
    }

    #[test]
    fn compare_mimes_subtype_nonequal_test() {
        test_compare_mimes("image/png", "image/jpeg", None)
    }

    #[test]
    fn compaire_mimes_both_star_test() {
        test_compare_mimes("*/*", "*/*", Some("*/*"))
    }

    #[test]
    fn tree_magic_test() -> Result<()> {
        assert_eq!(tree_magic_mime("dude")?, mime::TEXT_PLAIN);

        Ok(())
    }

    #[test]
    fn tree_magic_test_pdf() -> Result<()> {
        assert_eq!(tree_magic_mime("latex_book.pdf")?, mime::APPLICATION_PDF);

        Ok(())
    }
}
