//! Unit lineup restrictions applied by particular stages.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file::{self, Separator};

/// Represents errors that can occur during the parsing of unit restriction groups.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CharaGroupError {
    /// The supplied bytes yielded no parseable rows.
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

/// Selects how a unit restriction group constrains the player's lineup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharaGroupType {
    /// Only the listed units may be fielded.
    OnlyUse,
    /// The listed units are barred, and all others may be fielded.
    CannotUse,
    /// A restriction code this parser does not recognize, carrying its raw value.
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

/// A single unit restriction group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharaGroupEntry {
    /// The group's own identifier, referenced by the stages that apply it.
    pub id: u32,
    /// How the listed units constrain the player's lineup.
    pub kind: CharaGroupType,
    /// The identifiers of the units the restriction lists.
    pub units: Vec<u32>,
}

/// The parsed contents of the unit restriction group table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharaGroup {
    /// The restriction groups, keyed by group identifier.
    pub groups: HashMap<u32, CharaGroupEntry>,
}

impl CharaGroup {
    /// Parses the unit restriction group table into groups keyed by identifier.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the character group file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `CharaGroup` on success, or a
    /// `CharaGroupError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, CharaGroupError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<CharaGroup, CharaGroupError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

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
        let parsed_kind = kind_str.trim().parse::<u32>().unwrap_or(0);

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