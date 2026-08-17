//! Per-stage cost, music, and reward metadata.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

/// Represents errors that can occur during the parsing of stage metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapStageDataError {
    /// The supplied bytes yielded no parseable rows.
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

/// One possible item reward from clearing a stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DropReward {
    /// The weighting or percentage governing how often this reward is selected.
    pub chance: u32,
    /// The identifier of the item awarded.
    pub item_id: u32,
    /// The quantity of the item awarded.
    pub amount: u32,
}

/// A reward granted for completing a stage within a score threshold.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedScore {
    /// The score that must be reached to earn this reward.
    pub score: u32,
    /// The identifier of the item awarded.
    pub item_id: u32,
    /// The quantity of the item awarded.
    pub amount: u32,
}

/// The reward scheme a stage uses on completion.
///
/// The two schemes are mutually exclusive and select on different criteria, so a
/// variant each keeps the inapplicable fields unrepresentable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RewardStructure {
    /// The stage grants no completion reward.
    #[default]
    None,
    /// The stage draws from a treasure pool.
    Treasure {
        /// The rule governing how many entries of the pool may be drawn and whether draws repeat.
        drop_rule: i32,
        /// The candidate rewards in the pool.
        drops: Vec<DropReward>,
    },
    /// The stage grants rewards according to the score achieved.
    Timed(Vec<TimedScore>),
}

/// The metadata describing a single stage's cost, music, and rewards.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageDataEntry {
    /// The energy consumed by attempting the stage.
    pub energy: u32,
    /// The experience awarded for clearing the stage.
    pub xp: u32,
    /// The identifier of the music track played from the start of the stage.
    pub init_track: u32,
    /// The base health percentage at which the music changes to the boss track.
    pub bgm_change_percent: u32,
    /// The identifier of the music track played after the change, or negative when there is none.
    pub boss_track: i16,
    /// The reward scheme the stage uses on completion.
    pub rewards: RewardStructure,
}

/// The parsed contents of a map's stage metadata file.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageData {
    /// The per-stage metadata in stage order, so an entry's position is its stage index.
    pub entries: Vec<MapStageDataEntry>,
}

impl MapStageData {
    /// Parses a map's stage metadata file into per-stage entries.
    ///
    /// The leading header rows are consumed first, after which each row is one
    /// stage in play order. The reward columns read as either a treasure pool or
    /// a score ladder according to the row's declared rule.
    ///
    /// An unreadable row yields a default entry rather than being dropped, so an
    /// entry's position is always its stage index.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the map's `MapStageData*.csv` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `MapStageData` on success, or a
    /// `MapStageDataError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, MapStageDataError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<MapStageData, MapStageDataError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::detect_separator(&file_content);

    // The first two lines of map stage data files are always headers/metadata.
    let lines_iterator = file_content.lines().skip(2);

    let mut parsed: Vec<(usize, MapStageDataEntry)> = Vec::new();
    let mut has_content = false;

    for (stage_index, file_line) in lines_iterator.enumerate() {
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
        if let Some(val_string) = parts.first()
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                energy = parsed_value;
            }

        let mut xp = 0;
        if let Some(val_string) = parts.get(1)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                xp = parsed_value;
            }

        let mut init_track = 0;
        if let Some(val_string) = parts.get(2)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                init_track = parsed_value;
            }

        let mut bgm_change_percent = 0;
        if let Some(val_string) = parts.get(3)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                bgm_change_percent = parsed_value;
            }

        let mut boss_track = 0;
        if let Some(val_string) = parts.get(4)
            && let Ok(parsed_value) = val_string.trim().parse::<i16>() {
                boss_track = parsed_value;
            }

        let is_time_reward = parts.len() > 15 && parts[8..15].iter().all(|part| part.trim() == "-2");

        let rewards = if is_time_reward {
            extract_timed_scores(&parts)
        } else {
            extract_treasure_drops(&parts)
        };

        parsed.push((stage_index, MapStageDataEntry {
            energy,
            xp,
            init_track,
            bgm_change_percent,
            boss_track,
            rewards,
        }));
    }

    if !has_content {
        return Err(MapStageDataError::EmptyFile);
    }

    let mut entries = Vec::new();
    if let Some((last_index, _)) = parsed.last() {
        entries.resize_with(last_index + 1, MapStageDataEntry::default);
        for (stage_index, entry) in parsed {
            if let Some(slot) = entries.get_mut(stage_index) {
                *slot = entry;
            }
        }
    }

    Ok(MapStageData { entries })
}

