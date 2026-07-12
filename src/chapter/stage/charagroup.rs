use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::csv;

#[derive(Debug)]
pub enum CharaGroupError {
    EmptyFile,
}

impl fmt::Display for CharaGroupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid character group data."
            ),
        }
    }
}

impl std::error::Error for CharaGroupError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharaGroupType {
    OnlyUse,
    CannotUse,
    Unknown(u32),
}

impl From<u32> for CharaGroupType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::OnlyUse,
            2 => Self::CannotUse,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharaGroupEntry {
    pub id: u32,
    pub kind: CharaGroupType,
    pub units: Vec<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharaGroup {
    pub groups: HashMap<u32, CharaGroupEntry>,
}

impl CharaGroup {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, CharaGroupError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<CharaGroup, CharaGroupError> {
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut groups = HashMap::new();
    let mut has_content = false;

    for file_line in file_content.lines().skip(1) {
        let mut clean_line = file_line;

        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        if parts.len() < 3 {
            continue;
        }

        let Some(id_str) = parts.first() else { continue; };
        let Ok(id) = id_str.trim().parse::<u32>() else { continue; };

        let Some(kind_str) = parts.get(2) else { continue; };
        let parsed_kind = kind_str.trim().parse::<u32>().unwrap_or_else(|_| 0);

        let mut units = Vec::new();
        for unit_str in parts.iter().skip(3) {
            if let Ok(unit_id) = unit_str.trim().parse::<u32>() {
                units.push(unit_id);
            }
        }

        groups.insert(
            id,
            CharaGroupEntry {
                id,
                kind: CharaGroupType::from(parsed_kind),
                units,
            },
        );
    }

    if !has_content {
        return Err(CharaGroupError::EmptyFile);
    }

    Ok(CharaGroup { groups })
}