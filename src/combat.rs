//! Shared combat statistics for both Cat and Enemy entities.
//!
//! Both factions describe their units with the same set of mechanical
//! properties, laid out differently in their respective source files. This
//! module holds the unified [`Entity`] both are parsed into, along with the
//! registry that recognizes which abilities a given entity carries.

mod abilities;
pub(crate) mod entity;

pub use abilities::{Ability, AttrUnit, AttrValue, Attribute, Identity, REGISTRY, get_ability, get_talent};
pub use entity::{Entity, EntityError, Faction};