fn extract_timed_scores(parts: &[&str]) -> RewardStructure {
    let mut scores = Vec::new();
    let score_block_count = parts.len().saturating_sub(17) / 3;

    for block_index in 0..score_block_count {
        let mut score = 0;
        if let Some(val_string) = parts.get(16 + block_index * 3)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                score = parsed_value;
            }

        let mut item_id = 0;
        if let Some(val_string) = parts.get(17 + block_index * 3)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                item_id = parsed_value;
            }

        let mut amount = 0;
        if let Some(val_string) = parts.get(18 + block_index * 3)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                amount = parsed_value;
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
    if let Some(val_string) = parts.get(5)
        && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_chance = parsed_value;
        }

    let mut base_item_id = 0;
    if let Some(val_string) = parts.get(6)
        && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_item_id = parsed_value;
        }

    let mut base_amount = 0;
    if let Some(val_string) = parts.get(7)
        && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_amount = parsed_value;
        }

    drops.push(DropReward {
        chance: base_chance,
        item_id: base_item_id,
        amount: base_amount,
    });

    let is_multi_drop = parts.len() > 9;
    let mut drop_rule = 0;

    if is_multi_drop {
        if let Some(val_string) = parts.get(8)
            && let Ok(parsed_value) = val_string.trim().parse::<i32>() {
                drop_rule = parsed_value;
            }

        let drop_block_count = parts.len().saturating_sub(7) / 3;
        for block_index in 1..drop_block_count {
            let mut block_chance = 0;
            if let Some(val_string) = parts.get(6 + block_index * 3)
                && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_chance = parsed_value;
                }

            let mut block_item_id = 0;
            if let Some(val_string) = parts.get(7 + block_index * 3)
                && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_item_id = parsed_value;
                }

            let mut block_amount = 0;
            if let Some(val_string) = parts.get(8 + block_index * 3)
                && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_amount = parsed_value;
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
#[cfg(test)]
mod tests {
    use super::*;

    const HEADERS: &str = "0,0,0\n0,0,0\n";

    #[test]
    fn blank_rows_do_not_shift_stage_indices() {
        let raw = format!("{HEADERS}10,1000,1,0,-1\n\n30,3000,1,0,-1\n");
        let data = MapStageData::parse(raw).unwrap();

        assert_eq!(data.entries.len(), 3);
        assert_eq!(data.entries[0].energy, 10);
        assert_eq!(data.entries[0].xp, 1000);
        assert_eq!(data.entries[1], MapStageDataEntry::default());
        assert_eq!(data.entries[2].energy, 30);
        assert_eq!(data.entries[2].xp, 3000);
    }

    #[test]
    fn trailing_blank_rows_are_not_padded() {
        let raw = format!("{HEADERS}10,1000,1,0,-1\n20,2000,1,0,-1\n\n\n");
        let data = MapStageData::parse(raw).unwrap();

        assert_eq!(data.entries.len(), 2);
        assert_eq!(data.entries[1].energy, 20);
    }

    #[test]
    fn dense_files_are_unaffected() {
        let raw = format!("{HEADERS}10,1000,1,0,-1\n20,2000,1,0,-1\n30,3000,1,0,-1\n");
        let data = MapStageData::parse(raw).unwrap();

        assert_eq!(data.entries.len(), 3);
        let energies: Vec<u32> = data.entries.iter().map(|entry| entry.energy).collect();
        assert_eq!(energies, vec![10, 20, 30]);
    }

    #[test]
    fn a_file_with_no_data_rows_is_an_error() {
        assert_eq!(MapStageData::parse(HEADERS).unwrap_err(), MapStageDataError::EmptyFile);
    }
}
