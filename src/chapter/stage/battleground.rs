//! Stage layout and enemy spawn configuration.
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

/// Represents errors that can occur during the parsing of a stage layout.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BattlegroundError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
    /// The supplied bytes carried no leading row declaring the battlefield itself.
    MissingConfigLine,
}

impl fmt::Display for BattlegroundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid battleground data."
            ),
            Self::MissingConfigLine => write!(
                formatter,
                "The file has a header but is missing the required config line."
            ),
        }
    }
}

impl std::error::Error for BattlegroundError {}

/// Selects the presentation treatment applied to an enemy on spawn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BossType {
    /// An ordinary enemy, spawned without announcement.
    #[default]
    None,
    /// A boss, announced with the boss entry effect.
    Boss,
    /// A boss whose entry additionally shakes the screen.
    ScreenShake,
    /// A treatment code this parser does not recognize, carrying its raw value.
    Unknown(u32),
}

impl From<u32> for BossType {
    fn from(boss_val: u32) -> Self {
        match boss_val {
            0 => Self::None,
            1 => Self::Boss,
            2 => Self::ScreenShake,
            _ => Self::Unknown(boss_val),
        }
    }
}

/// The number of times an enemy may spawn over a stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum EnemyAmount {
    /// The enemy respawns for as long as the stage runs.
    #[default]
    Infinite,
    /// The enemy spawns at most the given number of times.
    Limit(u32),
}

/// One enemy's spawn configuration within a stage.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattlegroundEntry {
    /// The identifier of the enemy this row spawns.
    pub enemy_id: u32,
    /// How many times the enemy may spawn.
    pub amount: EnemyAmount,
    /// The delay in frames before the enemy first becomes eligible to spawn.
    pub start_frame: u32,
    /// The shortest delay in frames between successive spawns.
    pub respawn_min: u32,
    /// The longest delay in frames between successive spawns.
    pub respawn_max: u32,
    /// The base health percentage below which the enemy begins spawning.
    pub base_hp_perc: u32,
    /// The lower bound of the random draw layer used to order overlapping sprites.
    pub layer_min: i32,
    /// The upper bound of the random draw layer used to order overlapping sprites.
    pub layer_max: i32,
    /// The presentation treatment applied when the enemy spawns.
    pub boss_type: BossType,
    /// The health scaling percentage applied to the enemy.
    pub magnification: u32,
    /// The score awarded for defeating the enemy.
    pub score: u32,
    /// The attack scaling percentage applied to the enemy, separate from its health scaling.
    pub atk_magnification: u32,
    /// The flag selecting whether `start_frame` is measured from the stage start or from the base being struck.
    pub time_flag: u32,
    /// The number of enemy defeats required before this row begins spawning.
    pub kill_count: u32,
    /// Whether this row describes the enemy base rather than a spawning enemy.
    pub is_base: bool,
}

/// The complete layout and enemy roster of a single stage.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Battleground {
    /// The identifier of the enemy base defending the stage.
    pub base_id: i32,
    /// The horizontal extent of the battlefield in engine distance units.
    pub width: u32,
    /// The health pool of the enemy base.
    pub base_hp: u32,
    /// The shortest delay in frames between enemy spawn checks.
    pub min_spawn: u32,
    /// The longest delay in frames between enemy spawn checks.
    pub max_spawn: u32,
    /// The identifier of the background artwork the stage is drawn against.
    pub background_id: u32,
    /// The greatest number of enemies permitted on the field at once.
    pub max_enemies: u32,
    /// The identifier of the animation used for the enemy base.
    pub anim_base_id: u32,
    /// The stage's time limit in frames, or zero when it is untimed.
    pub time_limit: u32,
    /// Whether the player is barred from continuing after a loss.
    pub is_no_continues: bool,
    /// Whether the enemy base cannot be destroyed by ordinary means.
    pub is_base_indestructible: bool,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_value: u32,
    /// The enemy spawn configurations that make up the stage.
    pub entries: Vec<BattlegroundEntry>,
}

impl Battleground {
    /// Parses a stage layout file into its header and enemy spawn roster.
    ///
    /// The first row declares the battlefield itself and each subsequent row
    /// declares one enemy's spawn configuration. Column layouts vary between
    /// game versions, so absent trailing columns fall back to their engine
    /// defaults rather than rejecting the row.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the stage's `stage*.csv` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `Battleground` on success, or a
    /// `BattlegroundError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, BattlegroundError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<Battleground, BattlegroundError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::detect_separator(&file_content);

    let mut clean_lines_iterator = file_content.lines().filter_map(|line| {
        let clean = line.split_once("//").map(|(before, _)| before).unwrap_or(line).trim();
        if clean.is_empty() { None } else { Some(clean) }
    });

    let Some(first_line) = clean_lines_iterator.next() else {
        return Err(BattlegroundError::EmptyFile);
    };

    let first_line_parts: Vec<&str> = first_line.split(separator_char).collect();

    let mut has_header = false;
    if first_line_parts.len() <= 7 {
        has_header = true;
    } else if let Some(part) = first_line_parts.get(6)
        && part.trim().is_empty() {
            has_header = true;
        }

    let mut base_id = 0;
    let mut is_no_continues = false;

    let config_line = if !has_header {
        first_line
    } else {
        if let Some(val_string) = first_line_parts.first()
            && let Ok(parsed) = val_string.trim().parse::<i32>() {
                base_id = parsed;
            }
        if let Some(val_string) = first_line_parts.get(1) {
            is_no_continues = val_string.trim() == "1";
        }

        let Some(next_line) = clean_lines_iterator.next() else {
            return Err(BattlegroundError::MissingConfigLine);
        };
        next_line
    };

