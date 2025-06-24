use std::fmt::Display;

use crate::models::entry::IniEntry;

#[derive(Debug, Default)]
pub struct IniSection<'content> {
    pub entries: Vec<IniEntry<'content>>,
}

impl<'content> IniSection<'content> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_value_by_key(&self, key: &str) -> Option<&'content str> {
        self.entries
            .iter()
            .find_map(|entry| if entry.key == key { Some(entry.value) } else { None })
    }
}

impl<'content> Display for IniSection<'content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in self.entries.iter() {
            writeln!(f, "{entry}")?;
        }
        Ok(())
    }
}
