//! Frame resolution for a parsed rig.
//!
//! This module drives a [`Rig`] through an animation and flattens the result
//! into renderer-ready geometry, so that a caller supplies no rendering backend
//! of its own beyond the ability to draw textured triangles.

pub mod construct;
pub mod timeline;
pub mod transform;

use super::rig::{Animation, Rig};

/// One part of a rig, resolved into renderer-ready geometry for a single frame.
///
/// The payload is deliberately backend-agnostic: it carries flat coordinate
/// arrays and a sprite index rather than any texture handle or draw call, so any
/// canvas implementation can consume it without the engine knowing about that
/// canvas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameData {
    /// The index of the sprite region in the rig's atlas that this part draws.
    pub sprite_index: usize,
    /// The resolved world transform as a row-major three-by-three affine matrix.
    pub final_matrix: [f32; 9],
    /// The part's corner positions as six consecutive x and y pairs forming two triangles.
    pub vertices: [f32; 12],
    /// The texture coordinates matching `vertices`, as six consecutive u and v pairs.
    pub uvs: [f32; 12],
    /// The part's resolved opacity, from fully transparent at zero to fully opaque at one.
    pub opacity: f32,
    /// The additive blending mode to draw the part with, where zero is ordinary alpha blending.
    pub glow: u8,
}

/// Resolves a rig at a single frame into renderer-ready geometry.
///
/// The animation is applied to a copy of the rest pose, the hierarchy solved so
/// each part inherits its ancestors' transforms, and the result flattened into
/// one entry per drawable part in draw order.
///
/// # Arguments
/// * `rig` - The parsed rig supplying the part hierarchy and sprite atlas.
/// * `anim` - The animation to evaluate, or `None` to resolve the rig in its rest pose.
/// * `frame` - The frame to evaluate at, which may be fractional to sample between keyframes.
///
/// # Returns
/// A `Vec<FrameData>` containing one entry per drawable part, ordered so that
/// drawing them in sequence produces the correct depth layering.
pub fn resolve_frame(
    rig: &Rig,
    anim: Option<&Animation>,
    frame: f32,
) -> Vec<FrameData> {
    let parts = if let Some(animation) = anim {
        let mut state_buffer = rig.model.parts.clone();
        let _ = timeline::animate(&rig.model, animation, frame, &mut state_buffer);
        state_buffer
    } else {
        rig.model.parts.clone()
    };

    let world_parts = transform::solve_hierarchy(&parts, &rig.model);

    construct::build_geometry(&world_parts, &rig.sheet)
}
