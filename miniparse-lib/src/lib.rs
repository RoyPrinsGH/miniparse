mod builders;
pub mod models;

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

pub use crate::builders::{IniFileBuilder, IniSectionBuilder, SectionId};
use crate::models::{IniEntry, IniFile};

pub const ENTRY_KEY_GROUP_NAME: &str = "key";
pub const ENTRY_VALUE_GROUP_NAME: &str = "value";
pub const SECTION_NAME_GROUP_NAME: &str = "section_name";

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("The group {0} was not found in the provided regex")]
    RegexCaptureGroupNotFound(&'static str),
    #[error("Regex match, but the given named group {0} was not found: Did the regex capture group name change?")]
    RegexCaptureGroupNameMismatch(&'static str),
}

fn add_section_to_ini_builder<'content>(
    ini_file_builder: IniFileBuilder<'content>,
    current_section_builder: IniSectionBuilder<'content>,
) -> IniFileBuilder<'content> {
    let (id, section) = current_section_builder.build();

    log::debug!("Adding section {id:?}: {section:?}");

    // Do not add global section if it is empty. We can do this with named sections, because their start is explicit
    // but global section definitions are implicit.
    match id {
        SectionId::Global if !section.entries.is_empty() => ini_file_builder.set_global_section(section),
        SectionId::Named(name) => ini_file_builder.new_section(name, section),
        _ => ini_file_builder,
    }
}

lazy_static! {
    static ref KEY_VALUE_REGEX: Regex = Regex::new(&format!(
        r"^\s*(?P<{ENTRY_KEY_GROUP_NAME}>[^=\s]+)\s*=\s*(?P<{ENTRY_VALUE_GROUP_NAME}>[^=\s]+)\s*$"
    ))
    .expect("Invalid regex!");
    static ref SECTION_HEADER_REGEX: Regex = Regex::new(&format!(r"^\[(?P<{SECTION_NAME_GROUP_NAME}>.+)\]$")).expect("Invalid regex!");
}

pub fn parse<'content>(ini_string: &'content str) -> Result<IniFile<'content>, ParseError> {
    let mut ini_file_builder = IniFileBuilder::new();
    let mut current_section_builder = IniSectionBuilder::new(SectionId::Global);

    for line in ini_string.lines().map(str::trim) {
        log::debug!("Parsing line: {line}");

        if line.is_empty() {
            log::debug!("Line is empty: skipping");
            continue;
        }

        if let Some(key_value_captures) = KEY_VALUE_REGEX.captures(line) {
            log::debug!("Line matched key-value regex.");
            current_section_builder = current_section_builder.add_entry(IniEntry::try_from(key_value_captures)?);
            continue;
        }

        if let Some(section_header_captures) = SECTION_HEADER_REGEX.captures(line) {
            log::debug!("Line matched section start regex, adding current section");
            ini_file_builder = add_section_to_ini_builder(ini_file_builder, current_section_builder);

            let new_section_name = section_header_captures
                .name(SECTION_NAME_GROUP_NAME)
                .ok_or(ParseError::RegexCaptureGroupNotFound(SECTION_NAME_GROUP_NAME))?
                .as_str();

            current_section_builder = IniSectionBuilder::new(SectionId::Named(new_section_name));
            continue;
        }

        log::warn!("Skipping unparsable non-empty line: {line}");
    }

    log::debug!("End of file reached. Adding current section, if we are building one.");
    ini_file_builder = add_section_to_ini_builder(ini_file_builder, current_section_builder);

    log::debug!("Building ini file");
    Ok(ini_file_builder.build())
}

#[cfg(test)]
mod tests {
    use crate::{IniFileBuilder, builders::IniSectionBuilder, parse};

    fn make_dummy_ini_string() -> String {
        let (_, global_section) = IniSectionBuilder::default()
            .add_key_value_pair("g_key1", "g_value11")
            .add_key_value_pair("g_key2", "g_value12")
            .add_key_value_pair("g_key3", "g_value13")
            .build();

        let (_, section1) = IniSectionBuilder::default()
            .add_key_value_pair("key1", "value21")
            .add_key_value_pair("key2", "value22")
            .build();

        let (_, section2) = IniSectionBuilder::default()
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
