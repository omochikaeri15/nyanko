use std::cell::Cell;
use std::error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

/// Represents errors that can occur while parsing raw combat statistic rows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityError {
    /// The supplied bytes yielded no rows wide enough to be treated as combat data.
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

/// Identifies which side of a battle an entity belongs to.
///
/// The two factions read their statistics from different source files with
/// different column orderings, so this flag records which layout produced the
/// surrounding [`Entity`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    /// A player-controlled Cat unit, parsed from a per-unit `unit<id>.csv` file.
    #[default]
    Cat,
    /// An opposing enemy unit, parsed from a row of the shared `t_unit.csv` table.
    Enemy,
}

/// The complete mechanical combat definition of a single entity form.
///
/// The two factions store their statistics in separate files whose columns
/// diverge in order and meaning; both normalize into this layout, with the origin
/// recorded in [`Entity::faction`]. Fields absent from a faction's source file
/// keep their default.
///
/// Durations are frames at thirty per second, chances are whole percentages, and
/// distances are already divided by four where the raw column quadruples them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    /// The side of the battle this entity fights on, and therefore which source layout produced it.
    pub faction: Faction,
    /// The total health pool before any level or treasure scaling is applied.
    pub hitpoints: i32,
    /// The number of times the entity is repelled before it is defeated.
    pub knockbacks: i32,
    /// The horizontal movement rate in engine distance units per frame.
    pub speed: i32,
    /// The damage dealt by the first attack in the entity's attack sequence.
    pub attack_1: i32,
    /// The damage dealt by the second attack, or zero when the entity has a single-hit attack.
    pub attack_2: i32,
    /// The damage dealt by the third attack, or zero when the entity has fewer than three hits.
    pub attack_3: i32,
    /// The recovery period in frames between the end of one attack sequence and the start of the next.
    pub attack_cooldown: i32,
    /// The distance at which the entity halts and begins attacking.
    pub standing_range: i32,
    /// The offset of the damage hitbox relative to the entity's own position.
    pub hitbox_position: i32,
    /// The width of the damage hitbox.
    pub hitbox_width: i32,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unused: i32,
    /// The deployment cost in currency during the first chapter, before chapter cost multipliers.
    pub eoc1_cost: i32,
    /// The redeployment delay in frames after the entity is summoned.
    pub cooldown: i32,
    /// The currency awarded to the opposing player when this entity is defeated.
    pub cash_drop: i32,
    /// The lower bound of the random draw layer used to order overlapping sprites.
    pub minimum_z_layer: i32,
    /// The upper bound of the random draw layer used to order overlapping sprites.
    pub maximum_z_layer: i32,
    /// Non-zero when the entity carries the Red trait.
    pub trait_red: i32,
    /// Non-zero when the entity carries the Floating trait.
    pub trait_floating: i32,
    /// Non-zero when the entity carries the Black trait.
    pub trait_dark: i32,
    /// Non-zero when the entity carries the Metal trait.
    pub trait_metal: i32,
    /// Non-zero when the entity carries the Traitless classification.
    pub trait_traitless: i32,
    /// Non-zero when the entity carries the Angel trait.
    pub trait_angel: i32,
    /// Non-zero when the entity carries the Alien trait.
    pub trait_alien: i32,
    /// Non-zero when the entity carries the Zombie trait.
    pub trait_zombie: i32,
    /// Non-zero when the entity carries the Witch trait.
    pub trait_witch: i32,
    /// Non-zero when the entity carries the Eva Angel trait.
    pub trait_eva: i32,
    /// Non-zero when the entity carries the Relic trait.
    pub trait_relic: i32,
    /// Non-zero when the entity carries the Aku trait.
    pub trait_aku: i32,
    /// Non-zero when the entity carries the Dojo classification used by base-defense stages.
    pub trait_dojo: i32,
    /// Non-zero when the entity carries the Starred Alien trait.
    pub trait_starred_alien: i32,
    /// Non-zero when the entity carries the Behemoth trait.
    pub trait_behemoth: i32,
    /// Non-zero when the entity carries the Colossus trait.
    pub trait_colossus: i32,
    /// Non-zero when the entity carries the Sage trait.
    pub trait_sage: i32,
    /// Non-zero when the entity carries the Kaijin trait.
    pub trait_kaijin: i32,
    /// Non-zero when the entity damages every target within its hitbox rather than only the first.
    pub area_attack: i32,
    /// The frame within the attack animation at which the first attack applies its damage.
    pub time_until_attack_1: i32,
    /// The frame within the attack animation at which the second attack applies its damage.
    pub time_until_attack_2: i32,
    /// The frame within the attack animation at which the third attack applies its damage.
    pub time_until_attack_3: i32,
    /// A bitfield selecting which of the entity's abilities are carried by the first attack.
    pub attack_1_abilities: i32,
    /// A bitfield selecting which of the entity's abilities are carried by the second attack.
    pub attack_2_abilities: i32,
    /// A bitfield selecting which of the entity's abilities are carried by the third attack.
    pub attack_3_abilities: i32,
    /// The number of attacks that make up one complete attack sequence.
    pub attack_count_total: i32,
    /// The internal counter tracking which attack of the sequence is next to resolve.
    pub attack_count_state: i32,
    /// The delay in frames between the entity's death and the removal of its corpse.
    pub time_before_death: i32,
    /// The identifier of the animation played when the entity enters the battlefield.
    pub spawn_animation_type: i32,
    /// The identifier of the animation played when the entity is defeated.
    pub soul_animation_type: i32,
    /// Non-zero when the spawn animation is enabled for this entity.
    pub spawn_animation_flag: i32,
    /// Non-zero when the death animation is enabled for this entity.
    pub soul_animation_flag: i32,
    /// Non-zero when the entity may only attack and cannot block advancing opponents.
    pub attack_only: i32,
    /// The damage multiplier applied against the traits this entity is strong against.
    pub strong_against: i32,
    /// The damage multiplier applied by the Massive Damage ability.
    pub massive_damage: i32,
    /// The damage multiplier applied by the Insane Damage ability.
    pub insane_damage: i32,
    /// The incoming damage reduction applied by the Resistant ability.
    pub resist: i32,
    /// The incoming damage reduction applied by the Insanely Tough ability.
    pub insanely_tough: i32,
    /// Non-zero when the entity behaves as a Metal target, taking one damage from ordinary hits.
    pub is_metal: i32,
    /// Non-zero when the entity yields twice the ordinary currency on defeat.
    pub double_bounty: i32,
    /// The damage multiplier applied against Zombie targets, which also prevents their revival.
    pub zombie_killer: i32,
    /// Non-zero when the entity's attacks pierce the burrowed state of Zombie targets.
    pub soulstrike: i32,
    /// The damage multiplier applied against Colossus targets.
    pub colossus_slayer: i32,
    /// The damage multiplier applied against Sage targets.
    pub sage_slayer: i32,
    /// The damage multiplier applied against Behemoth targets.
    pub behemoth_slayer: i32,
    /// The percentage chance of evading an incoming attack from a Behemoth target.
    pub behemoth_dodge_chance: i32,
    /// The duration in frames of the evasion granted against Behemoth targets.
    pub behemoth_dodge_duration: i32,
    /// The damage multiplier applied against Witch targets.
    pub witch_killer: i32,
    /// The damage multiplier applied against Eva Angel targets.
    pub eva_killer: i32,
    /// The percentage of the target's health dealt as damage against Metal targets.
    pub metal_killer_percent: i32,
    /// The percentage chance of destroying a target's barrier outright.
    pub barrier_breaker_chance: i32,
    /// The percentage chance of ignoring a target's shield.
    pub shield_pierce_chance: i32,
    /// The identifier of the unit summoned by the Conjure ability, or negative one when absent.
    pub conjure_unit_id: i32,
    /// The percentage chance of repelling the target on hit.
    pub knockback_chance: i32,
    /// The percentage chance of freezing the target on hit.
    pub freeze_chance: i32,
    /// The duration in frames of the freeze effect.
    pub freeze_duration: i32,
    /// The percentage chance of slowing the target on hit.
    pub slow_chance: i32,
    /// The duration in frames of the slow effect.
    pub slow_duration: i32,
    /// The percentage chance of weakening the target on hit.
    pub weaken_chance: i32,
    /// The duration in frames of the weaken effect.
    pub weaken_duration: i32,
    /// The percentage of its original attack power a weakened target retains.
    pub weaken_to: i32,
    /// The percentage chance of cursing the target, suppressing its trait-based abilities.
    pub curse_chance: i32,
    /// The duration in frames of the curse effect.
    pub curse_duration: i32,
    /// The percentage chance of warping the target backwards on hit.
    pub warp_chance: i32,
    /// The duration in frames the target remains immobilized after being warped.
    pub warp_duration: i32,
    /// The shortest distance a warped target is displaced.
    pub warp_distance_minimum: i32,
    /// The longest distance a warped target is displaced.
    pub warp_distance_maximum: i32,
    /// The percentage chance of evading an incoming attack.
    pub dodge_chance: i32,
    /// The duration in frames of the evasion effect.
    pub dodge_duration: i32,
    /// The percentage chance of triggering a Savage Blow.
    pub savage_blow_chance: i32,
    /// The additional damage percentage applied by a Savage Blow.
    pub savage_blow_boost: i32,
    /// The percentage chance of dealing a critical hit, which also damages Metal targets normally.
    pub critical_chance: i32,
    /// The remaining health percentage below which the Strengthen ability activates.
    pub strengthen_threshold: i32,
    /// The additional attack percentage granted once Strengthen activates.
    pub strengthen_boost: i32,
    /// The percentage chance of surviving a lethal hit with one health point remaining.
    pub survive: i32,
    /// The damage multiplier applied against enemy and player bases.
    pub base_destroyer: i32,
    /// The percentage chance of emitting a shockwave on attack.
    pub wave_chance: i32,
    /// The level of the emitted shockwave, which determines how far it travels.
    pub wave_level: i32,
    /// Non-zero when the emitted shockwave deals reduced mini-wave damage.
    pub mini_wave_flag: i32,
    /// Non-zero when the entity is immune to shockwaves and prevents them from passing.
    pub wave_block: i32,
    /// The percentage chance of creating a surge at a distant point on attack.
    pub surge_chance: i32,
    /// The nearest distance at which a surge may be created.
    pub surge_spawn_anchor: i32,
    /// The width of the range within which a surge may be created.
    pub surge_spawn_span: i32,
    /// The level of the created surge, which determines its duration.
    pub surge_level: i32,
    /// Non-zero when the created surge deals reduced mini-surge damage.
    pub mini_surge_flag: i32,
    /// Non-zero when the entity retaliates against surges that strike it.
    pub counter_surge: i32,
    /// The percentage chance of creating an explosion at a distant point on attack.
    pub explosion_chance: i32,
    /// The nearest distance at which an explosion may be created.
    pub explosion_spawn_anchor: i32,
    /// The width of the range within which an explosion may be created.
    pub explosion_spawn_span: i32,
    /// The percentage chance of creating a surge at the point of the entity's death.
    pub death_surge_chance: i32,
    /// The nearest distance at which a death surge may be created.
    pub death_surge_spawn_anchor: i32,
    /// The width of the range within which a death surge may be created.
    pub death_surge_spawn_span: i32,
    /// The level of the created death surge, which determines its duration.
    pub death_surge_level: i32,
    /// The nearest distance covered by the entity's first long-distance attack band.
    pub long_distance_1_anchor: i32,
    /// The width of the entity's first long-distance attack band.
    pub long_distance_1_span: i32,
    /// Non-zero when the second long-distance attack band is active.
    pub long_distance_2_flag: i32,
    /// The nearest distance covered by the entity's second long-distance attack band.
    pub long_distance_2_anchor: i32,
    /// The width of the entity's second long-distance attack band.
    pub long_distance_2_span: i32,
    /// Non-zero when the third long-distance attack band is active.
    pub long_distance_3_flag: i32,
    /// The nearest distance covered by the entity's third long-distance attack band.
    pub long_distance_3_anchor: i32,
    /// The width of the entity's third long-distance attack band.
    pub long_distance_3_span: i32,
    /// Non-zero when the entity ignores incoming shockwaves.
    pub wave_immune: i32,
    /// Non-zero when the entity ignores incoming knockback effects.
    pub knockback_immune: i32,
    /// Non-zero when the entity ignores incoming freeze effects.
    pub freeze_immune: i32,
    /// Non-zero when the entity ignores incoming slow effects.
    pub slow_immune: i32,
    /// Non-zero when the entity ignores incoming weaken effects.
    pub weaken_immune: i32,
    /// Non-zero when the entity ignores incoming warp effects.
    pub warp_immune: i32,
    /// Non-zero when the entity ignores incoming curse effects.
    pub curse_immune: i32,
    /// Non-zero when the entity ignores incoming toxic damage.
    pub toxic_immune: i32,
    /// Non-zero when the entity ignores incoming surges.
    pub surge_immune: i32,
    /// Non-zero when the entity ignores incoming explosions.
    pub explosion_immune: i32,
    /// Non-zero when the entity ignores the knockback inflicted by boss waves.
    pub boss_wave_immune: i32,
    /// The health pool of the barrier that must be broken before the entity itself takes damage.
    pub barrier_hitpoints: i32,
    /// The health pool of the regenerating shield that absorbs incoming damage.
    pub shield_hitpoints: i32,
    /// The percentage of shield health restored once the shield is broken.
    pub shield_regen: i32,
    /// The number of times the entity may burrow beneath attacks.
    pub burrow_amount: i32,
    /// The distance from the player base at which the entity begins to burrow.
    pub burrow_distance: i32,
    /// The number of times the entity returns to life after being defeated.
    pub revive_count: i32,
    /// The delay in frames between the entity's death and its revival.
    pub revive_time: i32,
    /// The percentage of maximum health the entity is restored to on revival.
    pub revive_hp: i32,
    /// The percentage chance of inflicting proportional toxic damage on hit.
    pub toxic_chance: i32,
    /// The percentage of the target's maximum health dealt as toxic damage.
    pub toxic_damage: i32,
    /// The percentage chance of restoring health proportional to the damage dealt.
    pub drain_chance: i32,
    /// The percentage of damage dealt that is restored as health.
    pub drain_percent: i32,
    /// Non-zero when the source row carried trailing values this parser does not interpret.
    pub has_unknown_abilities: i32,
}

impl Entity {
    /// Calculates the total length in frames of one complete attack cycle.
    ///
    /// The cycle spans from the start of the attack animation through the frame
    /// on which the final hit lands and then through the recovery period. When
    /// the entity has multiple attacks, the effective wind-up is taken from the
    /// last attack that both deals damage and declares a strike frame.
    ///
    /// # Arguments
    /// * `frames` - The measured length of the attack animation, used as a lower bound when the statistical values imply a shorter cycle.
    ///
    /// # Returns
    /// An `i32` containing the greater of the computed cycle length and the supplied animation length.
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

pub(crate) fn parse_single(bytes: &[u8], skip: usize, id: usize, build: fn(&[&str]) -> Entity) -> Option<Entity> {
    let content = file::scrub(bytes);
    let separator = file::detect_separator(&content);
    let target_line = content.lines().skip(skip).nth(id)?;
    let cols: Vec<&str> = target_line.split(separator).collect();
    if cols.len() < 10 {
        return None;
    }
    Some(build(&cols))
}
