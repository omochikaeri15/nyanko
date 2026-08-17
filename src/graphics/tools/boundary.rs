//! Measurement of the screen area a rig occupies across its animations.

use crate::graphics::rig::SpriteSheet;
use crate::graphics::rig::Animation;
use crate::graphics::rig::Model;
use crate::graphics::engine::timeline;
use crate::graphics::engine::transform::{self, Vector};

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
    /// A `Vector` positioned at the center of the enclosed region.
    pub fn center(&self) -> Vector {
        Vector {
            x: (self.min_x + self.max_x) / 2.0,
            y: (self.min_y + self.max_y) / 2.0,
        }
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
/// such as transparent markers or parts scaled off-screen. Each threshold
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
    /// The ratio of vertical to horizontal scale above which a part is treated as degenerate.
    pub maximum_vertical_stretch: f32,
    /// The rendered height above which a part is treated as a degenerate outlier.
    pub maximum_height_threshold: f32,
    /// The vertical position below which a part is considered off-screen.
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
        let clamped_level = level.clamp(0.0, 1.0);
        let inverse_level = 1.0 - clamped_level;

        Self {
            minimum_opacity: 0.01 + (0.24 * clamped_level),
            minimum_glow_opacity: 0.75 * clamped_level,
            maximum_scale: 3.0 + (inverse_level * 100.0),
            scale_opacity_threshold: 0.95 * clamped_level,
            maximum_vertical_stretch: 2.0 + (inverse_level * 50.0),
            maximum_height_threshold: 1000.0 + (inverse_level * 10000.0),
            minimum_y_bound: -1200.0 - (inverse_level * 10000.0),
        }
    }
}

/// Measures the combined bounding box of a model across several animations.
///
/// Each animation is scanned independently and the results combined.
///
/// # Arguments
/// * `model` - The rest-pose hierarchy to measure.
/// * `sheet` - The atlas supplying the sprite dimensions each part draws at.
/// * `animations` - The animations to sweep.
/// * `tolerance` - The thresholds deciding which parts count towards the result.
///
/// # Returns
/// An `Option` containing the combined `BoundingBox`, or `None` if no animation
/// produced a single measurable frame.
pub fn calculate_animation_bounds(
    model: &Model,
    sheet: &SpriteSheet,
    animations: &[&Animation],
    tolerance: Tolerance,
) -> Option<BoundingBox> {
    let mut master_bounds: Option<BoundingBox> = None;

    for loaded_animation in animations {
        let new_bounds_option = scan_bounds(model, Some(*loaded_animation), sheet, tolerance, None);

        let Some(new_bounds) = new_bounds_option else {
            continue;
        };

        master_bounds = Some(match master_bounds {
            Some(existing_bounds) => existing_bounds.union(&new_bounds),
            None => new_bounds,
        });
    }

    master_bounds
}

