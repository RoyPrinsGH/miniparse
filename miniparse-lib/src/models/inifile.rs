use std::{collections::HashMap, fmt::Display};

use crate::models::section::IniSection;

#[derive(Debug, Default)]
pub struct IniFile<'content> {
    pub(crate) global_section: Option<IniSection<'content>>,
    pub(crate) sections: HashMap<&'content str, IniSection<'content>>,
}

impl<'content> IniFile<'content> {
    pub fn get_global_section(&self) -> Option<&IniSection<'content>> {
        (&self.global_section).into()
    }

    pub fn get_section_by_name(&self, name: &str) -> Option<&IniSection<'content>> {
        self.sections.get(name)
    }
}

impl<'content> Display for IniFile<'content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(global_section) = self.get_global_section() {
            writeln!(f, "{global_section}")?;
        }
        for (section_name, section) in self.sections.iter() {
            writeln!(f, "[{section_name}]")?;
            write!(f, "{section}")?;
        }
        Ok(())
    }
}
