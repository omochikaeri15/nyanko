use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum SpecialRulesMapError {
    InvalidJson,
}

impl fmt::Display for SpecialRulesMapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => write!(
                formatter,
                "The provided byte slice could not be parsed as valid JSON."
            ),
        }
    }
}

impl std::error::Error for SpecialRulesMapError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    TrustFund(Vec<u32>),
    CooldownEquality(Vec<u32>),
    RarityLimit(Vec<u32>),
    CheapLabor(Vec<u32>),
    CatCost(Vec<u32>),
    CatProduction(Vec<u32>),
    TotalDeployLimit(Vec<u32>),
    MoreThanOne(Vec<u32>),
    MegaCatCannon(Vec<u32>),
    UniformMotion(Vec<u32>),
    Unknown(u8, Vec<u32>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMapEntry {
    pub contents_type: u8,
    pub rules: Vec<RuleType>,
    pub name_label: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecialRulesMap {
    pub entries: HashMap<u32, SpecialRulesMapEntry>,
}

impl SpecialRulesMap {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, SpecialRulesMapError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<SpecialRulesMap, SpecialRulesMapError> {
    let json_value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| SpecialRulesMapError::InvalidJson)?;

    let mut entries = HashMap::new();

    if let Some(map_id_object) = json_value.get("MapID").and_then(|v| v.as_object()) {
        for (map_id_str, map_data) in map_id_object {
            let Ok(map_id) = map_id_str.parse::<u32>() else { continue; };

            let contents_type = map_data
                .get("ContentsType")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8;

            let name_label = map_data
                .get("RuleNameLabel")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let mut rules = Vec::new();
            if let Some(rule_type_object) = map_data.get("RuleType").and_then(|v| v.as_object()) {
                for (rule_id_str, rule_data) in rule_type_object {
                    let Ok(rule_id) = rule_id_str.parse::<u8>() else { continue; };

                    let rule_values: Vec<u32> = rule_data
                        .get("Parameters")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter().filter_map(|v| v.as_u64().map(|n| n as u32)).collect()
                        })
                        .unwrap_or_default();

                    let rule_enum = match rule_id {
                        0 => RuleType::TrustFund(rule_values),
                        1 => RuleType::CooldownEquality(rule_values),
                        3 => RuleType::RarityLimit(rule_values),
                        4 => RuleType::CheapLabor(rule_values),
                        5 => RuleType::CatCost(rule_values),
                        6 => RuleType::CatProduction(rule_values),
                        7 => RuleType::TotalDeployLimit(rule_values),
                        8 => RuleType::MoreThanOne(rule_values),
                        9 => RuleType::MegaCatCannon(rule_values),
                        10 => RuleType::UniformMotion(rule_values),
                        _ => RuleType::Unknown(rule_id, rule_values),
                    };

                    rules.push(rule_enum);
                }
            }

            entries.insert(
                map_id,
                SpecialRulesMapEntry {
                    contents_type,
                    rules,
                    name_label,
                },
            );
        }
    }

    Ok(SpecialRulesMap { entries })
}