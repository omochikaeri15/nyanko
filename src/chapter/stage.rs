use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use super::data::battleground::{Battleground, BattlegroundEntry, BattlegroundError, BossType, EnemyAmount};
pub use super::data::drop_chara::{DropChara, DropCharaError};
pub use super::data::charagroup::{CharaGroup, CharaGroupEntry, CharaGroupError, CharaGroupType};
pub use super::data::certification_preset::{CertificationPreset, CertificationPresetError, PresetChara, PresetAbility, PresetTreasure, EvolutionForm, CannonType, AbilityType, TreasureType};
pub use super::data::difficulty_level::{DifficultyLevel, DifficultyLevelError};
pub use super::data::fixed_formation::{FixedFormation, FixedFormationEntry, FixedFormationError};
pub use super::data::mapstagedata::{DropReward, MapStageData, MapStageDataError, RewardStructure, TimedScore};
pub use super::data::scatcpusetting::{SCatCpuSetting, SCatCpuSettingError};
pub use super::data::stage_option::{StageOption, StageOptionEntry, StageOptionError};
pub use super::data::stagename::{StageName, StageNameEntry, StageNameError};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stage {
    pub id: String,
    pub name: String,
    pub category: String,
    pub category_name: String,
    pub map_id: u32,
    pub stage_id: u32,
    pub base_id: i32,
    pub anim_base_id: u32,
    pub width: u32,
    pub base_hp: u32,
    pub min_spawn: u32,
    pub max_spawn: u32,
    pub background_id: u32,
    pub max_enemies: u32,
    pub time_limit: u32,
    pub is_no_continues: bool,
    pub is_base_indestructible: bool,
    pub unknown_value: u32,
    pub enemies: Vec<BattlegroundEntry>,
    pub energy: u32,
    pub xp: u32,
    pub init_track: u32,
    pub bgm_change_percent: u32,
    pub boss_track: i16,
    pub rewards: RewardStructure,
    pub difficulty: u16,
    pub max_crowns: u8,
    pub target_crowns: i8,
    pub rarity_mask: u8,
    pub deploy_limit: u32,
    pub allowed_rows: u8,
    pub min_cost: u32,
    pub max_cost: u32,
    pub charagroup: Option<CharaGroup>,
    pub fixed_lineups: HashMap<u8, CertificationPreset>,
}