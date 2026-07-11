use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ScoreBonusMapError {
    InvalidJson,
}

impl fmt::Display for ScoreBonusMapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => write!(
                formatter,
                "The provided byte slice could not be parsed as valid JSON."
            ),
        }
    }
}

impl std::error::Error for ScoreBonusMapError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusType {
    Weaken(Vec<u32>),
    Freeze(Vec<u32>),
    Slow(Vec<u32>),
    Knockback(Vec<u32>),
    StrongAttack(Vec<u32>),
    MassiveDamage(Vec<u32>),
    StrongDefense(Vec<u32>),
    Resist(Vec<u32>),
    Unknown(u8, Vec<u32>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreBonusMapEntry {
    pub bonuses: Vec<BonusType>,
    pub name_label: String,
    pub explanation_label: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreBonusMap {
    pub entries: HashMap<u32, ScoreBonusMapEntry>,
}

// --- Private Deserialization Helpers ---

#[derive(Deserialize)]
struct RawBonusType {
    #[serde(rename = "Parameters")]
    parameters: Vec<u32>,
}

#[derive(Deserialize)]
struct RawBonusData {
    #[serde(rename = "BonusType", default)]
    bonus_type: HashMap<String, RawBonusType>,
    #[serde(rename = "BonusNameLabel")]
    bonus_name_label: Option<String>,
    #[serde(rename = "BonusExplanationLabel")]
    bonus_explanation_label: Option<String>,
}

#[derive(Deserialize)]
struct RawBonusesMap {
    #[serde(rename = "MapID", default)]
    map_id: HashMap<String, RawBonusData>,
}

// --- Parsing Logic ---

impl ScoreBonusMap {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, ScoreBonusMapError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<ScoreBonusMap, ScoreBonusMapError> {
    let Ok(json_data) = serde_json::from_slice::<RawBonusesMap>(bytes) else {
        return Err(ScoreBonusMapError::InvalidJson);
    };

    let mut entries = HashMap::new();

    for (map_id_str, raw_data) in json_data.map_id {
        let Ok(map_id) = map_id_str.parse::<u32>() else { continue; };

        let mut bonuses = Vec::new();
        for (key_str, raw_type) in raw_data.bonus_type {
            let Ok(key) = key_str.parse::<u8>() else { continue; };
            let effect_values = raw_type.parameters;

            let bonus_enum = match key {
                0 => BonusType::Weaken(effect_values),
                1 => BonusType::Freeze(effect_values),
                2 => BonusType::Slow(effect_values),
                3 => BonusType::Knockback(effect_values),
                13 => BonusType::StrongAttack(effect_values),
                14 => BonusType::MassiveDamage(effect_values),
                16 => BonusType::StrongDefense(effect_values),
                17 => BonusType::Resist(effect_values),
                _ => BonusType::Unknown(key, effect_values),
            };

            bonuses.push(bonus_enum);
        }

        entries.insert(
            map_id,
            ScoreBonusMapEntry {
                bonuses,
                name_label: raw_data.bonus_name_label.unwrap_or_default(),
                explanation_label: raw_data.bonus_explanation_label.unwrap_or_default(),
            },
        );
    }

    Ok(ScoreBonusMap { entries })
}