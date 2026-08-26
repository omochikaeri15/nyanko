//! Public facade for Cat data.
//!
//! This module re-exports the aggregate [`Unit`] structure, the shared table
//! bundle used to build it, and every specialized parser and error type behind
//! it, so callers never reach into the internal submodule layout. It also
//! carries the combo tables: [`NyancomboData`] for the lineups that trigger a
//! combo, [`Nyancombo`] for the localized text naming one, and
//! [`NyancomboParam`] for the magnitude each effect awards.

pub mod unit;

pub use crate::common::tools::file::Separator;
pub use unit::unitid;
pub use unit::{
    AssembleError, ComboSlot, ComboStrength, EvolveMaterial, LevelCurve, LevelError, Nyancombo,
    NyancomboData, NyancomboDataError, NyancomboError, NyancomboParam, NyancomboParamError,
    SkillAcquisitionError, SkillDescriptions, SkillDescriptionsError, SkillLevelError, Tables,
    Talent, TalentCost, TalentGroup, Unit, UnitBuy, UnitBuyError, UnitEvolve, UnitEvolveError,
    UnitExplanation, UnitExplanationError,
};
