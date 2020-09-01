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

    pub fn get_open_iter_mut(&mut self) -> Result<ArrayOfTablesIterMut> {
        Ok(ArrayOfTablesIterMut::new(self.get_open()?))
    }

    pub fn get_preview_iter_mut(&mut self) -> Result<ArrayOfTablesIterMut> {
        Ok(ArrayOfTablesIterMut::new(self.get_preview()?))
    }

    pub fn store(&self) -> Result<()> {
        store_string(&self.doc.to_string())
    }

    pub fn to_string(&self) -> String {
        self.doc.to_string()
    }
}

/// Can only be used in while let Some(value) = streaming_iterator.next()
pub trait StreamingIteratorMut {
    type Item: ?Sized;

    fn advance(&mut self);
    fn get_mut(&mut self) -> Option<&mut Self::Item>;
    fn next(&mut self) -> Option<&mut Self::Item> {
        self.advance();
        let item = (*self).get_mut();
        item
    }
}

pub struct ArrayOfTablesIterMut<'a> {
    idx: isize,
    array: &'a mut ArrayOfTables
}

impl<'a> StreamingIteratorMut for ArrayOfTablesIterMut<'a> {
    type Item = Table;

    fn advance(&mut self) {
        self.idx += 1;
    }

    fn get_mut(&mut self) -> Option<&mut Table> {
        self.array.get_mut(self.idx as usize)
    }
}

impl<'a> ArrayOfTablesIterMut<'a> {
    pub fn new(array: &'a mut ArrayOfTables) -> Self {
        ArrayOfTablesIterMut {
            idx: -1,
            array,
        }
    }

    pub fn append_single_entry_table(&mut self, key: &str, value: String) {
        let mut table = toml_edit::Table::new();
        table[key] = toml_edit::value(value);
        self.array.append(table);
    }
}
