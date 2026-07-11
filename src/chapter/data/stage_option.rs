use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::chapter::domain::redirect::redirect_map_id;
use crate::common::utils::csv;

#[derive(Debug)]
pub enum StageOptionError {
    EmptyFile,
}

impl fmt::Display for StageOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid stage option data."
            ),
        }
    }
}

impl std::error::Error for StageOptionError {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageOptionEntry {
    pub map_id: u32,
    pub target_crowns: i8,
    pub target_stage: i32,
    pub rarity_mask: u8,
    pub deploy_limit: u32,
    pub allowed_rows: u8,
    pub min_cost: u32,
    pub max_cost: u32,
    pub charagroup_id: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageOption {
    pub entries: HashMap<u32, Vec<StageOptionEntry>>,
}

impl StageOption {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, StageOptionError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<StageOption, StageOptionError> {
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut entries: HashMap<u32, Vec<StageOptionEntry>> = HashMap::new();
    let mut has_content = false;

    let lines_iterator = file_content.lines().skip(1);

    for file_line in lines_iterator {
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
        if parts.len() < 9 {
            continue;
        }

        let Some(raw_map_id_str) = parts.first() else { continue; };
        let Ok(raw_map_id) = raw_map_id_str.trim().parse::<u32>() else { continue; };

        let map_id = redirect_map_id(raw_map_id);

        let mut target_crowns: i8 = -1;
        if let Some(val_str) = parts.get(1) {
            if let Ok(parsed) = val_str.trim().parse::<i8>() {
                target_crowns = parsed;
            }
        }

        let mut target_stage: i32 = -1;
        if let Some(val_str) = parts.get(2) {
            if let Ok(parsed) = val_str.trim().parse::<i32>() {
                target_stage = parsed;
            }
        }

        let mut rarity_mask: u8 = 0;
        if let Some(val_str) = parts.get(3) {
            if let Ok(parsed) = val_str.trim().parse::<u8>() {
                rarity_mask = parsed;
            }
        }

        let mut deploy_limit: u32 = 0;
        if let Some(val_str) = parts.get(4) {
            if let Ok(parsed) = val_str.trim().parse::<u32>() {
                deploy_limit = parsed;
            }
        }

        let mut allowed_rows: u8 = 0;
        if let Some(val_str) = parts.get(5) {
            if let Ok(parsed) = val_str.trim().parse::<u8>() {
                allowed_rows = parsed;
            }
        }

        let mut min_cost: u32 = 0;
        if let Some(val_str) = parts.get(6) {
            if let Ok(parsed) = val_str.trim().parse::<u32>() {
                min_cost = parsed;
            }
        }

        let mut max_cost: u32 = 0;
        if let Some(val_str) = parts.get(7) {
            if let Ok(parsed) = val_str.trim().parse::<u32>() {
                max_cost = parsed;
            }
        }

        let mut charagroup_id: u32 = 0;
        if let Some(val_str) = parts.get(8) {
            if let Ok(parsed) = val_str.trim().parse::<u32>() {
                charagroup_id = parsed;
            }
        }

        let entry = StageOptionEntry {
            map_id,
            target_crowns,
            target_stage,
            rarity_mask,
            deploy_limit,
            allowed_rows,
            min_cost,
            max_cost,
            charagroup_id,
        };

        entries.entry(map_id).or_default().push(entry);
    }

    if !has_content {
        return Err(StageOptionError::EmptyFile);
    }

    Ok(StageOption { entries })
}