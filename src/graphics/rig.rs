//! Parsing and geometric analysis of a unit's animation rig.
//!
//! A unit's appearance is defined by three files: the texture atlas, its sprite
//! cut list, and the model hierarchy. This module turns them into a single
//! [`Rig`], and parses the separate animation timelines that drive it.

mod imgcut;
mod maanim;
mod mamodel;

use std::error;
use std::fmt;

use super::engine::{timeline, transform};
use super::tools::{boundary, periodicity};

pub use boundary::BoundingBox;
pub use imgcut::{ImgRect, ImgVec2, SpriteCut, SpriteSheet};
pub use maanim::{AnimModification, Animation, Keyframe};
pub use mamodel::{Model, ModelPart};

const DEFAULT_FRAME_CEILING: i32 = 10_000;

/// Represents errors that can occur while parsing a unit's rig or animations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RigError {
    /// The supplied bytes contained no non-empty lines.
    EmptyFile,
    /// The supplied bytes were too short to contain a complete file header.
    TruncatedHeader,
    /// No line declaring a usable model part count was found in the file's leading rows.
    NoPartHeader,
    /// The texture atlas could not be decoded, and salvage of the stream also failed.
    ImageDecodeFailed,
    /// The atlas decoded successfully but its cut list described no usable sprite regions.
    NoSpriteCuts,
}

impl fmt::Display for RigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no readable lines."),
            Self::TruncatedHeader => write!(f, "The provided bytes were too short to contain a complete header."),
            Self::NoPartHeader => write!(f, "No usable model part count was declared in the file header."),
            Self::ImageDecodeFailed => write!(f, "The texture atlas could not be decoded or salvaged."),
            Self::NoSpriteCuts => write!(f, "The sprite cut list described no usable regions."),
        }
    }
}

impl error::Error for RigError {}

/// A unit's complete visual definition: its part hierarchy and its texture atlas.
///
/// The engine splits this across three files, because the model references
/// sprite regions by index and those indices are only meaningful against the
/// atlas that accompanies it. Pairing them in one structure keeps that
/// correspondence intact for the geometry routines that consume both.
#[derive(Debug, Clone)]
pub struct Rig {
    /// The hierarchical part tree describing how the unit is assembled and animated.
    pub model: Model,
    /// The texture atlas and the sprite regions the model's parts index into.
    pub sheet: SpriteSheet,
}

/// A detected periodic interval within an animation, measured in frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cycle {
    /// The first frame of the repeating interval.
    pub start: i32,
    /// The frame at which the interval repeats, one past the last distinct frame.
    pub end: i32,
}

impl Cycle {
    /// Returns the length of the interval in frames.
    ///
    /// # Returns
    /// An `i32` containing the number of frames between the start of the
    /// interval and its repetition.
    pub fn length(&self) -> i32 {
        self.end - self.start
    }
}

impl Rig {
    /// Parses a unit's texture atlas, sprite cut list, and model hierarchy into a rig.
    ///
    /// The atlas and cut list resolve together, since sprite regions are pixel
    /// rectangles normalized against the decoded image dimensions.
    ///
    /// # Arguments
    /// * `png` - The raw bytes of the unit's PNG texture atlas.
    /// * `imgcut` - The raw bytes of the unit's `.imgcut` sprite region list.
    /// * `mamodel` - The raw bytes of the unit's `.mamodel` part hierarchy.
    ///
    /// # Returns
    /// A `Result` containing the assembled `Rig` on success, or a `RigError`
    /// identifying which of the three inputs could not be interpreted.
    pub fn parse(
        png: impl AsRef<[u8]>,
        imgcut: impl AsRef<[u8]>,
        mamodel: impl AsRef<[u8]>,
    ) -> Result<Self, RigError> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref(), mamodel.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8], mamodel: &[u8]) -> Result<Self, RigError> {
        let sheet = SpriteSheet::parse(png, imgcut)?;
        let model = Model::parse(mamodel)?;

        Ok(Self { model, sheet })
    }

    /// Calculates the smallest rectangle enclosing the rig across a set of animations.
    ///
    /// Every frame of every animation is solved and its visible geometry
    /// accumulated, so the result encloses the full range of motion rather than
    /// any single pose. Parts the tolerance judges invisible are excluded.
    ///
    /// # Arguments
    /// * `animations` - The animations to sweep. An empty slice measures the rig in its rest pose.
    /// * `tolerance` - A value from zero to one controlling how aggressively marginal parts are discarded, where zero excludes nothing.
    ///
    /// # Returns
    /// An `Option` containing the enclosing `BoundingBox`, or `None` if no part
    /// of the rig was visible in any evaluated frame.
    pub fn calculate_bounds(
        &self,
        animations: &[&Animation],
        tolerance: f32,
    ) -> Option<BoundingBox> {
        let tolerance = boundary::Tolerance::new(tolerance);
        boundary::calculate_animation_bounds(&self.model, &self.sheet, animations, tolerance)
    }

    /// Searches an animation for the shortest interval after which its pose repeats.
    ///
    /// Frames are solved in sequence and compared against every earlier frame
    /// until two match within the tolerance, detecting the true visual loop of
    /// animations whose declared keyframe range disagrees with their actual
    /// period.
    ///
    /// # Arguments
    /// * `animation` - The animation timeline to search.
    /// * `tolerance` - The maximum accumulated difference at which two frames are considered identical.
    /// * `minimum_frame` - The shortest interval to accept, which suppresses degenerate one-frame matches. Defaults to one.
    /// * `maximum_frame` - The highest frame to search before abandoning the search. Defaults to an internal ceiling.
    /// * `progress_callback` - Invoked with each frame index as it is evaluated; returning `false` abandons the search.
    ///
    /// # Returns
    /// An `Option` containing the detected `Cycle`, or `None` if the animation
    /// did not repeat within the frame limit or the callback abandoned the
    /// search.
    pub fn calculate_cycle(
        &self,
        animation: &Animation,
        tolerance: f32,
        minimum_frame: Option<i32>,
        maximum_frame: Option<i32>,
        mut progress_callback: impl FnMut(usize) -> bool,
    ) -> Option<Cycle> {
        let mut frame_states: Vec<Vec<([f32; 9], f32)>> = Vec::new();
        let mut state_buffer = self.model.parts.clone();
        let mut current_frame = 0;

        let minimum_loop_length = minimum_frame.unwrap_or(1);
        let frame_ceiling = maximum_frame.unwrap_or(DEFAULT_FRAME_CEILING);

        loop {
            if !progress_callback(current_frame) {
                return None;
            }

            if current_frame as i64 > frame_ceiling as i64 {
                return None;
            }

            let frame_float = current_frame as f32;
            let _ = timeline::animate(&self.model, animation, frame_float, &mut state_buffer);
            let world_parts = transform::solve_hierarchy(&state_buffer, &self.model);

            let mut current_state = Vec::with_capacity(world_parts.len());
            for part in &world_parts {
                current_state.push((part.matrix, part.opacity));
            }

            for (past_frame_index, past_state) in frame_states.iter().enumerate() {
                let loop_length = current_frame as i32 - past_frame_index as i32;

                if loop_length < minimum_loop_length {
                    continue;
                }

                let difference = periodicity::calculate_difference(&current_state, past_state);

                if difference <= tolerance {
                    return Some(Cycle {
                        start: past_frame_index as i32,
                        end: current_frame as i32,
                    });
                }
            }

            frame_states.push(current_state);
            current_frame += 1;
        }
    }
}
