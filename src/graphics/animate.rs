//! The geometry a caller consumes, and the one call that produces it.
//!
//! This is the whole front of the graphics module. It holds no animation logic
//! of its own: `engine` poses the parts and flattens them into geometry, and
//! this places the result where the caller asked for it.

use super::engine;
use super::rig::{Animation, Rig};

/// One part of a rig, resolved into renderer-ready geometry for a single frame.
///
/// The payload is deliberately backend-agnostic: it carries flat coordinate
/// arrays and a sprite index rather than any texture handle or draw call, so any
/// canvas implementation can consume it without the engine knowing about that
/// canvas.
///
/// The engine resolves a part all the way to four world-space corners and rounds
/// them to whole pixels, so `vertices` already holds that result and
/// `final_matrix` is the identity. It is kept so a consumer can still impose a
/// per-part transform of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameData {
    /// The index of the sprite region in the rig's atlas that this part draws.
    pub sprite_index: usize,
    /// The identity, since the engine bakes each part's transform into `vertices`.
    pub final_matrix: [f32; 9],
    /// The part's corner positions as six consecutive x and y pairs forming two triangles.
    pub vertices: [f32; 12],
    /// The texture coordinates matching `vertices`, as six consecutive u and v pairs.
    pub uvs: [f32; 12],
    /// The part's resolved opacity, from fully transparent at zero to fully opaque at one.
    pub opacity: f32,
    /// The blending mode to draw the part with, where zero is ordinary alpha blending.
    pub glow: u8,
}

impl Default for FrameData {
    fn default() -> Self {
        Self {
            sprite_index: 0,
            final_matrix: engine::IDENTITY,
            vertices: [0.0; 12],
            uvs: [0.0; 12],
            opacity: 0.0,
            glow: 0,
        }
    }
}

/// Resolves a rig at a single frame into renderer-ready geometry.
///
/// # Arguments
/// * `rig` - The parsed rig supplying the part hierarchy and sprite atlas.
/// * `anim` - The animation to evaluate, or `None` to resolve the rig in its resting pose.
/// * `frame` - The frame to evaluate at. The engine has no notion of a fractional frame.
/// * `offset` - The index of the alignment row placing the rig, where zero is combat, or `None` to leave the rig at the engine's own origin.
///
/// # Returns
/// A `Vec<FrameData>` containing one entry per drawn part, ordered so that
/// drawing them in sequence produces the correct depth layering.
pub fn resolve_frame(
    rig: &Rig,
    anim: Option<&Animation>,
    frame: i32,
    offset: Option<usize>,
) -> Vec<FrameData> {
    let parts = engine::resolve(&rig.model, anim, frame, &rig.sheet);
    let mut frames = engine::build(&parts, rig);

    if let Some(alignment) = offset.and_then(|row| rig.model.alignment.get(row)) {
        shift(&mut frames, alignment.x, alignment.y);
    }

    frames
}

/// Translates every vertex by an alignment row, which states its offset negated.
fn shift(frames: &mut [FrameData], x: i32, y: i32) {
    let (x, y) = (-(x as f32), -(y as f32));

    for frame in frames {
        for corner in frame.vertices.chunks_exact_mut(2) {
            corner[0] += x;
            corner[1] += y;
        }
    }
}
