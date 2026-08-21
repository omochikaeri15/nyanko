//! Measurement of the screen area a rig occupies across its animations.

use crate::graphics::animate::resolve_frame;
use crate::graphics::rig::{Animation, Rig};

/// The opacity below which a part contributes nothing whatever the tolerance says.
const INVISIBLE: f32 = 0.01;

/// An axis-aligned rectangle enclosing a region of world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    /// The leftmost extent.
    pub min_x: f32,
    /// The topmost extent.
    pub min_y: f32,
    /// The rightmost extent.
    pub max_x: f32,
    /// The bottommost extent.
    pub max_y: f32,
}

impl BoundingBox {
    /// Returns the horizontal extent of the rectangle.
    ///
    /// # Returns
    /// An `f32` containing the distance between the left and right bounds.
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Returns the vertical extent of the rectangle.
    ///
    /// # Returns
    /// An `f32` containing the distance between the top and bottom bounds.
    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    /// Returns the midpoint of the rectangle.
    ///
    /// # Returns
    /// An `(f32, f32)` holding the horizontal and vertical center of the enclosed region.
    pub fn center(&self) -> (f32, f32) {
        ((self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0)
    }

    /// Returns the smallest rectangle enclosing both this rectangle and another.
    ///
    /// # Arguments
    /// * `other` - The rectangle to combine with this one.
    ///
    /// # Returns
    /// A `BoundingBox` enclosing the full extent of both inputs.
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

/// The thresholds deciding which parts count towards a measured bounding box.
///
/// The engine's assets contain parts that are present but visually irrelevant,
/// such as transparent markers or parts scaled off screen. Each threshold
/// excludes one such case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    /// The opacity below which an ordinary part is treated as invisible.
    pub minimum_opacity: f32,
    /// The opacity below which an additively blended part is treated as invisible.
    pub minimum_glow_opacity: f32,
    /// The scale factor above which a part is treated as a degenerate outlier.
    pub maximum_scale: f32,
    /// The opacity below which a heavily scaled part is discarded rather than measured.
    pub scale_opacity_threshold: f32,
    /// The ratio of vertical to horizontal extent above which a part is treated as degenerate.
    pub maximum_vertical_stretch: f32,
    /// The rendered height above which a part is treated as a degenerate outlier.
    pub maximum_height_threshold: f32,
    /// The vertical position below which a part is considered off screen.
    pub minimum_y_bound: f32,
}

impl Tolerance {
    /// Derives a full threshold set from a single strictness level.
    ///
    /// Zero admits nearly everything; one discards parts that are only
    /// marginally visible.
    ///
    /// # Arguments
    /// * `level` - The strictness from zero to one, which is clamped into that range.
    ///
    /// # Returns
    /// A `Tolerance` populated with thresholds matching the requested level.
    pub fn new(level: f32) -> Self {
        let level = level.clamp(0.0, 1.0);
        let inverse = 1.0 - level;

        Self {
            minimum_opacity: 0.01 + (0.24 * level),
            minimum_glow_opacity: 0.75 * level,
            maximum_scale: 3.0 + (inverse * 100.0),
            scale_opacity_threshold: 0.95 * level,
            maximum_vertical_stretch: 2.0 + (inverse * 50.0),
            maximum_height_threshold: 1000.0 + (inverse * 10000.0),
            minimum_y_bound: -1200.0 - (inverse * 10000.0),
        }
    }
}

/// Measures the combined bounding box of a rig across several animations.
///
/// # Arguments
/// * `rig` - The rig to measure.
/// * `animations` - The animations to sweep.
/// * `tolerance` - The thresholds deciding which parts count towards the result.
///
/// # Returns
/// An `Option` containing the combined `BoundingBox`, or `None` if no animation
/// produced a single measurable frame.
pub fn calculate_animation_bounds(
    rig: &Rig,
    animations: &[&Animation],
    tolerance: Tolerance,
) -> Option<BoundingBox> {
    animations.iter().fold(None, |combined, animation| {
        let measured = scan_bounds(rig, Some(animation), tolerance, None);

        match (combined, measured) {
            (Some(combined), Some(measured)) => Some(combined.union(&measured)),
            (combined, measured) => combined.or(measured),
        }
    })
}

/// Measures the bounding box of a rig across one animation's frame range.
///
/// # Arguments
/// * `rig` - The rig to measure.
/// * `animation` - The animation to sweep, or `None` to measure the resting pose alone.
/// * `tolerance` - The thresholds deciding which parts count towards the result.
/// * `override_range` - An explicit inclusive frame range to scan, replacing the animation's own playback range.
///
/// # Returns
/// An `Option` containing the measured `BoundingBox`, or `None` if no part was
/// visible in any scanned frame.
pub fn scan_bounds(
    rig: &Rig,
    animation: Option<&Animation>,
    tolerance: Tolerance,
    override_range: Option<(i32, i32)>,
) -> Option<BoundingBox> {
    let (start, end) = override_range.unwrap_or_else(|| {
        (0, animation.map_or(0, |animation| animation.playback_frames().saturating_sub(1)))
    });

    let mut bounds: Option<BoundingBox> = None;

    for frame in start..=end {
        for part in resolve_frame(rig, animation, frame) {
            if part.opacity <= INVISIBLE || part.opacity < tolerance.minimum_opacity { continue; }
            if part.glow > 0 && part.opacity < tolerance.minimum_glow_opacity { continue; }

            let Some(cut) = rig.sheet.cuts.get(part.sprite_index) else { continue };

            let corner = |at: usize| (part.vertices[2 * at], part.vertices[2 * at + 1]);
            let span = |from: usize, to: usize| {
                let ((from_x, from_y), (to_x, to_y)) = (corner(from), corner(to));
                (to_x - from_x).hypot(to_y - from_y)
            };

            let scale_x = if cut.width == 0 { 0.0 } else { span(0, 1) / cut.width as f32 };
            let scale_y = if cut.height == 0 { 0.0 } else { span(0, 2) / cut.height as f32 };

            if scale_x.max(scale_y) > tolerance.maximum_scale
                && (part.opacity < tolerance.scale_opacity_threshold || part.glow > 0) {
                continue;
            }

            let mut measured = BoundingBox { min_x: f32::MAX, min_y: f32::MAX, max_x: f32::MIN, max_y: f32::MIN };

            for at in [0, 1, 2, 5] {
                let (x, y) = corner(at);

                measured.min_x = measured.min_x.min(x);
                measured.max_x = measured.max_x.max(x);
                measured.min_y = measured.min_y.min(y);
                measured.max_y = measured.max_y.max(y);
            }

            if measured.height() > tolerance.maximum_height_threshold
                && measured.height() > measured.width() * tolerance.maximum_vertical_stretch {
                continue;
            }

            if measured.max_y < tolerance.minimum_y_bound { continue; }

            bounds = Some(bounds.map_or(measured, |bounds| bounds.union(&measured)));
        }
    }

    bounds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_encloses_both_inputs() {
        let first = BoundingBox { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        let second = BoundingBox { min_x: 5.0, min_y: -5.0, max_x: 15.0, max_y: 5.0 };

        let combined = first.union(&second);

        assert_eq!(combined, BoundingBox { min_x: 0.0, min_y: -5.0, max_x: 15.0, max_y: 10.0 });
        assert_eq!(combined.width(), 15.0);
        assert_eq!(combined.height(), 15.0);
        assert_eq!(combined.center(), (7.5, 2.5));
    }

    #[test]
    fn tolerance_clamps_its_level() {
        let strict = Tolerance::new(1.0);

        assert_eq!(strict.minimum_opacity, 0.25);
        assert_eq!(strict.maximum_scale, 3.0);
        assert_eq!(Tolerance::new(5.0), strict);
    }
}
