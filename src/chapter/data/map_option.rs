use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::utils::csv;

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
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut lines_iterator = file_content.lines();

    let Some(header_line) = lines_iterator.next() else {
        return Err(MapOptionError::MissingHeaders);
    };

    let headers_map: HashMap<&str, usize> = header_line
        .split(separator_char)
        .enumerate()
        .map(|(index, header_string)| (header_string.trim(), index))
        .collect();

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

        let parts: Vec<&str> = trimmed_data.split(separator_char).collect();

        let get_value = |header_name: &str, fallback_index: usize| -> Option<&str> {
            if let Some(&column_index) = headers_map.get(header_name) {
                if let Some(value) = parts.get(column_index) {
                    return Some(value.trim());
                }
            }
            if let Some(value) = parts.get(fallback_index) {
                return Some(value.trim());
            }
            None
        };

        // Determine offset for shifted columns
        let mut column_offset = 0;
        if let Some(column_2_value) = parts.get(2) {
            let trimmed_col2 = column_2_value.trim();
            if trimmed_col2.is_empty() || trimmed_col2.parse::<u32>().is_err() {
                column_offset = 1;
            }
        } else {
            column_offset = 1;
        }

        let Some(map_id_string) = get_value("stageID", 0) else { continue; };
        let Ok(map_id) = map_id_string.parse::<u32>() else { continue; };

        let mut max_crowns = 1;
        if let Some(value_str) = get_value("星解放", 1) {
            if let Ok(parsed_value) = value_str.parse::<u8>() {
                max_crowns = parsed_value;
            }
        }

        let mut has_abyss = false;
        if let Some(value_str) = get_value("裏星解放", 2) {
            if let Ok(parsed_value) = value_str.parse::<u8>() {
                if parsed_value == 1 {
                    has_abyss = true;
                }
            }
        }

        let mut reset_type_val = 0;
        if let Some(value_str) = get_value("報酬リセットType", 8 + column_offset) {
            if let Ok(parsed_value) = value_str.parse::<u8>() {
                reset_type_val = parsed_value;
            }
        }

        let mut max_clears = 0;
        if let Some(value_str) = get_value("1度きり表示", 9 + column_offset) {
            if let Ok(parsed_value) = value_str.parse::<u32>() {
                max_clears = parsed_value;
            }
        }

        let mut cooldown_minutes = 0;
        if let Some(value_str) = get_value("インターバル", 11 + column_offset) {
            if let Ok(parsed_value) = value_str.parse::<u32>() {
                cooldown_minutes = parsed_value;
            }
        }

        let mut hidden_upon_clear = false;
        if let Some(value_str) = get_value("クリア後非表示", 14 + column_offset) {
            if let Ok(parsed_value) = value_str.parse::<u8>() {
                if parsed_value == 1 {
                    hidden_upon_clear = true;
                }
            }
        }

        entries.insert(
            map_id,
            MapOptionEntry {
                map_id,
                max_crowns,
                has_abyss,
                crown_1_mag: get_value("星1倍率", 3 + column_offset).and_then(|val| val.parse::<u32>().ok()),
                crown_2_mag: get_value("星2倍率", 4 + column_offset).and_then(|val| val.parse::<u32>().ok()),
                crown_3_mag: get_value("星3倍率", 5 + column_offset).and_then(|val| val.parse::<u32>().ok()),
                crown_4_mag: get_value("星4倍率", 6 + column_offset).and_then(|val| val.parse::<u32>().ok()),
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