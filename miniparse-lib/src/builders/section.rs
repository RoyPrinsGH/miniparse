use crate::models::{IniEntry, IniSection, SectionId};

#[derive(Debug, Default)]
pub struct IniSectionBuilder<'content> {
    section: IniSection<'content>,
    id: SectionId<'content>,
}

impl<'content> IniSectionBuilder<'content> {
    pub fn new(id: SectionId<'content>) -> Self {
        Self { id, ..Default::default() }
    }

    pub fn add_entry(mut self, entry: IniEntry<'content>) -> Self {
        self.section.entries.push(entry);
        self
    }

    pub fn add_key_value_pair(self, key: &'content str, value: &'content str) -> Self {
        self.add_entry(IniEntry { key, value })
    }

    pub fn build(self) -> (SectionId<'content>, IniSection<'content>) {
        (self.id, self.section)
    }
}
