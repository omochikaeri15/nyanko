use crate::cat::unit::TalentGroup;
use crate::common::data::img015;

use super::{Entity, Faction};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AttrUnit {
    None,
    Percent,
    Frames,
    Range,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AttrValue {
    Finite(i32),
    Infinite,
}

impl AttrValue {
    pub fn from_sentinel(raw: i32) -> Self {
        if raw < 0 { Self::Infinite } else { Self::Finite(raw) }
    }
}

impl From<i32> for AttrValue {
    fn from(raw: i32) -> Self {
        Self::Finite(raw)
    }
}

pub type Attribute = (&'static str, AttrValue, AttrUnit);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Identity {
    SingleAttack, AreaAttack, MultiHit, LongDistance, OmniStrike,
    TraitRed, TraitFloating, TraitDark, TraitMetal, TraitTraitless, TraitAngel,
    TraitAlien, TraitZombie, TraitWitch, TraitEva, TraitRelic, TraitAku,
    TraitDojo, TraitStarredAlien, TraitCatGod, TraitBehemoth, TraitColossus,
    TraitSage, TraitKaijin,
    AttackOnly, StrongAgainst, MassiveDamage, InsaneDamage, Resist, InsanelyTough,
    IsMetal, DoubleBounty, ZombieKiller, Soulstrike, ColossusSlayer, SageSlayer,
    BehemothSlayer, WitchKiller, EvaKiller, MetalKiller, BarrierBreaker,
    ShieldPiercer, Conjure,
    BaseDestroyer, Kamikaze, Stop, WaveBlock, CounterSurge, WaveAttack, MiniWave,
    SurgeAttack, MiniSurge, Explosion, SavageBlow, CriticalHit, Strengthen,
    Survive, Dodge, Weaken, Freeze, Slow, Knockback, Curse, Warp, Unknown,
    Barrier, AkuShield, Burrow, Revive, Toxic, Drain, DeathSurge,
    ImmuneWave, ImmuneSurge, ImmuneExplosion, ImmuneWeaken, ImmuneFreeze,
    ImmuneSlow, ImmuneKnockback, ImmuneCurse, ImmuneToxic, ImmuneWarp,
    ImmuneBossWave,
    ResistWeaken, ResistFreeze, ResistSlow, ResistKnockback, ResistWave,
    ResistWarp, ResistCurse, ResistToxic, ResistSurge,
    CostDown, RecoverSpeedUp, MoveSpeedUp, AttackBuff, HealthBuff, TbaDown,
    ImproveKnockbacks,
}

pub struct Ability {
    pub identity: Identity,
    pub talent_id: Option<u8>,
    pub icon_id: Option<usize>,
    pub name: &'static str,
    pub description: &'static str,
    pub schema: &'static [(&'static str, AttrUnit)],
    pub attributes: fn(&Entity) -> Vec<Attribute>,
    pub apply_talent: Option<fn(&mut Entity, val1: i32, val2: i32, group: &TalentGroup)>,
}

fn active() -> Vec<Attribute> {
    vec![("Active", AttrValue::Finite(1), AttrUnit::None)]
}

fn flag(raw: i32) -> Vec<Attribute> {
    if raw > 0 { active() } else { Vec::new() }
}

fn get_dur_val(v1: i32, v2: i32) -> i32 {
    if v1 != 0 { v1 } else { v2 }
}

fn wave_reach(stats: &Entity) -> i32 {
    let base = match stats.faction {
        Faction::Cat => 332.5,
        Faction::Enemy => 467.5,
    };
    (base + ((stats.wave_level - 1) as f32 * 200.0)).round() as i32
}

fn has_omni(stats: &Entity) -> bool {
    (stats.long_distance_1_span < 0 || (stats.long_distance_1_span == 0 && stats.long_distance_1_anchor != 0)) ||
        (stats.long_distance_2_flag > 0 && (stats.long_distance_2_span < 0 || (stats.long_distance_2_span == 0 && stats.long_distance_2_anchor != 0))) ||
        (stats.long_distance_3_flag > 0 && (stats.long_distance_3_span < 0 || (stats.long_distance_3_span == 0 && stats.long_distance_3_anchor != 0)))
}

