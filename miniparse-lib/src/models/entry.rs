use std::fmt::Display;

use regex::Captures;

use crate::{ENTRY_KEY_GROUP_NAME, ENTRY_VALUE_GROUP_NAME, ParseError};

#[derive(Debug)]
pub struct IniEntry<'content> {
    pub key: &'content str,
    pub value: &'content str,
}

impl<'content> Display for IniEntry<'content> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.key, self.value)
    }
}

impl<'content> TryFrom<Captures<'content>> for IniEntry<'content> {
    type Error = ParseError;

    fn try_from(captures: Captures<'content>) -> Result<Self, Self::Error> {
        let key = captures
            .name(ENTRY_KEY_GROUP_NAME)
            .ok_or(ParseError::RegexCaptureGroupNotFound(ENTRY_KEY_GROUP_NAME))?
            .as_str();

        let value = captures
            .name(ENTRY_VALUE_GROUP_NAME)
            .ok_or(ParseError::RegexCaptureGroupNotFound(ENTRY_VALUE_GROUP_NAME))?
            .as_str();

        Ok(Self { key, value })
    }
}
