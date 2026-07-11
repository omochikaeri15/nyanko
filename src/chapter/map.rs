use serde::{Deserialize, Serialize};

pub use super::data::dropitem::{DropItem, DropItemEntry, DropItemError};
pub use super::data::ex_option::{ExOption, ExOptionError};
pub use super::data::lockskipdata::{LockSkipData, LockSkipDataEntry, LockSkipDataError};
pub use super::data::map_name::{MapName, MapNameError};
pub use super::data::map_option::{MapOption, MapOptionEntry, MapOptionError, ResetType};
pub use super::data::scorebonusmap::{ScoreBonusMap, ScoreBonusMapEntry, ScoreBonusMapError, BonusType};
pub use super::data::specialrulesmap::{SpecialRulesMap, SpecialRulesMapEntry, SpecialRulesMapError, RuleType};
pub use super::data::specialrulesmapoption::{SpecialRulesMapOption, SpecialRulesMapOptionEntry, SpecialRulesMapOptionError};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Map {
    pub id: String,
    pub name: String,
    pub category: String,
    pub category_name: String,
    pub map_id: u32,
    pub stages: Vec<String>,
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
    pub ex_invasion: Option<u32>,
    pub score_bonuses: Option<ScoreBonusMapEntry>,
    pub special_rules: Option<SpecialRulesMapEntry>,
    pub invalid_combos: Vec<u32>,
    pub drop_items: Option<DropItem>,
}