fn has_long_distance(stats: &Entity) -> bool {
    (stats.long_distance_1_span > 0) ||
        (stats.long_distance_2_flag > 0 && stats.long_distance_2_span > 0) ||
        (stats.long_distance_3_flag > 0 && stats.long_distance_3_span > 0)
}

pub fn get_talent(id: u8) -> Option<&'static Ability> {
    REGISTRY.iter().find(|ability| ability.talent_id == Some(id))
}

pub fn get_ability(identity: Identity) -> Option<&'static Ability> {
    REGISTRY.iter().find(|ability| ability.identity == identity)
}

pub static REGISTRY: &[Ability] = &[
    Ability {
        identity: Identity::SingleAttack,
        talent_id: None,
        icon_id: Some(img015::ICON_SINGLE_ATTACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.area_attack == 0 { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::AreaAttack,
        talent_id: None,
        icon_id: Some(img015::ICON_AREA_ATTACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.area_attack == 1 { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::MultiHit,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.attack_2),
        apply_talent: None,
    },
    Ability {
        identity: Identity::LongDistance,
        talent_id: None,
        icon_id: Some(img015::ICON_LONG_DISTANCE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if has_long_distance(stats) && !has_omni(stats) { active() } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::OmniStrike,
        talent_id: None,
        icon_id: Some(img015::ICON_OMNI_STRIKE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if has_omni(stats) { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitRed,
        talent_id: Some(33),
        icon_id: Some(img015::ICON_TRAIT_RED),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_red),
        apply_talent: Some(|stats,_,_,_| stats.trait_red = 1),
    },
    Ability {
        identity: Identity::TraitFloating,
        talent_id: Some(34),
        icon_id: Some(img015::ICON_TRAIT_FLOATING),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_floating),
        apply_talent: Some(|stats,_,_,_| stats.trait_floating = 1),
    },
    Ability {
        identity: Identity::TraitDark,
        talent_id: Some(35),
        icon_id: Some(img015::ICON_TRAIT_BLACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_dark),
        apply_talent: Some(|stats,_,_,_| stats.trait_dark = 1),
    },
    Ability {
        identity: Identity::TraitMetal,
        talent_id: Some(36),
        icon_id: Some(img015::ICON_TRAIT_METAL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_metal),
        apply_talent: Some(|stats,_,_,_| stats.trait_metal = 1),
    },
    Ability {
        identity: Identity::TraitAngel,
        talent_id: Some(37),
        icon_id: Some(img015::ICON_TRAIT_ANGEL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_angel),
        apply_talent: Some(|stats,_,_,_| stats.trait_angel = 1),
    },
    Ability {
        identity: Identity::TraitAlien,
        talent_id: Some(38),
        icon_id: Some(img015::ICON_TRAIT_ALIEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_alien),
        apply_talent: Some(|stats,_,_,_| stats.trait_alien = 1),
    },
    Ability {
        identity: Identity::TraitZombie,
        talent_id: Some(39),
        icon_id: Some(img015::ICON_TRAIT_ZOMBIE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_zombie),
        apply_talent: Some(|stats,_,_,_| stats.trait_zombie = 1),
    },
    Ability {
        identity: Identity::TraitRelic,
        talent_id: Some(40),
        icon_id: Some(img015::ICON_TRAIT_RELIC),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_relic),
        apply_talent: Some(|stats,_,_,_| stats.trait_relic = 1),
    },
    Ability {
        identity: Identity::TraitAku,
        talent_id: Some(57),
        icon_id: Some(img015::ICON_TRAIT_AKU),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_aku),
        apply_talent: Some(|stats,_,_,_| stats.trait_aku = 1),
    },
    Ability {
        identity: Identity::TraitTraitless,
        talent_id: Some(41),
        icon_id: Some(img015::ICON_TRAIT_TRAITLESS),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_traitless),
        apply_talent: Some(|stats,_,_,_| stats.trait_traitless = 1),
    },
    Ability {
        identity: Identity::TraitWitch,
        talent_id: None,
        icon_id: Some(img015::ICON_WITCH),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_witch),
        apply_talent: Some(|stats,_,_,_| stats.trait_witch = 1),
    },
    Ability {
        identity: Identity::TraitEva,
        talent_id: None,
        icon_id: Some(img015::ICON_EVA),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_eva),
        apply_talent: Some(|stats,_,_,_| stats.trait_eva = 1),
    },
    Ability {
        identity: Identity::TraitDojo,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_dojo),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitStarredAlien,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| if stats.trait_starred_alien == 1 { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitCatGod,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Type", AttrUnit::None)],
        attributes: |stats| {
            if stats.trait_starred_alien >= 2 && stats.trait_starred_alien <= 4 {
                vec![("Type", AttrValue::Finite(stats.trait_starred_alien), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitColossus,
        talent_id: None,
        icon_id: Some(img015::ICON_COLOSSUS),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_colossus),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitBehemoth,
        talent_id: None,
        icon_id: Some(img015::ICON_BEHEMOTH),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_behemoth),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitSage,
        talent_id: None,
        icon_id: Some(img015::ICON_SAGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_sage),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitKaijin,
        talent_id: None,
        icon_id: Some(img015::ICON_SUPERVILLIAN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.trait_kaijin),
        apply_talent: None,
    },
    Ability {
        identity: Identity::AttackOnly,
        talent_id: Some(4),
        icon_id: Some(img015::ICON_ATTACK_ONLY),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.attack_only),
        apply_talent: Some(|stats, _, _, _| stats.attack_only = 1),
    },
    Ability {
        identity: Identity::StrongAgainst,
        talent_id: Some(5),
        icon_id: Some(img015::ICON_STRONG_AGAINST),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.strong_against),
        apply_talent: Some(|stats, _, _, _| stats.strong_against = 1),
    },
    Ability {
        identity: Identity::MassiveDamage,
        talent_id: Some(7),
        icon_id: Some(img015::ICON_MASSIVE_DAMAGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.massive_damage),
        apply_talent: Some(|stats, _, _, _| stats.massive_damage = 1),
    },
    Ability {
        identity: Identity::InsaneDamage,
        talent_id: Some(7),
        icon_id: Some(img015::ICON_INSANE_DAMAGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.insane_damage),
        apply_talent: None,
    },
    Ability {
        identity: Identity::Resist,
        talent_id: Some(6),
        icon_id: Some(img015::ICON_RESIST),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.resist),
        apply_talent: Some(|stats, _, _, _| stats.resist = 1),
    },
    Ability {
        identity: Identity::InsanelyTough,
        talent_id: Some(6),
        icon_id: Some(img015::ICON_INSANELY_TOUGH),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.insanely_tough),
        apply_talent: None,
    },
    Ability {
        identity: Identity::IsMetal,
        talent_id: Some(43),
        icon_id: Some(img015::ICON_METAL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.is_metal),
        apply_talent: Some(|stats,_,_,_| stats.is_metal = 1),
    },
    Ability {
        identity: Identity::DoubleBounty,
        talent_id: Some(16),
        icon_id: Some(img015::ICON_DOUBLE_BOUNTY),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.double_bounty),
        apply_talent: Some(|stats, _, _, _| stats.double_bounty = 1),
    },
    Ability {
        identity: Identity::ZombieKiller,
        talent_id: Some(14),
        icon_id: Some(img015::ICON_ZOMBIE_KILLER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.zombie_killer),
        apply_talent: Some(|stats, _, _, _| stats.zombie_killer = 1),
    },
    Ability {
        identity: Identity::Soulstrike,
        talent_id: Some(59),
        icon_id: Some(img015::ICON_SOULSTRIKE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.soulstrike == 2 || (stats.soulstrike > 0 && stats.zombie_killer > 0) { active() } else { Vec::new() }
        },
        apply_talent: Some(|stats, _, _, _| stats.soulstrike = 2),
    },
    Ability {
        identity: Identity::ColossusSlayer,
        talent_id: Some(63),
        icon_id: Some(img015::ICON_COLOSSUS_SLAYER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.colossus_slayer),
        apply_talent: Some(|stats, _, _, _| stats.colossus_slayer = 1),
    },
    Ability {
        identity: Identity::SageSlayer,
        talent_id: Some(66),
        icon_id: Some(img015::ICON_SAGE_SLAYER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.sage_slayer),
        apply_talent: Some(|stats, _, _, _| stats.sage_slayer = 1),
    },
    Ability {
        identity: Identity::BehemothSlayer,
        talent_id: Some(64),
        icon_id: Some(img015::ICON_BEHEMOTH_SLAYER),
        name: "",
        description: "",
        schema: &[
            ("Dodge Chance", AttrUnit::Percent),
            ("Dodge Duration", AttrUnit::Frames),
        ],
        attributes: |stats| {
            if stats.behemoth_slayer <= 0 { return Vec::new(); }
            if stats.behemoth_dodge_chance > 0 {
                vec![
                    ("Active", AttrValue::Finite(1), AttrUnit::None),
                    ("Dodge Chance", AttrValue::Finite(stats.behemoth_dodge_chance), AttrUnit::Percent),
                    ("Dodge Duration", AttrValue::Finite(stats.behemoth_dodge_duration), AttrUnit::Frames),
                ]
            } else {
                active()
            }
        },
        apply_talent: Some(|stats, chance, duration, _| {
            stats.behemoth_slayer = 1;
            stats.behemoth_dodge_chance = if chance > 0 { chance } else { 5 };
            stats.behemoth_dodge_duration = if duration > 0 { duration } else { 30 };
        }),
    },
    Ability {
        identity: Identity::WitchKiller,
        talent_id: None,
        icon_id: Some(img015::ICON_WITCH_KILLER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.witch_killer),
        apply_talent: Some(|stats,_,_,_| stats.witch_killer = 1),
    },
    Ability {
        identity: Identity::EvaKiller,
        talent_id: None,
        icon_id: Some(img015::ICON_EVA_KILLER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.eva_killer),
        apply_talent: Some(|stats,_,_,_| stats.eva_killer = 1),
    },
    Ability {
        identity: Identity::MetalKiller,
        talent_id: None,
        icon_id: Some(img015::ICON_METAL_KILLER),
        name: "",
        description: "",
        schema: &[("Damage", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.metal_killer_percent > 0 {
                vec![("Damage", AttrValue::Finite(stats.metal_killer_percent), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, percent, _, _| stats.metal_killer_percent = percent),
    },
    Ability {
        identity: Identity::BarrierBreaker,
        talent_id: Some(15),
        icon_id: Some(img015::ICON_BARRIER_BREAKER),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.barrier_breaker_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.barrier_breaker_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.barrier_breaker_chance += chance),
    },
    Ability {
        identity: Identity::ShieldPiercer,
        talent_id: Some(58),
        icon_id: Some(img015::ICON_SHIELD_PIERCER),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.shield_pierce_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.shield_pierce_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.shield_pierce_chance += chance),
    },
    Ability {
        identity: Identity::Conjure,
        talent_id: None,
        icon_id: Some(img015::ICON_CONJURE),
        name: "",
        description: "",
        schema: &[("Spirit ID", AttrUnit::None)],
        attributes: |stats| {
            if stats.conjure_unit_id > -1 {
                vec![("Spirit ID", AttrValue::Finite(stats.conjure_unit_id), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::BaseDestroyer,
        talent_id: Some(12),
        icon_id: Some(img015::ICON_BASE_DESTROYER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.base_destroyer),
        apply_talent: Some(|stats, _, _, _| stats.base_destroyer = 1),
    },
    Ability {
        identity: Identity::Kamikaze,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Attacks", AttrUnit::None)],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 2 {
                vec![("Attacks", AttrValue::Finite(stats.attack_count_total), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Stop,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Attacks", AttrUnit::None)],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 0 {
                vec![("Attacks", AttrValue::Finite(stats.attack_count_total), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::WaveBlock,
        talent_id: None,
        icon_id: Some(img015::ICON_WAVE_BLOCK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.wave_block),
        apply_talent: Some(|stats, _, _, _| stats.wave_block = 1),
    },
    Ability {
        identity: Identity::CounterSurge,
        talent_id: Some(68),
        icon_id: Some(img015::ICON_COUNTER_SURGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.counter_surge),
        apply_talent: Some(|stats,_,_,_| stats.counter_surge = 1),
    },
    Ability {
        identity: Identity::WaveAttack,
        talent_id: Some(17),
        icon_id: Some(img015::ICON_WAVE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Max Reach", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.mini_wave_flag == 0 && stats.wave_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.wave_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.wave_level), AttrUnit::None),
                    ("Max Reach", AttrValue::Finite(wave_reach(stats)), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, _| { stats.wave_chance += chance; stats.wave_level = level; }),
    },
    Ability {
        identity: Identity::MiniWave,
        talent_id: Some(62),
        icon_id: Some(img015::ICON_MINI_WAVE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Max Reach", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.mini_wave_flag > 0 && stats.wave_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.wave_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.wave_level), AttrUnit::None),
                    ("Max Reach", AttrValue::Finite(wave_reach(stats)), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, _| { stats.mini_wave_flag = 1; stats.wave_chance += chance; stats.wave_level = level; }),
    },
    Ability {
        identity: Identity::SurgeAttack,
        talent_id: Some(56),
        icon_id: Some(img015::ICON_SURGE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.mini_surge_flag == 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.surge_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.surge_level), AttrUnit::None),
                    ("Min Range", AttrValue::Finite(stats.surge_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.surge_spawn_anchor + stats.surge_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.surge_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, group_data| {
            stats.surge_chance += chance;
            stats.surge_level = level;
            stats.surge_spawn_anchor = group_data.min_3 as i32 / 4;
            stats.surge_spawn_span = group_data.min_4 as i32 / 4;
        }),
    },
    Ability {
        identity: Identity::MiniSurge,
        talent_id: Some(65),
        icon_id: Some(img015::ICON_MINI_SURGE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.mini_surge_flag > 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.surge_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.surge_level), AttrUnit::None),
                    ("Min Range", AttrValue::Finite(stats.surge_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.surge_spawn_anchor + stats.surge_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.surge_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, group_data| {
            stats.mini_surge_flag = 1;
            stats.surge_chance += chance;
            stats.surge_level = level;
            stats.surge_spawn_anchor = group_data.min_3 as i32 / 4;
            stats.surge_spawn_span = group_data.min_4 as i32 / 4;
        }),
    },
    Ability {
        identity: Identity::Explosion,
        talent_id: Some(67),
        icon_id: Some(img015::ICON_EXPLOSION),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.explosion_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.explosion_chance), AttrUnit::Percent),
                    ("Min Range", AttrValue::Finite(stats.explosion_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.explosion_spawn_anchor + stats.explosion_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.explosion_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, group_data| {
            stats.explosion_chance += chance;
            stats.explosion_spawn_anchor = group_data.min_2 as i32 / 4;
            stats.explosion_spawn_span = group_data.min_3 as i32 / 4;
        }),
    },
    Ability {
        identity: Identity::SavageBlow,
        talent_id: Some(50),
        icon_id: Some(img015::ICON_SAVAGE_BLOW),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.savage_blow_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.savage_blow_chance), AttrUnit::Percent),
                    ("Boost", AttrValue::Finite(stats.savage_blow_boost), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, boost, _| {
            stats.savage_blow_chance += chance;
            if boost > 0 { stats.savage_blow_boost = boost; }
        }),
    },
    Ability {
        identity: Identity::CriticalHit,
        talent_id: Some(13),
        icon_id: Some(img015::ICON_CRITICAL_HIT),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.critical_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.critical_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.critical_chance += chance),
    },
    Ability {
        identity: Identity::Strengthen,
        talent_id: Some(10),
        icon_id: Some(img015::ICON_STRENGTHEN),
        name: "",
        description: "",
        schema: &[("HP", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.strengthen_threshold > 0 {
                vec![
                    ("HP", AttrValue::Finite(stats.strengthen_threshold), AttrUnit::Percent),
                    ("Boost", AttrValue::Finite(stats.strengthen_boost), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, threshold, boost, _| {
            if stats.strengthen_boost == 0 {
                stats.strengthen_threshold = 100 - threshold;
                stats.strengthen_boost = boost;
            } else {
                stats.strengthen_boost += if threshold != 0 { threshold } else { boost };
            }
        }),
    },
    Ability {
        identity: Identity::Survive,
        talent_id: Some(11),
        icon_id: Some(img015::ICON_SURVIVE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.survive > 0 {
                vec![("Chance", AttrValue::Finite(stats.survive), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.survive += chance),
    },
    Ability {
        identity: Identity::Dodge,
        talent_id: Some(51),
        icon_id: Some(img015::ICON_DODGE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.dodge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.dodge_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.dodge_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, _| { stats.dodge_chance += chance; stats.dodge_duration += duration; }),
    },
    Ability {
        identity: Identity::Weaken,
        talent_id: Some(1),
        icon_id: Some(img015::ICON_WEAKEN),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Reduced To", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.weaken_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.weaken_chance), AttrUnit::Percent),
                    ("Reduced To", AttrValue::Finite(stats.weaken_to), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.weaken_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.weaken_chance == 0 {
                stats.weaken_chance = chance;
                stats.weaken_duration = duration;
                stats.weaken_to = (100 - group_data.min_3) as i32;
            } else if group_data.text_id == 42 {
                stats.weaken_duration += get_dur_val(chance, duration);
            } else {
                stats.weaken_chance += chance;
                stats.weaken_duration += duration;
            }
        }),
    },
    Ability {
        identity: Identity::Freeze,
        talent_id: Some(2),
        icon_id: Some(img015::ICON_FREEZE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.freeze_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.freeze_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.freeze_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.freeze_chance == 0 {
                stats.freeze_chance = chance;
                stats.freeze_duration = duration;
            } else if group_data.text_id == 74 {
                stats.freeze_chance += chance;
            } else {
                stats.freeze_duration += get_dur_val(chance, duration);
            }
        }),
    },
    Ability {
        identity: Identity::Slow,
        talent_id: Some(3),
        icon_id: Some(img015::ICON_SLOW),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.slow_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.slow_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.slow_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.slow_chance == 0 {
                stats.slow_chance = chance;
                stats.slow_duration = duration;
            } else if group_data.text_id == 63 {
                stats.slow_chance += chance;
            } else {
                stats.slow_duration += get_dur_val(chance, duration);
            }
        }),
    },
    Ability {
        identity: Identity::Knockback,
        talent_id: Some(8),
        icon_id: Some(img015::ICON_KNOCKBACK),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.knockback_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.knockback_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.knockback_chance += chance),
    },
    Ability {
        identity: Identity::Curse,
        talent_id: Some(60),
        icon_id: Some(img015::ICON_CURSE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.curse_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.curse_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.curse_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.curse_chance == 0 {
                stats.curse_chance = chance;
                stats.curse_duration = duration;
            } else if group_data.text_id == 93 {
                stats.curse_duration += get_dur_val(chance, duration);
            } else {
                stats.curse_chance += chance;
            }
        }),
    },
    Ability {
        identity: Identity::Warp,
        talent_id: Some(9),
        icon_id: Some(img015::ICON_WARP),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames), ("Min Distance", AttrUnit::Range), ("Max Distance", AttrUnit::Range)],
        attributes: |stats| {
            if stats.warp_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.warp_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.warp_duration), AttrUnit::Frames),
                    ("Min Distance", AttrValue::Finite(stats.warp_distance_minimum), AttrUnit::Range),
                    ("Max Distance", AttrValue::Finite(stats.warp_distance_maximum), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Unknown,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.has_unknown_abilities),
        apply_talent: None,
    },
    Ability {
        identity: Identity::Barrier,
        talent_id: None,
        icon_id: Some(img015::ICON_BARRIER),
        name: "",
        description: "",
        schema: &[("Hitpoints", AttrUnit::None)],
        attributes: |stats| {
            if stats.barrier_hitpoints > 0 {
                vec![("Hitpoints", AttrValue::Finite(stats.barrier_hitpoints), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::AkuShield,
        talent_id: None,
        icon_id: Some(img015::ICON_SHIELD),
        name: "",
        description: "",
        schema: &[("Hitpoints", AttrUnit::None), ("Regen", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.shield_hitpoints > 0 {
                vec![
                    ("Hitpoints", AttrValue::Finite(stats.shield_hitpoints), AttrUnit::None),
                    ("Regen", AttrValue::Finite(stats.shield_regen), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Burrow,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Count", AttrUnit::None), ("Distance", AttrUnit::Range)],
        attributes: |stats| {
            if stats.burrow_amount != 0 {
                vec![
                    ("Count", AttrValue::from_sentinel(stats.burrow_amount), AttrUnit::None),
                    ("Distance", AttrValue::Finite(stats.burrow_distance), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Revive,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Count", AttrUnit::None), ("Duration", AttrUnit::Frames), ("Hitpoints", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.revive_count != 0 {
                vec![
                    ("Count", AttrValue::from_sentinel(stats.revive_count), AttrUnit::None),
                    ("Duration", AttrValue::Finite(stats.revive_time), AttrUnit::Frames),
                    ("Hitpoints", AttrValue::Finite(stats.revive_hp), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Toxic,
        talent_id: None,
        icon_id: Some(img015::ICON_TOXIC),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Damage", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.toxic_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.toxic_chance), AttrUnit::Percent),
                    ("Damage", AttrValue::Finite(stats.toxic_damage), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Drain,
        talent_id: None,
        icon_id: Some(img015::ICON_DRAIN),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Amount", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.drain_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.drain_chance), AttrUnit::Percent),
                    ("Amount", AttrValue::Finite(stats.drain_percent), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::DeathSurge,
        talent_id: None,
        icon_id: Some(img015::ICON_DEATH_SURGE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.death_surge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.death_surge_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.death_surge_level), AttrUnit::None),
                    ("Min Range", AttrValue::Finite(stats.death_surge_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.death_surge_spawn_anchor + stats.death_surge_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.death_surge_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::ImmuneWave,
        talent_id: Some(48),
        icon_id: Some(img015::ICON_IMMUNE_WAVE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.wave_immune),
        apply_talent: Some(|stats,_,_,_| stats.wave_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneSurge,
        talent_id: Some(55),
        icon_id: Some(img015::ICON_IMMUNE_SURGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.surge_immune),
        apply_talent: Some(|stats,_,_,_| stats.surge_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneExplosion,
        talent_id: Some(69),
        icon_id: Some(img015::ICON_IMMUNE_EXPLOSION),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.explosion_immune),
        apply_talent: Some(|stats,_,_,_| stats.explosion_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneWeaken,
        talent_id: Some(44),
        icon_id: Some(img015::ICON_IMMUNE_WEAKEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.weaken_immune),
        apply_talent: Some(|stats,_,_,_| stats.weaken_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneFreeze,
        talent_id: Some(45),
        icon_id: Some(img015::ICON_IMMUNE_FREEZE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.freeze_immune),
        apply_talent: Some(|stats,_,_,_| stats.freeze_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneSlow,
        talent_id: Some(46),
        icon_id: Some(img015::ICON_IMMUNE_SLOW),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.slow_immune),
        apply_talent: Some(|stats,_,_,_| stats.slow_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneKnockback,
        talent_id: Some(47),
        icon_id: Some(img015::ICON_IMMUNE_KNOCKBACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.knockback_immune),
        apply_talent: Some(|stats,_,_,_| stats.knockback_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneCurse,
        talent_id: Some(29),
        icon_id: Some(img015::ICON_IMMUNE_CURSE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.curse_immune),
        apply_talent: Some(|stats,_,_,_| stats.curse_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneWarp,
        talent_id: Some(49),
        icon_id: Some(img015::ICON_IMMUNE_WARP),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.warp_immune),
        apply_talent: Some(|stats,_,_,_| stats.warp_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneToxic,
        talent_id: Some(53),
        icon_id: Some(img015::ICON_IMMUNE_TOXIC),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.toxic_immune),
        apply_talent: Some(|stats,_,_,_| stats.toxic_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneBossWave,
        talent_id: None,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| flag(stats.boss_wave_immune),
        apply_talent: Some(|stats,_,_,_| stats.boss_wave_immune = 1),
    },
    Ability {
        identity: Identity::ResistWeaken,
        talent_id: Some(18),
        icon_id: Some(img015::ICON_RESIST_WEAKEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistFreeze,
        talent_id: Some(19),
        icon_id: Some(img015::ICON_RESIST_FREEZE),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistSlow,
        talent_id: Some(20),
        icon_id: Some(img015::ICON_RESIST_SLOW),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistKnockback,
        talent_id: Some(21),
        icon_id: Some(img015::ICON_RESIST_KNOCKBACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistWave,
        talent_id: Some(22),
        icon_id: Some(img015::ICON_RESIST_WAVE),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistWarp,
        talent_id: Some(24),
        icon_id: Some(img015::ICON_RESIST_WARP),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistCurse,
        talent_id: Some(30),
        icon_id: Some(img015::ICON_RESIST_CURSE),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistToxic,
        talent_id: Some(52),
        icon_id: Some(img015::ICON_RESIST_TOXIC),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistSurge,
        talent_id: Some(54),
        icon_id: Some(img015::ICON_SURGE_RESIST),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::CostDown,
        talent_id: Some(25),
        icon_id: Some(img015::ICON_COST_DOWN),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, reduction, _, _| stats.eoc1_cost = stats.eoc1_cost.saturating_sub(reduction)),
    },
    Ability {
        identity: Identity::RecoverSpeedUp,
        talent_id: Some(26),
        icon_id: Some(img015::ICON_RECOVER_SPEED_UP),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, frames, _, _| stats.cooldown = stats.cooldown.saturating_sub(frames)),
    },
    Ability {
        identity: Identity::MoveSpeedUp,
        talent_id: Some(27),
        icon_id: Some(img015::ICON_MOVE_SPEED),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, speed, _, _| stats.speed += speed),
    },
    Ability {
        identity: Identity::AttackBuff,
        talent_id: Some(31),
        icon_id: Some(img015::ICON_ATTACK_BUFF),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, percent, _, _| {
            let percentage_factor = (100 + percent) as f32 / 100.0;
            stats.attack_1 = (stats.attack_1 as f32 * percentage_factor).round() as i32;
            stats.attack_2 = (stats.attack_2 as f32 * percentage_factor).round() as i32;
            stats.attack_3 = (stats.attack_3 as f32 * percentage_factor).round() as i32;
        }),
    },
    Ability {
        identity: Identity::HealthBuff,
        talent_id: Some(32),
        icon_id: Some(img015::ICON_HEALTH_BUFF),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, percent, _, _| {
            let percentage_factor = (100 + percent) as f32 / 100.0;
            stats.hitpoints = (stats.hitpoints as f32 * percentage_factor).round() as i32;
        }),
    },
    Ability {
        identity: Identity::TbaDown,
        talent_id: Some(61),
        icon_id: Some(img015::ICON_TBA_DOWN),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, percent, _, _| {
            let time_reduction = (stats.attack_cooldown as f32 * percent as f32 / 100.0).round() as i32;
            stats.attack_cooldown = stats.attack_cooldown.saturating_sub(time_reduction);
        }),
    },
    Ability {
        identity: Identity::ImproveKnockbacks,
        talent_id: Some(28),
        icon_id: Some(img015::ICON_IMPROVE_KNOCKBACK_COUNT),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, count, _, _| stats.knockbacks += count),
    },
];
