use std::cell::Cell;
use std::error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityError {
    EmptyFile,
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid combat entities."),
        }
    }
}

impl error::Error for EntityError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    #[default]
    Cat,
    Enemy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entity {
    pub faction: Faction,
    pub hitpoints: i32,
    pub knockbacks: i32,
    pub speed: i32,
    pub attack_1: i32,
    pub attack_2: i32,
    pub attack_3: i32,
    pub attack_cooldown: i32,
    pub standing_range: i32,
    pub hitbox_position: i32,
    pub hitbox_width: i32,
    pub unused: i32,
    pub eoc1_cost: i32,
    pub cooldown: i32,
    pub cash_drop: i32,
    pub minimum_z_layer: i32,
    pub maximum_z_layer: i32,
    pub trait_red: i32,
    pub trait_floating: i32,
    pub trait_dark: i32,
    pub trait_metal: i32,
    pub trait_traitless: i32,
    pub trait_angel: i32,
    pub trait_alien: i32,
    pub trait_zombie: i32,
    pub trait_witch: i32,
    pub trait_eva: i32,
    pub trait_relic: i32,
    pub trait_aku: i32,
    pub trait_dojo: i32,
    pub trait_starred_alien: i32,
    pub trait_behemoth: i32,
    pub trait_colossus: i32,
    pub trait_sage: i32,
    pub trait_kaijin: i32,
    pub area_attack: i32,
    pub time_until_attack_1: i32,
    pub time_until_attack_2: i32,
    pub time_until_attack_3: i32,
    pub attack_1_abilities: i32,
    pub attack_2_abilities: i32,
    pub attack_3_abilities: i32,
    pub attack_count_total: i32,
    pub attack_count_state: i32,
    pub time_before_death: i32,
    pub spawn_animation_type: i32,
    pub soul_animation_type: i32,
    pub spawn_animation_flag: i32,
    pub soul_animation_flag: i32,
    pub attack_only: i32,
    pub strong_against: i32,
    pub massive_damage: i32,
    pub insane_damage: i32,
    pub resist: i32,
    pub insanely_tough: i32,
    pub is_metal: i32,
    pub double_bounty: i32,
    pub zombie_killer: i32,
    pub soulstrike: i32,
    pub colossus_slayer: i32,
    pub sage_slayer: i32,
    pub behemoth_slayer: i32,
    pub behemoth_dodge_chance: i32,
    pub behemoth_dodge_duration: i32,
    pub witch_killer: i32,
    pub eva_killer: i32,
    pub metal_killer_percent: i32,
    pub barrier_breaker_chance: i32,
    pub shield_pierce_chance: i32,
    pub conjure_unit_id: i32,
    pub knockback_chance: i32,
    pub freeze_chance: i32,
    pub freeze_duration: i32,
    pub slow_chance: i32,
    pub slow_duration: i32,
    pub weaken_chance: i32,
    pub weaken_duration: i32,
    pub weaken_to: i32,
    pub curse_chance: i32,
    pub curse_duration: i32,
    pub warp_chance: i32,
    pub warp_duration: i32,
    pub warp_distance_minimum: i32,
    pub warp_distance_maximum: i32,
    pub dodge_chance: i32,
    pub dodge_duration: i32,
    pub savage_blow_chance: i32,
    pub savage_blow_boost: i32,
    pub critical_chance: i32,
    pub strengthen_threshold: i32,
    pub strengthen_boost: i32,
    pub survive: i32,
    pub base_destroyer: i32,
    pub wave_chance: i32,
    pub wave_level: i32,
    pub mini_wave_flag: i32,
    pub wave_block: i32,
    pub surge_chance: i32,
    pub surge_spawn_anchor: i32,
    pub surge_spawn_span: i32,
    pub surge_level: i32,
    pub mini_surge_flag: i32,
    pub counter_surge: i32,
    pub explosion_chance: i32,
    pub explosion_spawn_anchor: i32,
    pub explosion_spawn_span: i32,
    pub death_surge_chance: i32,
    pub death_surge_spawn_anchor: i32,
    pub death_surge_spawn_span: i32,
    pub death_surge_level: i32,
    pub long_distance_1_anchor: i32,
    pub long_distance_1_span: i32,
    pub long_distance_2_flag: i32,
    pub long_distance_2_anchor: i32,
    pub long_distance_2_span: i32,
    pub long_distance_3_flag: i32,
    pub long_distance_3_anchor: i32,
    pub long_distance_3_span: i32,
    pub wave_immune: i32,
    pub knockback_immune: i32,
    pub freeze_immune: i32,
    pub slow_immune: i32,
    pub weaken_immune: i32,
    pub warp_immune: i32,
    pub curse_immune: i32,
    pub toxic_immune: i32,
    pub surge_immune: i32,
    pub explosion_immune: i32,
    pub boss_wave_immune: i32,
    pub barrier_hitpoints: i32,
    pub shield_hitpoints: i32,
    pub shield_regen: i32,
    pub burrow_amount: i32,
    pub burrow_distance: i32,
    pub revive_count: i32,
    pub revive_time: i32,
    pub revive_hp: i32,
    pub toxic_chance: i32,
    pub toxic_damage: i32,
    pub drain_chance: i32,
    pub drain_percent: i32,
    pub has_unknown_abilities: i32,
}

impl Entity {
    pub fn attack_cycle(&self, frames: i32) -> i32 {
        let mut effective_foreswing = self.time_until_attack_1;

        if self.attack_3 > 0 && self.time_until_attack_3 > 0 {
            effective_foreswing = self.time_until_attack_3;
        }
        else if self.attack_2 > 0 && self.time_until_attack_2 > 0 {
            effective_foreswing = self.time_until_attack_2;
        }

        let cooldown_frames = self.attack_cooldown.saturating_sub(1);

        (effective_foreswing + cooldown_frames).max(frames)
    }
}

pub(crate) fn reader<'a>(cols: &'a [&'a str], max_read: &'a Cell<usize>) -> impl Fn(usize, i32) -> i32 + 'a {
    move |index, fallback| {
        max_read.set(max_read.get().max(index));
        cols.get(index).and_then(|s| s.trim().parse::<i32>().ok()).unwrap_or(fallback)
    }
}

pub(crate) fn trailing_unknowns(cols: &[&str], start: usize) -> i32 {
    i32::from(cols.iter().skip(start).any(|col| {
        let value = col.trim().parse::<i32>().unwrap_or(0);
        value != 0 && value != -1
    }))
}

pub(crate) fn parse_rows(bytes: &[u8], skip: usize, build: fn(&[&str]) -> Entity) -> Result<Vec<Entity>, EntityError> {
    let content = file::scrub(bytes);
    let separator = file::detect_separator(&content);
    let entities: Vec<Entity> = content
        .lines()
        .skip(skip)
        .filter_map(|line| {
            let cols: Vec<&str> = line.split(separator).collect();
            (cols.len() >= 10).then(|| build(&cols))
        })
        .collect();
    if entities.is_empty() {
        return Err(EntityError::EmptyFile);
    }
    Ok(entities)
}

pub(crate) fn parse_single(bytes: &[u8], skip: usize, id: usize, build: fn(&[&str]) -> Entity) -> Result<Option<Entity>, EntityError> {
    let content = file::scrub(bytes);
    let separator = file::detect_separator(&content);
    let Some(target_line) = content.lines().skip(skip).nth(id) else {
        return Ok(None);
    };
    let cols: Vec<&str> = target_line.split(separator).collect();
    if cols.len() < 10 {
        return Ok(None);
    }
    Ok(Some(build(&cols)))
}
