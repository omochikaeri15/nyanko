//! Animation and rendering support for unit graphics.
//!
//! This module turns the engine's raw animation assets into geometry any canvas
//! implementation can draw, without itself depending on a rendering backend.
//! Parsing lives in [`rig`], the game's own animation logic in `engine`, the
//! geometry a caller consumes in [`animate`], the mapping from that geometry back
//! to the parts that drew it in [`part`], and the measurement and comparison
//! helpers in [`tools`].
//!
//! This module requires the `graphics` feature.

pub mod animate;
mod engine;
pub mod part;
pub mod rig;
pub mod tools;
