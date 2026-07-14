use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

#[derive(Debug)]
pub enum MapOptionError {
    EmptyFile,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResetType {
    #[default]
    None,
    ResetRewards,
    ResetRewardsAndClear,
    ResetMaxClears,
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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapOptionEntry {
    pub map_id: u32,
    pub max_crowns: u8,
    pub has_abyss: bool,
    pub crown_1_mag: Option<u32>,
    pub crown_2_mag: Option<u32>,
    pub crown_3_mag: Option<u32>,
    pub crown_4_mag: Option<u32>,
    pub reset_type: ResetType,
    pub max_clears: u32,
    pub cooldown_minutes: u32,
    pub hidden_upon_clear: bool,
    pub comment: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapOption {
    pub entries: HashMap<u32, MapOptionEntry>,
}

impl MapOption {
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
        for i in 0..missing_cols.min(injection_points.len()) {
            let (inject_idx, default_val) = injection_points[i];
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