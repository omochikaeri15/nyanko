use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

#[derive(Debug)]
pub enum MapStageDataError {
    EmptyFile,
}

impl fmt::Display for MapStageDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid map stage data."
            ),
        }
    }
}

impl std::error::Error for MapStageDataError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DropReward {
    pub chance: u32,
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedScore {
    pub score: u32,
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RewardStructure {
    #[default]
    None,
    Treasure {
        drop_rule: i32,
        drops: Vec<DropReward>,
    },
    Timed(Vec<TimedScore>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageDataEntry {
    pub energy: u32,
    pub xp: u32,
    pub init_track: u32,
    pub bgm_change_percent: u32,
    pub boss_track: i16,
    pub rewards: RewardStructure,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageData {
    pub entries: Vec<MapStageDataEntry>,
}

impl MapStageData {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, MapStageDataError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<MapStageData, MapStageDataError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::detect_separator(&file_content);

    // The first two lines of map stage data files are always headers/metadata.
    let lines_iterator = file_content.lines().skip(2);

    let mut entries = Vec::new();
    let mut has_content = false;

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
        if parts.len() < 2 {
            continue;
        }

        let mut energy = 0;
        if let Some(val_string) = parts.first() {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                energy = parsed_value;
            }
        }

        let mut xp = 0;
        if let Some(val_string) = parts.get(1) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                xp = parsed_value;
            }
        }

        let mut init_track = 0;
        if let Some(val_string) = parts.get(2) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                init_track = parsed_value;
            }
        }

        let mut bgm_change_percent = 0;
        if let Some(val_string) = parts.get(3) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                bgm_change_percent = parsed_value;
            }
        }

        let mut boss_track = 0;
        if let Some(val_string) = parts.get(4) {
            if let Ok(parsed_value) = val_string.trim().parse::<i16>() {
                boss_track = parsed_value;
            }
        }

        let is_time_reward = parts.len() > 15 && parts[8..15].iter().all(|part| part.trim() == "-2");

        let rewards = if is_time_reward {
            extract_timed_scores(&parts)
        } else {
            extract_treasure_drops(&parts)
        };

        entries.push(MapStageDataEntry {
            energy,
            xp,
            init_track,
            bgm_change_percent,
            boss_track,
            rewards,
        });
    }

    if !has_content {
        return Err(MapStageDataError::EmptyFile);
    }

    Ok(MapStageData { entries })
}

fn extract_timed_scores(parts: &[&str]) -> RewardStructure {
    let mut scores = Vec::new();
    let score_block_count = parts.len().saturating_sub(17) / 3;

    for block_index in 0..score_block_count {
        let mut score = 0;
        if let Some(val_string) = parts.get(16 + block_index * 3) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                score = parsed_value;
            }
        }

        let mut item_id = 0;
        if let Some(val_string) = parts.get(17 + block_index * 3) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                item_id = parsed_value;
            }
        }

        let mut amount = 0;
        if let Some(val_string) = parts.get(18 + block_index * 3) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                amount = parsed_value;
            }
        }

        scores.push(TimedScore { score, item_id, amount });
    }

    RewardStructure::Timed(scores)
}

fn extract_treasure_drops(parts: &[&str]) -> RewardStructure {
    if parts.len() < 8 {
        return RewardStructure::None;
    }

    let mut drops = Vec::new();

    let mut base_chance = 0;
    if let Some(val_string) = parts.get(5) {
        if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_chance = parsed_value;
        }
    }

    let mut base_item_id = 0;
    if let Some(val_string) = parts.get(6) {
        if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_item_id = parsed_value;
        }
    }

    let mut base_amount = 0;
    if let Some(val_string) = parts.get(7) {
        if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_amount = parsed_value;
        }
    }

    drops.push(DropReward {
        chance: base_chance,
        item_id: base_item_id,
        amount: base_amount,
    });

    let is_multi_drop = parts.len() > 9;
    let mut drop_rule = 0;

    if is_multi_drop {
        if let Some(val_string) = parts.get(8) {
            if let Ok(parsed_value) = val_string.trim().parse::<i32>() {
                drop_rule = parsed_value;
            }
        }

        let drop_block_count = parts.len().saturating_sub(7) / 3;
        for block_index in 1..drop_block_count {
            let mut block_chance = 0;
            if let Some(val_string) = parts.get(6 + block_index * 3) {
                if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_chance = parsed_value;
                }
            }

            let mut block_item_id = 0;
            if let Some(val_string) = parts.get(7 + block_index * 3) {
                if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_item_id = parsed_value;
                }
            }

            let mut block_amount = 0;
            if let Some(val_string) = parts.get(8 + block_index * 3) {
                if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_amount = parsed_value;
                }
            }

            drops.push(DropReward {
                chance: block_chance,
                item_id: block_item_id,
                amount: block_amount,
            });
        }
    }

    RewardStructure::Treasure { drop_rule, drops }
}