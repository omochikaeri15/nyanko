use std::collections::HashMap;
use std::fmt;

use crate::common::tools::file;

/// Represents errors that can occur during the parsing of unit progression data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnitBuyError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for UnitBuyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid unit buy data."),
        }
    }
}

impl std::error::Error for UnitBuyError {}

/// A unit's economic and progression parameters.
///
/// Covers rarity, purchase and upgrade costs, level caps, unlock conditions, and
/// evolution material requirements.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitBuy {
    /// The identifier of the stage that must be cleared before the unit becomes purchasable.
    pub stage_unlock_requirement: i32,
    /// The currency cost of the initial purchase.
    pub purchase_cost: i32,
    /// The identifier of the currency the purchase and upgrade costs are denominated in.
    pub currency_type: i32,
    /// The rarity tier of the unit, ordering it within the roster.
    pub rarity: i32,
    /// The position of the unit within the in-game unit guide listing.
    pub guide_order: i32,
    /// The identifier of the chapter that must be cleared before the unit becomes purchasable.
    pub chapter_unlock_requirement: i32,
    /// The experience refunded when the unit is sold.
    pub sell_xp_yield: i32,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_17: i32,
    /// The maximum level attainable after clearing the second chapter.
    pub level_cap_ch2: i32,
    /// The maximum plus level attainable before any cap extension is applied.
    pub base_max_plus_level: i32,
    /// The experience required to perform the standard evolution.
    pub evolve_level_xp: i32,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_21: i32,
    /// The maximum level attainable after clearing the first chapter.
    pub level_cap_ch1: i32,
    /// The identifier of the unit's True form, or negative one when the unit has none.
    pub true_form_id: i32,
    /// The identifier of the unit's Ultra form, or negative one when the unit has none.
    pub ultra_form_id: i32,
    /// The level the unit must reach before the True form evolution becomes available.
    pub true_form_unlock_level: i32,
    /// The level the unit must reach before the Ultra form evolution becomes available.
    pub ultra_form_unlock_level: i32,
    /// The experience consumed by the True form evolution.
    pub true_form_xp_cost: i32,
    /// The experience consumed by the Ultra form evolution.
    pub ultra_form_xp_cost: i32,
    /// The maximum level attainable through ordinary experience upgrades.
    pub level_cap_standard: i32,
    /// The maximum level attainable once Catseye items are applied.
    pub level_cap_catseye: i32,
    /// The maximum plus level attainable once the cap is fully extended.
    pub level_cap_plus: i32,
    /// The vertical sprite offset applied to the normal form on the evolution screen.
    pub normal_evolution_y_offset: i32,
    /// The vertical sprite offset applied to the evolved form on the evolution screen.
    pub evolved_evolution_y_offset: i32,
    /// The vertical sprite offset applied to the True form on the evolution screen.
    pub true_evolution_y_offset: i32,
    /// The vertical sprite offset applied to the Ultra form on the evolution screen.
    pub ultra_evolution_y_offset: i32,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_56: i32,
    /// The game version in which the unit was introduced, encoded as a packed integer.
    pub version_added: i64,
    /// The Neko Points refunded when the unit is sold.
    pub sell_np_yield: i32,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_59: i32,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_60: i32,
    /// The raw egg identifier for the unit's base form, or negative one when it is not an egg unit.
    /// Prefer [`UnitBuy::egg_ids`] over reading this sentinel directly.
    pub egg_id_normal: i32,
    /// The raw egg identifier for the unit's evolved form, or negative one when it is not an egg unit.
    /// Prefer [`UnitBuy::egg_ids`] over reading this sentinel directly.
    pub egg_id_evolved: i32,
    /// Any trailing columns beyond the known layout, retained verbatim for forward compatibility.
    pub rest: Vec<i32>,
    /// The experience cost of each of the ten ordinary level upgrades.
    pub upgrade_costs: Vec<i32>,
    /// The item identifier and quantity pairs consumed by the True form evolution.
    pub true_form_materials: Vec<(i32, i32)>,
    /// The item identifier and quantity pairs consumed by the Ultra form evolution.
    pub ultra_form_materials: Vec<(i32, i32)>,
}

impl Default for UnitBuy {
    /// Produces a row in which every reference to another entity is absent.
    ///
    /// The four identifier fields hold negative one rather than zero, matching
    /// the sentinel the raw columns use.
    fn default() -> Self {
        Self {
            true_form_id: -1,
            ultra_form_id: -1,
            egg_id_normal: -1,
            egg_id_evolved: -1,
            stage_unlock_requirement: 0,
            purchase_cost: 0,
            currency_type: 0,
            rarity: 0,
            guide_order: 0,
            chapter_unlock_requirement: 0,
            sell_xp_yield: 0,
            unknown_17: 0,
            level_cap_ch2: 0,
            base_max_plus_level: 0,
            evolve_level_xp: 0,
            unknown_21: 0,
            level_cap_ch1: 0,
            true_form_unlock_level: 0,
            ultra_form_unlock_level: 0,
            true_form_xp_cost: 0,
            ultra_form_xp_cost: 0,
            level_cap_standard: 0,
            level_cap_catseye: 0,
            level_cap_plus: 0,
            normal_evolution_y_offset: 0,
            evolved_evolution_y_offset: 0,
            true_evolution_y_offset: 0,
            ultra_evolution_y_offset: 0,
            unknown_56: 0,
            version_added: 0,
            sell_np_yield: 0,
            unknown_59: 0,
            unknown_60: 0,
            rest: Vec::new(),
            upgrade_costs: Vec::new(),
            true_form_materials: Vec::new(),
            ultra_form_materials: Vec::new(),
        }
    }
}

