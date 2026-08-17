//! Parsing of a Cat unit's per-form combat statistics.
//!
//! The engine stores these in a `unit<id>.csv` file alongside the unit's other
//! assets, with one row per evolutionary form and no header.

use std::cell::Cell;

use crate::combat::{Entity, EntityError, Faction, entity};

/// Parses a per-unit Cat statistic file into one entity per evolutionary form.
///
/// The file carries no header, and each row describes one form in ascending
/// order beginning with the normal form. Units with fewer than four forms
/// declare correspondingly fewer rows.
///
/// # Arguments
/// * `bytes` - The raw, decrypted byte slice of a unit's `unit<id>.csv` file.
///
/// # Returns
/// A `Result` containing the parsed entities in form order on success, or an
/// `EntityError` if the file contained no rows wide enough to be interpreted
/// as combat data.
pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Vec<Entity>, EntityError> {
    entity::parse_rows(bytes.as_ref(), 0, from_row)
}

fn from_row(cols: &[&str]) -> Entity {
    let max_read = Cell::new(0);
    let cell = entity::reader(cols, &max_read);
    let mut unit = Entity {
        faction: Faction::Cat,
        hitpoints: cell(0, 0),
        knockbacks: cell(1, 0),
        speed: cell(2, 0),
        attack_1: cell(3, 0),
        attack_cooldown: cell(4, 0) * 2,
        standing_range: cell(5, 0),
        eoc1_cost: cell(6, 0),
        cooldown: cell(7, 0) * 2,
        hitbox_position: cell(8, 0),
        hitbox_width: cell(9, 0),
        trait_red: cell(10, 0),
        unused: cell(11, 0),
        area_attack: cell(12, 0),
        time_until_attack_1: cell(13, 0),
        minimum_z_layer: cell(14, 0),
        maximum_z_layer: cell(15, 0),
        trait_floating: cell(16, 0),
        trait_dark: cell(17, 0),
        trait_metal: cell(18, 0),
        trait_traitless: cell(19, 0),
        trait_angel: cell(20, 0),
        trait_alien: cell(21, 0),
        trait_zombie: cell(22, 0),
        strong_against: cell(23, 0),
        knockback_chance: cell(24, 0),
        freeze_chance: cell(25, 0),
        freeze_duration: cell(26, 0),
        slow_chance: cell(27, 0),
        slow_duration: cell(28, 0),
        resist: cell(29, 0),
        massive_damage: cell(30, 0),
        critical_chance: cell(31, 0),
        attack_only: cell(32, 0),
        double_bounty: cell(33, 0),
        base_destroyer: cell(34, 0),
        wave_chance: cell(35, 0),
        wave_level: cell(36, 0),
        weaken_chance: cell(37, 0),
        weaken_duration: cell(38, 0),
        weaken_to: cell(39, 0),
        strengthen_threshold: cell(40, 0),
        strengthen_boost: cell(41, 0),
        survive: cell(42, 0),
        is_metal: cell(43, 0),
        long_distance_1_anchor: cell(44, 0),
        long_distance_1_span: cell(45, 0),
        wave_immune: cell(46, 0),
        wave_block: cell(47, 0),
        knockback_immune: cell(48, 0),
        freeze_immune: cell(49, 0),
        slow_immune: cell(50, 0),
        weaken_immune: cell(51, 0),
        zombie_killer: cell(52, 0),
        witch_killer: cell(53, 0),
        trait_witch: cell(54, 0),
        attack_count_total: cell(55, -1),
        boss_wave_immune: cell(56, -1),
        time_before_death: cell(57, -1),
        attack_count_state: cell(58, 0),
        attack_2: cell(59, 0),
        attack_3: cell(60, 0),
        time_until_attack_2: cell(61, 0),
        time_until_attack_3: cell(62, 0),
        attack_1_abilities: cell(63, 0),
        attack_2_abilities: cell(64, 0),
        attack_3_abilities: cell(65, 0),
        spawn_animation_type: cell(66, -1),
        soul_animation_type: cell(67, 0),
        spawn_animation_flag: cell(68, 0),
        soul_animation_flag: cell(69, 0),
        barrier_breaker_chance: cell(70, 0),
        warp_chance: cell(71, 0),
        warp_duration: cell(72, 0),
        warp_distance_minimum: cell(73, 0) / 4,
        warp_distance_maximum: cell(74, 0) / 4,
        warp_immune: cell(75, 0),
        trait_eva: cell(76, 0),
        eva_killer: cell(77, 0),
        trait_relic: cell(78, 0),
        curse_immune: cell(79, 0),
        insanely_tough: cell(80, 0),
        insane_damage: cell(81, 0),
        savage_blow_chance: cell(82, 0),
        savage_blow_boost: cell(83, 0),
        dodge_chance: cell(84, 0),
        dodge_duration: cell(85, 0),
        surge_chance: cell(86, 0),
        surge_spawn_anchor: cell(87, 0) / 4,
        surge_spawn_span: cell(88, 0) / 4,
        surge_level: cell(89, 0),
        toxic_immune: cell(90, 0),
        surge_immune: cell(91, 0),
        curse_chance: cell(92, 0),
        curse_duration: cell(93, 0),
        mini_wave_flag: cell(94, 0),
        shield_pierce_chance: cell(95, 0),
        trait_aku: cell(96, 0),
        colossus_slayer: cell(97, 0),
        soulstrike: cell(98, 0),
        long_distance_2_flag: cell(99, 0),
        long_distance_2_anchor: cell(100, 0),
        long_distance_2_span: cell(101, 0),
        long_distance_3_flag: cell(102, 0),
        long_distance_3_anchor: cell(103, 0),
        long_distance_3_span: cell(104, 0),
        behemoth_slayer: cell(105, 0),
        behemoth_dodge_chance: cell(106, 0),
        behemoth_dodge_duration: cell(107, 0),
        mini_surge_flag: cell(108, 0),
        counter_surge: cell(109, 0),
        conjure_unit_id: cell(110, -1),
        sage_slayer: cell(111, 0),
        metal_killer_percent: cell(112, 0),
        explosion_chance: cell(113, 0),
        explosion_spawn_anchor: cell(114, 0) / 4,
        explosion_spawn_span: cell(115, 0) / 4,
        explosion_immune: cell(116, 0),
        ..Entity::default()
    };
    unit.has_unknown_abilities = entity::trailing_unknowns(cols, max_read.get() + 1);
    unit
}
