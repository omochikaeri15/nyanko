//! The geometry a caller consumes, and the one call that produces it.
//!
//! This is the whole front of the graphics module. It holds no animation logic
//! of its own: [`super::engine`] poses and places the parts, and this turns the
//! result into flat vertex arrays.

use super::engine::{self, Part, Point};
use super::rig::{Animation, Rig};

/// The highest blending mode defined.
const GLOW_MODES: i32 = 3;

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
            final_matrix: IDENTITY,
            vertices: [0.0; 12],
            uvs: [0.0; 12],
            opacity: 0.0,
            glow: 0,
        }
    }
}

/// The row-major three by three identity, which every part carries.
const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// Resolves a rig at a single frame into renderer-ready geometry.
///
/// # Arguments
/// * `rig` - The parsed rig supplying the part hierarchy and sprite atlas.
/// * `anim` - The animation to evaluate, or `None` to resolve the rig in its resting pose.
/// * `frame` - The frame to evaluate at. The engine has no notion of a fractional frame.
///
/// # Returns
/// A `Vec<FrameData>` containing one entry per drawn part, ordered so that
/// drawing them in sequence produces the correct depth layering.
pub fn resolve_frame(rig: &Rig, anim: Option<&Animation>, frame: i32) -> Vec<FrameData> {
    let parts = engine::resolve(&rig.model, anim, frame, &rig.sheet);

    build(&parts, rig)
}

/// Flattens the placed parts into one quad each, in the order they arrive.
///
/// A part the engine never draws, one naming an atlas region that does not
/// exist, and one whose opacity quantizes away to nothing are all dropped, which
/// is what the engine's own draw call does with each of them.
///
/// A blending mode the engine's table does not define leaves the mode the last
/// part set still in force, since the mode lives on the draw context rather than
/// on the part.
fn build(parts: &[Part<'_>], rig: &Rig) -> Vec<FrameData> {
    let sheet = &rig.sheet;
    let (atlas_width, atlas_height) = sheet.image_data.as_ref()
        .map_or((1.0, 1.0), |image| (image.width() as f32, image.height() as f32));

    let opacity_unit = rig.model.opacity_unit;
    let mut frames = Vec::with_capacity(parts.len());
    let mut glow = 0;

    for part in parts {
        if !part.drawn() { continue; }

        if part.glow() <= GLOW_MODES {
            glow = part.glow().max(0) as u8;
        }

        let alpha = part.alpha(opacity_unit);
        if alpha == 0 { continue; }

        let Ok(sprite) = usize::try_from(part.region()) else { continue };
        let Some(cut) = sheet.cuts.get(sprite) else { continue };

        let [top_left, bottom_left, bottom_right, top_right] = part.corners();
        let corner = |point: Point| [point.x as f32, point.y as f32];

        let near_u = cut.x as f32 / atlas_width;
        let far_u = cut.x.wrapping_add(cut.width) as f32 / atlas_width;
        let near_v = cut.y as f32 / atlas_height;
        let far_v = cut.y.wrapping_add(cut.height) as f32 / atlas_height;

        let mut vertices = [0.0; 12];
        let mut uvs = [0.0; 12];

        // The engine submits four vertices and indexes them 0 1 2, 3 2 1, so the
        // two triangles wind this way round. Expanded here because `FrameData`
        // carries no index buffer.
        for (slot, (point, texel)) in [
            (top_left, [near_u, near_v]),
            (bottom_left, [near_u, far_v]),
            (top_right, [far_u, near_v]),
            (bottom_right, [far_u, far_v]),
            (top_right, [far_u, near_v]),
            (bottom_left, [near_u, far_v]),
        ].into_iter().enumerate() {
            vertices[2 * slot..2 * slot + 2].copy_from_slice(&corner(point));
            uvs[2 * slot..2 * slot + 2].copy_from_slice(&texel);
        }

        frames.push(FrameData {
            sprite_index: sprite,
            final_matrix: IDENTITY,
            vertices,
            uvs,
            opacity: alpha as f32 / u8::MAX as f32,
            glow,
        });
    }

    frames
}
