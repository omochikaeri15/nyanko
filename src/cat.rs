//! Public facade for Cat unit data.
//!
//! This module re-exports the aggregate [`Unit`] structure, the shared table
//! bundle used to build it, and every specialized parser and error type behind
//! it, so callers never reach into the internal submodule layout.

pub mod unit;

pub use crate::common::tools::file::Separator;
pub use unit::unitid;
pub use unit::{
    AssembleError, EvolveMaterial, LevelCurve, LevelError, SkillAcquisitionError,
    SkillDescriptions, SkillDescriptionsError, SkillLevelError, Tables, Talent, TalentCost,
    TalentGroup, Unit, UnitBuy, UnitBuyError, UnitEvolve, UnitEvolveError, UnitExplanation,
    UnitExplanationError,
};
