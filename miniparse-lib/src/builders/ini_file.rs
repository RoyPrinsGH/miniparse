use crate::models::{IniFile, IniSection};

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
