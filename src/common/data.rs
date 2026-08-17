//! Static lookup tables and shared data files.
//!
//! These modules cover values the engine addresses by index or key rather than
//! storing inline, such as icon atlas positions and localized string tables.

mod param;
mod localizable;

pub mod img015;
pub mod img022;

pub use param::{Param, ParamError};
pub use localizable::{Localizable, LocalizableError};