impl UnitBuy {
    /// Decodes the unit's egg counterparts into a pair of optional identifiers.
    ///
    /// The raw columns use negative one per element to mean the unit has no egg
    /// counterpart for that form. Each column resolves independently.
    ///
    /// # Returns
    /// A tuple whose first element is the base form's egg identifier and whose
    /// second is the evolved form's, each `None` when the corresponding raw
    /// column holds the sentinel or a value outside the representable range.
    pub fn egg_ids(&self) -> (Option<u16>, Option<u16>) {
        (
            u16::try_from(self.egg_id_normal).ok(),
            u16::try_from(self.egg_id_evolved).ok(),
        )
    }

    fn from_csv_line(csv_line: &str, delimiter: char) -> Self {
        let parts: Vec<&str> = csv_line.split(delimiter).map(|s| s.trim()).collect();

        let get_integer = |idx: usize| -> i32 {
            parts.get(idx).and_then(|s| s.parse::<i32>().ok()).unwrap_or(-1)
        };

        let get_long = |idx: usize| -> i64 {
            parts.get(idx).and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1)
        };

        let parse_materials = |start_idx: usize| -> Vec<(i32, i32)> {
            let mut material_list = Vec::new();
            for i in 0..5 {
                let base_idx = start_idx + (i * 2);
                let item_id = get_integer(base_idx);
                let item_cost = get_integer(base_idx + 1);
                if item_id != -1 && item_cost > 0 {
                    material_list.push((item_id, item_cost));
                }
            }
            material_list
        };

        let parse_upgrades = |start_idx: usize| -> Vec<i32> {
            (0..10).map(|i| get_integer(start_idx + i)).collect()
        };

        let rest_vector: Vec<i32> = parts
            .iter()
            .skip(63)
            .filter_map(|part| part.parse::<i32>().ok())
            .collect();

        Self {
            stage_unlock_requirement: get_integer(0),
            purchase_cost: get_integer(1),
            upgrade_costs: parse_upgrades(2),
            currency_type: get_integer(12),
            rarity: get_integer(13),
            guide_order: get_integer(14),
            chapter_unlock_requirement: get_integer(15),
            sell_xp_yield: get_integer(16),
            unknown_17: get_integer(17),
            level_cap_ch2: get_integer(18),
            base_max_plus_level: get_integer(19),
            evolve_level_xp: get_integer(20),
            unknown_21: get_integer(21),
            level_cap_ch1: get_integer(22),
            true_form_id: get_integer(23),
            ultra_form_id: get_integer(24),
            true_form_unlock_level: get_integer(25),
            ultra_form_unlock_level: get_integer(26),
            true_form_xp_cost: get_integer(27),
            true_form_materials: parse_materials(28),
            ultra_form_xp_cost: get_integer(38),
            ultra_form_materials: parse_materials(39),
            level_cap_standard: get_integer(49),
            level_cap_catseye: get_integer(50),
            level_cap_plus: get_integer(51),
            normal_evolution_y_offset: get_integer(52),
            evolved_evolution_y_offset: get_integer(53),
            true_evolution_y_offset: get_integer(54),
            ultra_evolution_y_offset: get_integer(55),
            unknown_56: get_integer(56),
            version_added: get_long(57),
            sell_np_yield: get_integer(58),
            unknown_59: get_integer(59),
            unknown_60: get_integer(60),
            egg_id_normal: get_integer(61),
            egg_id_evolved: get_integer(62),
            rest: rest_vector,
        }
    }

    /// Parses the unit progression table into rows keyed by unit identifier.
    ///
    /// A line's position in the file is that unit's identifier, which blank
    /// lines do not disturb.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `unitbuy.csv` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed rows keyed by unit identifier on
    /// success, or a `UnitBuyError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<HashMap<u32, Self>, UnitBuyError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<HashMap<u32, UnitBuy>, UnitBuyError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::detect_separator(&file_content);

    let mut map = HashMap::new();

    for (line_index, line) in file_content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        map.insert(line_index as u32, UnitBuy::from_csv_line(line, delimiter));
    }

    if map.is_empty() {
        return Err(UnitBuyError::EmptyFile);
    }

    Ok(map)
}