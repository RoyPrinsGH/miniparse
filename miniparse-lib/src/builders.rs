use crate::models::{IniEntry, IniFile, IniSection};

#[derive(Debug, Default)]
pub enum SectionId<'content> {
    #[default]
    Global,
    Named(&'content str),
}

#[derive(Debug, Default)]
pub struct IniSectionBuilder<'content> {
    section: IniSection<'content>,
    id: SectionId<'content>,
}

impl<'content> IniSectionBuilder<'content> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_id(mut self, id: SectionId<'content>) -> Self {
        self.id = id;
        self
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

#[derive(Debug, Default)]
pub struct IniFileBuilder<'content> {
    ini_file: IniFile<'content>,
}

impl<'content> IniFileBuilder<'content> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_section(mut self, name: &'content str, section: IniSection<'content>) -> Self {
        self.ini_file.sections.insert(name, section);
        self
    }

    pub fn set_global_section(mut self, section: IniSection<'content>) -> Self {
        self.ini_file.global_section = Some(section);
        self
    }

    pub fn build(self) -> IniFile<'content> {
        self.ini_file
    }
}
