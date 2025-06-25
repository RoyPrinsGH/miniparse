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

        if line.is_empty() {
            log::debug!("Line is empty: skipping");
            continue;
        }

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

    fn make_dummy_ini_string() -> String {
        let (_, global_section) = IniSectionBuilder::new()
            .add_key_value_pair("g_key1", "g_value11")
            .add_key_value_pair("g_key2", "g_value12")
            .add_key_value_pair("g_key3", "g_value13")
            .build();

        let (_, section1) = IniSectionBuilder::new()
            .add_key_value_pair("key1", "value21")
            .add_key_value_pair("key2", "value22")
            .build();

        let (_, section2) = IniSectionBuilder::new()
            .add_key_value_pair("key1", "value31")
            .add_key_value_pair("key2", "value32")
            .add_key_value_pair("key3", "value33")
            .build();

        let dummy_ini = IniFileBuilder::new()
            .set_global_section(global_section)
            .new_section("section1", section1)
            .new_section("section2", section2)
            .build();

        dummy_ini.to_string()
    }

    #[test]
    fn parse_happy_flow() {
        let dummy_ini_string = make_dummy_ini_string();
        parse(dummy_ini_string.as_str()).unwrap();
    }

    #[test]
    fn find_existing_section() {
        let dummy_ini_string = make_dummy_ini_string();
        let ini_file = parse(dummy_ini_string.as_str()).unwrap();
        assert!(ini_file.get_section_by_name("section1").is_some())
    }

    #[test]
    fn do_not_find_non_existing_section() {
        let dummy_ini_string = make_dummy_ini_string();
        let ini_file = parse(dummy_ini_string.as_str()).unwrap();
        assert!(ini_file.get_section_by_name("i do not exist").is_none())
    }

    #[test]
    fn find_existing_key() {
        let dummy_ini_string = make_dummy_ini_string();
        let ini_file = parse(dummy_ini_string.as_str()).unwrap();
        let section1 = ini_file.get_section_by_name("section1").unwrap();
        assert!(section1.get_value_by_key("key1").is_some())
    }

    #[test]
    fn do_not_find_non_existing_key() {
        let dummy_ini_string = make_dummy_ini_string();
        let ini_file = parse(dummy_ini_string.as_str()).unwrap();
        let section1 = ini_file.get_section_by_name("section1").unwrap();
        assert!(section1.get_value_by_key("i do not exist").is_none())
    }

    #[test]
    fn find_correct_value() {
        let dummy_ini_string = make_dummy_ini_string();
        let ini_file = parse(dummy_ini_string.as_str()).unwrap();
        let section1 = ini_file.get_section_by_name("section1").unwrap();
        assert_eq!(section1.get_value_by_key("key1").unwrap(), "value21")
    }

    #[test]
    fn find_correct_global_value() {
        let dummy_ini_string = make_dummy_ini_string();
        let ini_file = parse(dummy_ini_string.as_str()).unwrap();
        let global_section = ini_file.get_global_section().unwrap();
        assert_eq!(global_section.get_value_by_key("g_key1").unwrap(), "g_value11")
    }
}
