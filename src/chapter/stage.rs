mod battleground;
mod certification_preset;
mod charagroup;
mod difficulty_level;
mod drop_chara;
mod fixed_formation;
mod mapstagedata;
mod scatcpusetting;
mod stage_option;
mod stagename;
mod xp;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use battleground::{Battleground, BattlegroundEntry, BattlegroundError, BossType, EnemyAmount};
pub use certification_preset::{AbilityType, CannonType, CertificationPreset, CertificationPresetError, EvolutionForm, PresetAbility, PresetChara, PresetTreasure, TreasureType};
pub use charagroup::{CharaGroup, CharaGroupEntry, CharaGroupError, CharaGroupType};
pub use difficulty_level::{DifficultyLevel, DifficultyLevelError};
pub use drop_chara::{DropChara, DropCharaError};
pub use fixed_formation::{FixedFormation, FixedFormationEntry, FixedFormationError};
pub use mapstagedata::{DropReward, MapStageData, MapStageDataEntry, MapStageDataError, RewardStructure, TimedScore};
pub use scatcpusetting::{ScatCpuSetting, ScatCpuSettingError};
pub use stage_option::{StageOption, StageOptionEntry, StageOptionError};
pub use stagename::{StageName, StageNameEntry, StageNameError};
pub use xp::get_hardcoded_xp;

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