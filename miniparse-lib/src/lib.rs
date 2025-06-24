mod builders;
pub mod models;

use regex::Regex;
use thiserror::Error;

pub use crate::builders::{IniFileBuilder, IniSectionBuilder, SectionId};
use crate::models::{IniEntry, IniFile};

pub const ENTRY_KEY_GROUP_NAME: &str = "key";
pub const ENTRY_VALUE_GROUP_NAME: &str = "value";
pub const SECTION_NAME_GROUP_NAME: &str = "section_name";

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Regex compilation error: {0}")]
    RegexCompilationError(#[from] regex::Error),
    #[error("The group {0} was not found in the provided regex")]
    RegexCaptureGroupNotFound(&'static str),
    #[error("Regex match, but the given named group {0} was not found: Did the regex capture group name change?")]
    RegexCaptureGroupNameMismatch(&'static str),
}

pub fn parse<'content>(ini_string: &'content str) -> Result<IniFile<'content>, ParseError> {
    let key_value_regex = Regex::new(&format!(
        r"^\s*(?P<{ENTRY_KEY_GROUP_NAME}>[^=\s]+)\s*=\s*(?P<{ENTRY_VALUE_GROUP_NAME}>[^=\s]+)\s*$"
    ))?;
    let section_header_regex = Regex::new(&format!(r"^\[(?P<{SECTION_NAME_GROUP_NAME}>.+)\]$"))?;

    let mut ini_file_builder = IniFileBuilder::new();
    let mut current_section_builder = IniSectionBuilder::new();

    for line in ini_string.lines().map(str::trim) {
        log::debug!("Parsing line: {line}");

        if let Some(key_value_captures) = key_value_regex.captures(line) {
            log::debug!("Line matched key-value regex.");
            current_section_builder = current_section_builder.add_entry(IniEntry::try_from(key_value_captures)?);
            continue;
        }

        if let Some(section_header_captures) = section_header_regex.captures(line) {
            log::debug!("Line matched section start regex");

            let (id, section) = current_section_builder.build();

            log::debug!("Adding section {id:?}: {section:?}");

            ini_file_builder = match id {
                // Do not add global section if it is empty. We can do this with named sections, because their start is explicit
                // but global section definitions are implicit.
                SectionId::Global if !section.entries.is_empty() => ini_file_builder.set_global_section(section),
                SectionId::Named(name) => ini_file_builder.new_section(name, section),
                _ => ini_file_builder,
            };

            match section_header_captures.name(SECTION_NAME_GROUP_NAME) {
                Some(section_name) => current_section_builder = IniSectionBuilder::new().set_id(SectionId::Named(section_name.as_str())),
                None => return Err(ParseError::RegexCaptureGroupNotFound(SECTION_NAME_GROUP_NAME)),
            }

            continue;
        }

        if line.is_empty() {
            continue;
        }

        log::warn!("Skipping unparsable non-empty line: {line}");
    }

    log::debug!("End of file reached. Adding current section, if we are building one.");

    let (id, section) = current_section_builder.build();

    log::debug!("Adding section {id:?}: {section:?}");

    ini_file_builder = match id {
        SectionId::Global if !section.entries.is_empty() => ini_file_builder.set_global_section(section),
        SectionId::Named(name) => ini_file_builder.new_section(name, section),
        _ => ini_file_builder,
    };

    log::debug!("Building ini file");

    Ok(ini_file_builder.build())
}

#[cfg(test)]
mod tests {
    use crate::{IniFileBuilder, builders::IniSectionBuilder, parse};

    #[test]
    fn parse_happy_flow_no_global_section() {
        let (_, section1) = IniSectionBuilder::new()
            .add_key_value_pair("key1", "value11")
            .add_key_value_pair("key2", "value12")
            .add_key_value_pair("key3", "value13")
            .build();

        let (_, section2) = IniSectionBuilder::new()
            .add_key_value_pair("key1", "value21")
            .add_key_value_pair("key2", "value22")
            .build();

        let dummy_ini = IniFileBuilder::new()
            .new_section("section1", section1)
            .new_section("section2", section2)
            .build();

        let dummy_ini_string = dummy_ini.to_string();

        let parsed_ini = parse(dummy_ini_string.as_str()).unwrap();

        assert_eq!(parsed_ini.sections.len(), dummy_ini.sections.len());

        assert!(parsed_ini.get_global_section().is_none());

        assert_eq!(parsed_ini.get_section_by_name("section1").unwrap().entries.len(), 3);

        assert_eq!(parsed_ini.get_section_by_name("section2").unwrap().entries.len(), 2);

        let parsed_section1 = parsed_ini.get_section_by_name("section1").unwrap();

        assert_eq!(parsed_section1.get_value_by_key("key1").unwrap(), "value11");
        assert_eq!(parsed_section1.get_value_by_key("key2").unwrap(), "value12");
        assert_eq!(parsed_section1.get_value_by_key("key3").unwrap(), "value13");

        let parsed_section2 = parsed_ini.get_section_by_name("section2").unwrap();

        assert_eq!(parsed_section2.get_value_by_key("key1").unwrap(), "value21");
        assert_eq!(parsed_section2.get_value_by_key("key2").unwrap(), "value22");
    }

    #[test]
    fn parse_happy_flow_with_global_section() {
        let (_, global_section) = IniSectionBuilder::new()
            .add_key_value_pair("g_key1", "g_value11")
            .add_key_value_pair("g_key2", "g_value12")
            .add_key_value_pair("g_key3", "g_value13")
            .build();

        let (_, section1) = IniSectionBuilder::new()
            .add_key_value_pair("key1", "value21")
            .add_key_value_pair("key2", "value22")
            .build();

        let dummy_ini = IniFileBuilder::new()
            .set_global_section(global_section)
            .new_section("section1", section1)
            .build();

        let dummy_ini_string = dummy_ini.to_string();

        let parsed_ini = parse(dummy_ini_string.as_str()).unwrap();

        assert_eq!(parsed_ini.sections.len(), dummy_ini.sections.len());

        assert_eq!(parsed_ini.get_global_section().unwrap().entries.len(), 3);

        assert_eq!(parsed_ini.get_section_by_name("section1").unwrap().entries.len(), 2);

        let parsed_global_section = parsed_ini.get_global_section().unwrap();

        assert_eq!(parsed_global_section.get_value_by_key("g_key1").unwrap(), "g_value11");
        assert_eq!(parsed_global_section.get_value_by_key("g_key2").unwrap(), "g_value12");
        assert_eq!(parsed_global_section.get_value_by_key("g_key3").unwrap(), "g_value13");

        let parsed_section1 = parsed_ini.get_section_by_name("section1").unwrap();

        assert_eq!(parsed_section1.get_value_by_key("key1").unwrap(), "value21");
        assert_eq!(parsed_section1.get_value_by_key("key2").unwrap(), "value22");
    }
}