    let config_parts: Vec<&str> = config_line.split(separator_char).collect();

    let mut width = 0;
    if let Some(val) = config_parts.first()
        && let Ok(p) = val.trim().parse::<u32>() { width = p; }

    let mut base_hp = 0;
    if let Some(val) = config_parts.get(1)
        && let Ok(p) = val.trim().parse::<u32>() { base_hp = p; }

    let mut min_spawn = 0;
    if let Some(val) = config_parts.get(2)
        && let Ok(p) = val.trim().parse::<u32>() { min_spawn = p; }

    let mut max_spawn = min_spawn;
    if let Some(val) = config_parts.get(3)
        && let Ok(p) = val.trim().parse::<u32>() { max_spawn = p; }

    let mut background_id = 0;
    if let Some(val) = config_parts.get(4)
        && let Ok(p) = val.trim().parse::<u32>() { background_id = p; }

    let mut max_enemies = 0;
    if let Some(val) = config_parts.get(5)
        && let Ok(p) = val.trim().parse::<u32>() { max_enemies = p; }

    let mut anim_base_id = 0;
    if let Some(val) = config_parts.get(6)
        && let Ok(p) = val.trim().parse::<u32>() { anim_base_id = p; }

    let mut time_limit = 0;
    if let Some(val) = config_parts.get(7)
        && let Ok(p) = val.trim().parse::<u32>() { time_limit = p; }

    let mut is_base_indestructible = false;
    if let Some(val) = config_parts.get(8)
        && let Ok(p) = val.trim().parse::<u8>() {
            is_base_indestructible = p == 1;
        }

    let mut unknown_value = 0;
    if let Some(val) = config_parts.get(9)
        && let Ok(p) = val.trim().parse::<u32>() { unknown_value = p; }

    let mut entries = Vec::new();

    for enemy_line in clean_lines_iterator {
        let enemy_parts: Vec<&str> = enemy_line.split(separator_char).collect();

        let mut raw_enemy_id = 0;
        if let Some(val) = enemy_parts.first()
            && let Ok(p) = val.trim().parse::<u32>() { raw_enemy_id = p; }

        if raw_enemy_id == 0 {
            break;
        }

        let mut raw_amount = 0;
        if let Some(val) = enemy_parts.get(1)
            && let Ok(p) = val.trim().parse::<u32>() { raw_amount = p; }

        let mut start_frame = 0;
        if let Some(val) = enemy_parts.get(2)
            && let Ok(p) = val.trim().parse::<u32>() { start_frame = p * 2; }

        let mut respawn_min = 0;
        if let Some(val) = enemy_parts.get(3)
            && let Ok(p) = val.trim().parse::<u32>() { respawn_min = p * 2; }

        let mut respawn_max = 0;
        if let Some(val) = enemy_parts.get(4)
            && let Ok(p) = val.trim().parse::<u32>() { respawn_max = p * 2; }

        let mut amount = if raw_amount == 0 { EnemyAmount::Infinite } else { EnemyAmount::Limit(raw_amount) };
        if respawn_min == 0 {
            amount = EnemyAmount::Infinite;
        }

        let mut base_hp_perc = 0;
        if let Some(val) = enemy_parts.get(5)
            && let Ok(p) = val.trim().parse::<u32>() { base_hp_perc = p; }

        let mut layer_min = 0;
        if let Some(val) = enemy_parts.get(6)
            && let Ok(p) = val.trim().parse::<i32>() { layer_min = p; }

        let mut layer_max = 0;
        if let Some(val) = enemy_parts.get(7)
            && let Ok(p) = val.trim().parse::<i32>() { layer_max = p; }

        let mut boss_type_val = 0;
        if let Some(val) = enemy_parts.get(8)
            && let Ok(p) = val.trim().parse::<u32>() { boss_type_val = p; }

        let mut mag_percent = 100;
        if let Some(val) = enemy_parts.get(9) {
            let trimmed = val.trim();
            if trimmed != "."
                && let Ok(p) = trimmed.parse::<u32>() { mag_percent = p; }
        }

        let mut score = 0;
        if let Some(val) = enemy_parts.get(10)
            && let Ok(p) = val.trim().parse::<u32>() { score = p; }

        let mut atk_magnification = mag_percent;
        if let Some(val) = enemy_parts.get(11)
            && let Ok(p) = val.trim().parse::<u32>()
                && p != 0 {
                    atk_magnification = p;
                }

        let mut time_flag = 0;
        if let Some(val) = enemy_parts.get(12)
            && let Ok(p) = val.trim().parse::<u32>() { time_flag = p; }

        let mut kill_count = 0;
        if let Some(val) = enemy_parts.get(13)
            && let Ok(p) = val.trim().parse::<u32>() { kill_count = p; }

        let actual_enemy_id = raw_enemy_id.saturating_sub(2);

        if actual_enemy_id == 21 && start_frame == 27000 {
            continue;
        }

        let is_base = raw_enemy_id != 0 && raw_enemy_id == anim_base_id;

        entries.push(BattlegroundEntry {
            enemy_id: actual_enemy_id,
            amount,
            start_frame,
            respawn_min,
            respawn_max,
            base_hp_perc,
            layer_min,
            layer_max,
            boss_type: BossType::from(boss_type_val),
            magnification: mag_percent,
            score,
            atk_magnification,
            time_flag,
            kill_count,
            is_base,
        });
    }

    Ok(Battleground {
        base_id,
        width,
        base_hp,
        min_spawn,
        max_spawn,
        background_id,
        max_enemies,
        anim_base_id,
        time_limit,
        is_no_continues,
        is_base_indestructible,
        unknown_value,
        entries,
    })
}