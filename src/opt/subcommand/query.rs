use anyhow::{anyhow, Result};
use log::*;
use std::io::{stdout, Write};

use super::parse_addtype;
use super::AddType;
use super::Runable;
use super::StructOpt;
use crate::mime_helpers::determine_mime;

#[derive(StructOpt, Debug)]
pub struct Query {
    #[structopt(parse(try_from_str = parse_addtype))]
    addtype: AddType,
}

impl Runable for Query {
    fn run(self) -> Result<()> {
        match self.addtype {
            AddType::Mime(ref mime) => {
                let extensions = mime_guess::get_mime_extensions(&mime)
                    .ok_or(anyhow!("No mime types found for given extension"))?;
                let stdout = stdout();
                let mut stdout = stdout.lock();
                for extension in extensions {
                    write!(stdout, ".{} ", extension).unwrap();
                }
                write!(stdout, "\n").unwrap();
                stdout.flush().unwrap();
            }
            AddType::Path(ref path) => {
                let mime_string = determine_mime(path)?.to_string();
                println!("{}", mime_string);
            }
            AddType::Extension(ref ext) => {
                let mime_string = mime_guess::from_ext(&ext)
                    .first()
                    .ok_or(anyhow!("Could not get mime type from extension"))?
                    .to_string();
                println!("{}", mime_string);
            }
        }
        Ok(())
    }
}
