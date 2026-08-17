//! Per-map behavioral configuration such as crown tiers and repeat timers.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

/// Represents errors that can occur during the parsing of map options.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapOptionError {
    /// The supplied bytes yielded no parseable rows beyond the header.
    EmptyFile,
    /// The supplied bytes did not begin with the required header row.
    MissingHeaders,
}

impl fmt::Display for MapOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid map option data."
            ),
            Self::MissingHeaders => write!(
                formatter,
                "The file is missing the required header row."
            ),
        }
    }
}

impl std::error::Error for MapOptionError {}

/// Selects what a map discards when its repeat timer elapses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResetType {
    /// Nothing is reset, so the map's rewards are claimable once only.
    #[default]
    None,
    /// The map's reward flags are cleared, making its drops claimable again.
    ResetRewards,
    /// The reward flags and the cleared marker are both cleared.
    ResetRewardsAndClear,
    /// The accumulated clear count is cleared.
    ResetMaxClears,
    /// A reset code this parser does not recognize, carrying its raw value.
    Unknown(u8),
}

impl From<u8> for ResetType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::ResetRewards,
            2 => Self::ResetRewardsAndClear,
            3 => Self::ResetMaxClears,
            _ => Self::Unknown(value),
        }
    }
}

/// The behavioral configuration of a single map.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapOptionEntry {
    /// The identifier of the map this configuration applies to.
    pub map_id: u32,
    /// The highest crown difficulty the map can be attempted at.
    pub max_crowns: u8,
    /// Whether the map exposes the additional Abyss difficulty tier.
    pub has_abyss: bool,
    /// The enemy strength percentage applied at one crown, if declared.
    pub crown_1_mag: Option<u32>,
    /// The enemy strength percentage applied at two crowns, if declared.
    pub crown_2_mag: Option<u32>,
    /// The enemy strength percentage applied at three crowns, if declared.
    pub crown_3_mag: Option<u32>,
    /// The enemy strength percentage applied at four crowns, if declared.
    pub crown_4_mag: Option<u32>,
    /// What the map discards when its repeat timer elapses.
    pub reset_type: ResetType,
    /// The number of times the map may be cleared for rewards.
    pub max_clears: u32,
    /// The delay in minutes before the map becomes available again.
    pub cooldown_minutes: u32,
    /// Whether the map is removed from the selection list once cleared.
    pub hidden_upon_clear: bool,
    /// The trailing comment text accompanying the row in the source file.
    pub comment: String,
}

/// The parsed contents of the map option table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapOption {
    /// The map configurations, keyed by map identifier.
    pub entries: HashMap<u32, MapOptionEntry>,
}

impl MapOption {
    /// Parses the map option table into per-map behavioral configurations.
    ///
    /// The leading header row is skipped. Rows short of the expected column
    /// count have known-optional columns reinserted at their documented
    /// positions, which keeps older regional files readable against the current
    /// layout. Trailing comment text is retained on the resulting entry.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the map option file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `MapOption` on success, or a
    /// `MapOptionError` if the header was absent or no rows were parseable.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, MapOptionError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<MapOption, MapOptionError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::detect_separator(&file_content);

    let mut lines_iterator = file_content.lines();

    if lines_iterator.next().is_none() {
        return Err(MapOptionError::MissingHeaders);
    }

    let mut entries = HashMap::new();
    let mut has_content = false;

    for file_line in lines_iterator {
        let mut data_part = file_line;
        let mut comment_part = "";

        if let Some((before_comment, after_comment)) = file_line.split_once("//") {
            data_part = before_comment;
            comment_part = after_comment;
        }

        let trimmed_data = data_part.trim();
        if trimmed_data.is_empty() {
            continue;
        }

        has_content = true;

        let mut parts: Vec<&str> = trimmed_data.split(separator_char).collect();

        let expected_columns: usize = 19;
        let injection_points = [(2, "0")];

        let missing_cols = expected_columns.saturating_sub(parts.len());
        for &(inject_idx, default_val) in injection_points.iter().take(missing_cols) {
            if inject_idx <= parts.len() {
                parts.insert(inject_idx, default_val);
            }
        }

        let get_value = |index: usize| -> Option<&str> {
            parts.get(index).map(|s| s.trim())
        };

        let Some(map_id_string) = get_value(0) else { continue; };
        let Ok(map_id) = map_id_string.parse::<u32>() else { continue; };

        let max_crowns = get_value(1)
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(1);

        let has_abyss = get_value(2)
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(0) == 1;

        let reset_type_val = get_value(8)
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(0);

        let max_clears = get_value(9)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let cooldown_minutes = get_value(11)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let hidden_upon_clear = get_value(14)
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(0) == 1;

        entries.insert(
            map_id,
            MapOptionEntry {
                map_id,
                max_crowns,
                has_abyss,
                crown_1_mag: get_value(3).and_then(|val| val.parse::<u32>().ok()),
                crown_2_mag: get_value(4).and_then(|val| val.parse::<u32>().ok()),
                crown_3_mag: get_value(5).and_then(|val| val.parse::<u32>().ok()),
                crown_4_mag: get_value(6).and_then(|val| val.parse::<u32>().ok()),
                reset_type: ResetType::from(reset_type_val),
                max_clears,
                cooldown_minutes,
                hidden_upon_clear,
                comment: comment_part.trim().to_string(),
            },
        );
    }

    if !has_content {
        return Err(MapOptionError::EmptyFile);
    }

    Ok(MapOption { entries })
}