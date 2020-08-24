use super::{store_string, load_to_string};
use anyhow::Result;
use toml_edit::{ArrayOfTables, Document, Item, Table};

/// The config that will be parsed into if editing the toml file is needed.
#[derive(Debug)]
pub struct EditConfig {
    doc: Document,
}

impl EditConfig {
    pub fn load() -> Result<Self> {
        let toml_string = load_to_string()?;
        let doc = toml_string.parse::<Document>().expect("invalid doc");
        Ok(EditConfig { doc })
    }

    pub fn root(&mut self) -> &mut Item {
        &mut self.doc.root
    }

    pub fn root_table(&mut self) -> &mut Table {
        self.root()
            .as_table_mut()
            .expect("The the root item should always be a table.")
    }

    pub fn get_open(&mut self) -> Result<&mut ArrayOfTables> {
        let array = self
            .root_table()
            .entry("open")
            .as_array_of_tables_mut()
            .unwrap();
        Ok(array)
    }

    pub fn get_preview(&mut self) -> Result<&mut ArrayOfTables> {
        let array = self
            .root_table()
            .entry("preview")
            .as_array_of_tables_mut()
            .unwrap();
        Ok(array)
    }

    pub fn store(&self) -> Result<()> {
        store_string(&self.doc.to_string())
    }

    pub fn to_string(&self) -> String {
        self.doc.to_string()
    }
}
