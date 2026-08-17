//! Animation and rendering support for unit graphics.
//!
//! This module turns the engine's raw animation assets into geometry any canvas
//! implementation can draw, without itself depending on a rendering backend.
//! Parsing lives in [`rig`], frame resolution in [`engine`], and the measurement
//! and comparison helpers in [`tools`].
//!
//! This module requires the `graphics` feature.

pub mod engine;
pub mod rig;
pub mod tools;