/// Measures the bounding box of a model across one animation's frame range.
///
/// Every frame in the range is posed and its visible parts accumulated; parts
/// the tolerance rejects are excluded.
///
/// # Arguments
/// * `model` - The rest-pose hierarchy to measure.
/// * `animation` - The animation to sweep, or `None` to measure the rest pose alone.
/// * `sheet` - The atlas supplying the sprite dimensions each part draws at.
/// * `tolerance` - The thresholds deciding which parts count towards the result.
/// * `override_range` - An explicit inclusive frame range to scan, replacing the animation's own declared length.
///
/// # Returns
/// An `Option` containing the measured `BoundingBox`, or `None` if no part was
/// visible in any scanned frame.
pub fn scan_bounds(
    model: &Model,
    animation: Option<&Animation>,
    sheet: &SpriteSheet,
    tolerance: Tolerance,
    override_range: Option<(i32, i32)>,
) -> Option<BoundingBox> {
    let mut minimum_x = f32::MAX;
    let mut minimum_y = f32::MAX;
    let mut maximum_x = f32::MIN;
    let mut maximum_y = f32::MIN;
    let mut found_any_valid_parts = false;

    let (start_frame, end_frame) = match override_range {
        Some(range) => range,
        None => match animation {
            Some(loaded_anim) => (0, loaded_anim.max_frame),
            None => (0, 0),
        },
    };

    let mut state_buffer = model.parts.clone();

    for frame_index in start_frame..=end_frame {
        let current_frame = frame_index as f32;

        let posed_parts = match animation {
            Some(loaded_anim) => {
                let _ = timeline::animate(model, loaded_anim, current_frame, &mut state_buffer);
                &state_buffer
            }
            None => &model.parts,
        };

        let world_parts = transform::solve_hierarchy(posed_parts, model);

        for part in world_parts {
            if part.opacity <= 0.01 || part.hidden {
                continue;
            }
            
            if part.opacity < tolerance.minimum_opacity {
                continue;
            }

            if part.glow > 0 && part.opacity < tolerance.minimum_glow_opacity {
                continue;
            }

            let scale_x = (part.matrix[0].powi(2) + part.matrix[1].powi(2)).sqrt();
            let scale_y = (part.matrix[3].powi(2) + part.matrix[4].powi(2)).sqrt();
            let maximum_scale = scale_x.max(scale_y);

            if maximum_scale > tolerance.maximum_scale && (part.opacity < tolerance.scale_opacity_threshold || part.glow > 0) {
                continue;
            }
            
            let Some(cut) = sheet.cuts_map.get(&part.sprite_index) else {
                continue;
            };

            let sprite_width = cut.original_size.x;
            let sprite_height = cut.original_size.y;
            let pivot_x = part.pivot.x;
            let pivot_y = part.pivot.y;

            let local_corners = [
                Vector { x: -pivot_x, y: -pivot_y },
                Vector { x: sprite_width - pivot_x, y: -pivot_y },
                Vector { x: sprite_width - pivot_x, y: sprite_height - pivot_y },
                Vector { x: -pivot_x, y: sprite_height - pivot_y },
            ];

            let transform_matrix = part.matrix;
            let mut part_minimum_x = f32::MAX;
            let mut part_minimum_y = f32::MAX;
            let mut part_maximum_x = f32::MIN;
            let mut part_maximum_y = f32::MIN;

            for point in local_corners {
                let world_x = point.x * transform_matrix[0] + point.y * transform_matrix[3] + transform_matrix[6];
                let world_y = point.x * transform_matrix[1] + point.y * transform_matrix[4] + transform_matrix[7];

                part_minimum_x = part_minimum_x.min(world_x);
                part_maximum_x = part_maximum_x.max(world_x);
                part_minimum_y = part_minimum_y.min(world_y);
                part_maximum_y = part_maximum_y.max(world_y);
            }
            
            let part_total_height = part_maximum_y - part_minimum_y;
            let part_total_width = part_maximum_x - part_minimum_x;

            if part_total_height > tolerance.maximum_height_threshold && part_total_height > part_total_width * tolerance.maximum_vertical_stretch {
                continue;
            }

            if part_maximum_y < tolerance.minimum_y_bound {
                continue;
            }
            
            minimum_x = minimum_x.min(part_minimum_x);
            maximum_x = maximum_x.max(part_maximum_x);
            minimum_y = minimum_y.min(part_minimum_y);
            maximum_y = maximum_y.max(part_maximum_y);

            found_any_valid_parts = true;
        }
    }

    if found_any_valid_parts {
        Some(BoundingBox {
            min_x: minimum_x,
            min_y: minimum_y,
            max_x: maximum_x,
            max_y: maximum_y,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_union() {
        let box_a = BoundingBox { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        let box_b = BoundingBox { min_x: 5.0, min_y: -5.0, max_x: 15.0, max_y: 5.0 };

        let combined = box_a.union(&box_b);

        assert_eq!(combined.min_x, 0.0);
        assert_eq!(combined.min_y, -5.0);
        assert_eq!(combined.max_x, 15.0);
        assert_eq!(combined.max_y, 10.0);
        assert_eq!(combined.width(), 15.0);
        assert_eq!(combined.height(), 15.0);
    }

    #[test]
    fn test_tolerance_clamping() {
        let max_tolerance = Tolerance::new(1.0);
        assert_eq!(max_tolerance.minimum_opacity, 0.25);
        assert_eq!(max_tolerance.maximum_scale, 3.0);

        let clamped_tolerance = Tolerance::new(5.0);
        assert_eq!(clamped_tolerance.minimum_opacity, 0.25);
        assert_eq!(clamped_tolerance.maximum_scale, 3.0);
    }
}