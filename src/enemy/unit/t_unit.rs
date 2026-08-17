//! Parsing of the shared enemy combat statistic table.
//!
//! The engine stores every enemy's statistics in a single `t_unit.csv` table
//! carrying two header lines, with one row per enemy in identifier order.

use std::cell::Cell;

use crate::combat::{Entity, EntityError, Faction, entity};

const HEADER_LINES: usize = 2;

/// Parses the shared enemy statistic table into one entity per declared enemy.
///
/// The table carries two leading header lines, which are skipped before the
/// remaining rows are read in file order. The position of an entity within the
/// returned vector is its internal enemy identifier.
///
/// # Arguments
/// * `bytes` - The raw, decrypted byte slice of the `t_unit.csv` file.
///
/// # Returns
/// A `Result` containing the parsed entities indexed by enemy identifier on
/// success, or an `EntityError` if the file contained no rows wide enough to
/// be interpreted as combat data.
pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Vec<Entity>, EntityError> {
    entity::parse_rows(bytes.as_ref(), HEADER_LINES, from_row)
}

/// Parses a single row of the shared enemy statistic table by enemy identifier.
///
/// This avoids materializing the entire table when only one enemy is required.
/// The two leading header lines are skipped before the identifier is applied as
/// a row offset.
///
/// # Arguments
/// * `bytes` - The raw, decrypted byte slice of the `t_unit.csv` file.
/// * `id` - The internal enemy identifier, used as a zero-based row offset past the header.
///
/// # Returns
/// An `Option` containing the parsed entity, or `None` if the identifier lies
/// beyond the end of the table or addresses a row too narrow to be combat data.
pub fn parse_row<B: AsRef<[u8]>>(bytes: B, id: usize) -> Option<Entity> {
    entity::parse_single(bytes.as_ref(), HEADER_LINES, id, from_row)
}

fn from_row(cols: &[&str]) -> Entity {
    let max_read = Cell::new(0);
    let cell = entity::reader(cols, &max_read);
    let mut unit = Entity {
        faction: Faction::Enemy,
        hitpoints: cell(0, 0),
        knockbacks: cell(1, 0),
        speed: cell(2, 0),
        attack_1: cell(3, 0),
        attack_cooldown: cell(4, 0) * 2,
        standing_range: cell(5, 0),
        cash_drop: cell(6, 0),
        hitbox_position: cell(7, 0),
        hitbox_width: cell(8, 0),
        unused: cell(9, 0),
        trait_red: cell(10, 0),
        area_attack: cell(11, 0),
        time_until_attack_1: cell(12, 0),
        trait_floating: cell(13, 0),
        trait_dark: cell(14, 0),
        trait_metal: cell(15, 0),
        trait_traitless: cell(16, 0),
        trait_angel: cell(17, 0),
        trait_alien: cell(18, 0),
        trait_zombie: cell(19, 0),
        knockback_chance: cell(20, 0),
        freeze_chance: cell(21, 0),
        freeze_duration: cell(22, 0),
        slow_chance: cell(23, 0),
        slow_duration: cell(24, 0),
        critical_chance: cell(25, 0),
        base_destroyer: cell(26, 0),
        wave_chance: cell(27, 0),
        wave_level: cell(28, 0),
        weaken_chance: cell(29, 0),
        weaken_duration: cell(30, 0),
        weaken_to: cell(31, 0),
        strengthen_threshold: cell(32, 0),
        strengthen_boost: cell(33, 0),
        survive: cell(34, 0),
        long_distance_1_anchor: cell(35, 0),
        long_distance_1_span: cell(36, 0),
        wave_immune: cell(37, 0),
        wave_block: cell(38, 0),
        knockback_immune: cell(39, 0),
        freeze_immune: cell(40, 0),
        slow_immune: cell(41, 0),
        weaken_immune: cell(42, 0),
        burrow_amount: cell(43, 0),
        burrow_distance: cell(44, 0) / 4,
        revive_count: cell(45, 0),
        revive_time: cell(46, 0),
        revive_hp: cell(47, 0),
        trait_witch: cell(48, 0),
        trait_dojo: cell(49, 0),
        attack_count_total: cell(50, -1),
        time_before_death: cell(51, -1),
        attack_count_state: cell(52, 0),
        spawn_animation_type: cell(53, 0),
        soul_animation_type: cell(54, 0),
        attack_2: cell(55, 0),
        attack_3: cell(56, 0),
        time_until_attack_2: cell(57, 0),
        time_until_attack_3: cell(58, 0),
        attack_1_abilities: cell(59, 0),
        attack_2_abilities: cell(60, 0),
        attack_3_abilities: cell(61, 0),
        spawn_animation_flag: cell(62, 0),
        soul_animation_flag: cell(63, 0),
        barrier_hitpoints: cell(64, 0),
        warp_chance: cell(65, 0),
        warp_duration: cell(66, 0),
        warp_distance_minimum: cell(67, 0) / 4,
        warp_distance_maximum: cell(68, 0) / 4,
        trait_starred_alien: cell(69, 0),
        warp_immune: cell(70, 0),
        trait_eva: cell(71, 0),
        trait_relic: cell(72, 0),
        curse_chance: cell(73, 0),
        curse_duration: cell(74, 0),
        savage_blow_chance: cell(75, 0),
        savage_blow_boost: cell(76, 0),
        dodge_chance: cell(77, 0),
        dodge_duration: cell(78, 0),
        toxic_chance: cell(79, 0),
        toxic_damage: cell(80, 0),
        surge_chance: cell(81, 0),
        surge_spawn_anchor: cell(82, 0) / 4,
        surge_spawn_span: cell(83, 0) / 4,
        surge_level: cell(84, 0),
        surge_immune: cell(85, 0),
        mini_wave_flag: cell(86, 0),
        shield_hitpoints: cell(87, 0),
        shield_regen: cell(88, 0),
        death_surge_chance: cell(89, 0),
        death_surge_spawn_anchor: cell(90, 0) / 4,
        death_surge_spawn_span: cell(91, 0) / 4,
        death_surge_level: cell(92, 0),
        trait_aku: cell(93, 0),
        trait_colossus: cell(94, 0),
        long_distance_2_flag: cell(95, 0),
        long_distance_2_anchor: cell(96, 0),
        long_distance_2_span: cell(97, 0),
        long_distance_3_flag: cell(98, 0),
        long_distance_3_anchor: cell(99, 0),
        long_distance_3_span: cell(100, 0),
        trait_behemoth: cell(101, 0),
        mini_surge_flag: cell(102, 0),
        counter_surge: cell(103, 0),
        trait_sage: cell(104, 0),
        curse_immune: cell(105, 0),
        explosion_chance: cell(106, 0),
        explosion_spawn_anchor: cell(107, 0) / 4,
        explosion_spawn_span: cell(108, 0) / 4,
        explosion_immune: cell(109, 0),
        trait_kaijin: cell(110, 0),
        drain_chance: cell(111, 0),
        drain_percent: cell(112, 0),
        conjure_unit_id: -1,
        ..Entity::default()
    };
    unit.has_unknown_abilities = entity::trailing_unknowns(cols, max_read.get() + 1);
    unit
}
