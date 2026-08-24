use serde::Serialize;

use crate::common::tools::columns::{self, Column};
use crate::common::tools::file;
use crate::graphics::tools::math;

use super::RigError;

/// A single control point on a modification's curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub struct Keyframe {
    /// The frame at which this control point takes effect.
    pub frame: i32,
    /// The raw value the curve holds here, in the units of the modified property.
    pub value: i32,
    /// The interpolation applied between this control point and the next.
    pub ease: i32,
    /// The exponent the exponential easing raises its progress to.
    pub ease_power: i32,
}

impl Keyframe {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `.maanim` keyframe row
    /// from the parser's own table instead of restating it.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        frame      : 0;
        value      : 1;
        ease       : 2;
        ease_power : 3;
    };
}

/// A single animated property of one model part over time.
///
/// A part that moves, rotates, and fades at once is described by three separate
/// modifications sharing a part index.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AnimModification {
    /// The index of the model part this drives, which the engine does not bound check.
    pub part: i32,
    /// The property being driven, from zero for the parent index to fourteen for the vertical flip.
    pub kind: i32,
    /// How many times the curve replays, where minus one replays it forever.
    pub loop_count: i32,
    /// The lower value bound the authoring tool recorded, which the engine ignores.
    pub min_value: i32,
    /// The upper value bound the authoring tool recorded, which the engine ignores.
    pub max_value: i32,
    /// The modification's declared name.
    pub name: String,
    /// The control points defining the curve.
    pub keyframes: Vec<Keyframe>,
}

impl AnimModification {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `.maanim` modification
    /// header from the parser's own table instead of restating it.
    /// [`AnimModification::name`] is the row's trailing text rather than a
    /// column, and [`AnimModification::keyframes`] follows on later rows.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        part       : 0;
        kind       : 1;
        loop_count : 2;
        min_value  : 3;
        max_value  : 4;
    };
}

/// A complete animation timeline for a unit's rig.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Animation {
    /// The declared format version.
    pub version: i32,
    /// The property curves that make up the animation.
    pub modifications: Vec<AnimModification>,
}

impl Animation {
    /// Parses a `.maanim` byte stream into a structured animation timeline.
    ///
    /// The file opens with a tag line, a version, and a modification count.
    /// Each modification is a header row followed by its keyframe count and that
    /// many keyframe rows. Modifications that resolve to no keyframes are
    /// dropped, matching the engine's inability to address them.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the unit's `.maanim` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `Animation` on success, or a `RigError`
    /// if the file contained no readable lines.
    pub fn parse(bytes: impl AsRef<[u8]>) -> Result<Self, RigError> {
        Self::parse_inner(bytes.as_ref())
    }

    fn parse_inner(bytes: &[u8]) -> Result<Self, RigError> {
        let content = file::scrub(bytes);
        let delimiter = file::resolve(None, &content);
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();

        if lines.is_empty() { return Err(RigError::EmptyFile); }

        let mut cursor = usize::from(lines[0].trim_start().starts_with('['));

        let version = lines.get(cursor).and_then(|line| line.trim().parse().ok()).unwrap_or(0);
        cursor += 1;

        let count = lines.get(cursor).and_then(|line| line.trim().parse::<usize>().ok()).unwrap_or(0);
        cursor += 1;

        let mut modifications = Vec::with_capacity(count.min(lines.len()));

        for _ in 0..count {
            let Some(header) = lines.get(cursor) else { break };
            cursor += 1;

            let row: Vec<&str> = header.split(delimiter).collect();
            let mut modification = AnimModification::default();

            let trailing = columns::apply(&row, AnimModification::COLUMNS, &mut modification);
            modification.name = row.get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            let Some(count_line) = lines.get(cursor) else { break };
            cursor += 1;

            let keyframe_count = count_line.split(delimiter).next()
                .and_then(|text| text.trim().parse::<usize>().ok())
                .unwrap_or(0);

            let declared = keyframe_count.min(lines.len().saturating_sub(cursor));
            modification.keyframes.reserve(declared);

            for index in 0..declared {
                let row: Vec<&str> = lines[cursor + index].split(delimiter).collect();
                let mut keyframe = Keyframe::default();

                columns::apply(&row, Keyframe::COLUMNS, &mut keyframe);
                modification.keyframes.push(keyframe);
            }

            cursor += declared;
            modifications.push(modification);
        }

        Ok(Self { version, modifications })
    }

    /// Returns the number of frames the engine considers the animation to occupy.
    ///
    /// Every modification is measured across all of its replays and the longest
    /// one wins. A timeline holding any modification that replays forever has no
    /// length at all, and the engine reports minus one for it rather than a
    /// frame count.
    ///
    /// # Returns
    /// An `i32` containing the frame count, zero for a timeline with no usable
    /// modifications, or minus one for a timeline that never ends.
    pub fn length(&self) -> i32 {
        let mut longest = 0;

        for modification in &self.modifications {
            if modification.loop_count == -1 { return -1; }

            let (Some(first), Some(last)) = (modification.keyframes.first(), modification.keyframes.last()) else {
                continue;
            };

            let span = last.frame.wrapping_sub(first.frame);
            let played = modification.loop_count
                .wrapping_sub(1)
                .wrapping_mul(span)
                .wrapping_add(last.frame);

            if played >= longest { longest = played.wrapping_add(1); }
        }

        longest
    }

    /// Returns the interval after which every modification realigns with its own start.
    ///
    /// The engine wraps a replaying modification over the span between its first
    /// and last keyframe, so a timeline whose modifications all replay forever
    /// repeats over the least common multiple of those spans. A timeline holding
    /// any modification that stops has no such interval.
    ///
    /// # Returns
    /// An `Option` containing the repeat interval in frames, or `None` if any
    /// modification stops replaying or the combined interval overflows.
    pub fn period(&self) -> Option<i32> {
        let mut period = 1;

        for modification in &self.modifications {
            if modification.loop_count != -1 { return None; }

            let (Some(first), Some(last)) = (modification.keyframes.first(), modification.keyframes.last()) else {
                continue;
            };

            let span = last.frame - first.frame;
            if span <= 0 { continue; }

            period = (period / math::gcd(period, span)).checked_mul(span)?;
        }

        (period > 1).then_some(period)
    }

    /// Returns the number of frames the file itself defines.
    ///
    /// A timeline that ends occupies the length the engine measures across every
    /// replay. A timeline that never ends is measured by the furthest keyframe
    /// any modification reaches, which is the point a forever replaying
    /// modification wraps back to its first keyframe and so renders the same as
    /// frame zero, leaving the count one short of that frame.
    ///
    /// # Returns
    /// An `i32` containing the engine's length for a timeline that ends, and
    /// otherwise the furthest frame any modification reaches, which is at least
    /// one.
    pub fn declared_frames(&self) -> i32 {
        match self.length() {
            length if length > 0 => length,
            _ => self.modifications.iter()
                .filter_map(|modification| modification.keyframes.last())
                .fold(0, |furthest, keyframe| furthest.max(keyframe.frame))
                .max(1),
        }
    }

    /// Returns the number of frames to play before returning to the start.
    ///
    /// The engine never loops an animation itself: a modification that replays
    /// wraps its own keyframe range, and [`Animation::length`] reports minus one
    /// for a timeline that never ends. Anything that plays a timeline back still
    /// needs somewhere to restart, so this prefers the interval over which every
    /// modification realigns and falls back to
    /// [`Animation::declared_frames`]. That interval is the least common
    /// multiple of the replaying spans and grows far past the authored content,
    /// so anything sizing a control, a default range or a bounded sweep wants
    /// [`Animation::declared_frames`] instead.
    ///
    /// # Returns
    /// An `i32` containing the frame count, which is at least one.
    pub fn playback_frames(&self) -> i32 {
        self.period().map_or_else(|| self.declared_frames(), |period| period.max(1))
    }

    /// Measures an animation's length without building the full timeline.
    ///
    /// This walks the modification headers and their first and last keyframe
    /// rows directly, and is substantially cheaper than [`Animation::parse`]
    /// followed by [`Animation::length`], which it agrees with.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the unit's `.maanim` file.
    ///
    /// # Returns
    /// An `Option` containing the animation's length in frames, minus one for a
    /// timeline that never ends, or `None` if the file contained no readable
    /// lines.
    pub fn scan_length(bytes: impl AsRef<[u8]>) -> Option<i32> {
        Self::scan_length_inner(bytes.as_ref())
    }

    fn scan_length_inner(bytes: &[u8]) -> Option<i32> {
        let content = file::scrub(bytes);
        let delimiter = file::resolve(None, &content);
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();

        if lines.is_empty() { return None; }

        let mut cursor = usize::from(lines[0].trim_start().starts_with('['));
        cursor += 1;

        let count = lines.get(cursor).and_then(|line| line.trim().parse::<usize>().ok()).unwrap_or(0);
        cursor += 1;

        let frame_at = |index: usize| -> i32 {
            lines.get(index)
                .and_then(|line| line.split(delimiter).next())
                .and_then(|text| text.trim().parse().ok())
                .unwrap_or(0)
        };

        let mut longest = 0;

        for _ in 0..count {
            let Some(header) = lines.get(cursor) else { break };
            cursor += 1;

            let loop_count = header.split(delimiter).nth(2)
                .and_then(|text| text.trim().parse::<i32>().ok())
                .unwrap_or(0);

            if loop_count == -1 { return Some(-1); }

            let Some(count_line) = lines.get(cursor) else { break };
            cursor += 1;

            let keyframe_count = count_line.split(delimiter).next()
                .and_then(|text| text.trim().parse::<usize>().ok())
                .unwrap_or(0);

            let declared = keyframe_count.min(lines.len().saturating_sub(cursor));

            if declared > 0 {
                let (first, last) = (frame_at(cursor), frame_at(cursor + declared - 1));
                let played = loop_count
                    .wrapping_sub(1)
                    .wrapping_mul(last.wrapping_sub(first))
                    .wrapping_add(last);

                if played >= longest { longest = played.wrapping_add(1); }
            }

            cursor += declared;
        }

        Some(longest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modification(loop_count: i32, frames: &[i32]) -> AnimModification {
        AnimModification {
            loop_count,
            keyframes: frames.iter().map(|&frame| Keyframe { frame, ..Keyframe::default() }).collect(),
            ..AnimModification::default()
        }
    }

    #[test]
    fn keyframe_columns_map_one_field_each() {
        columns::assert_one_field_per_column(Keyframe::COLUMNS);
    }

    #[test]
    fn modification_columns_map_one_field_each() {
        columns::assert_one_field_per_column(AnimModification::COLUMNS);
    }

    #[test]
    fn length_counts_every_replay() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(3, &[2, 12]), modification(1, &[0, 5])],
        };

        assert_eq!(animation.length(), 33);
    }

    #[test]
    fn an_endless_timeline_has_no_length() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(4, &[0, 30]), modification(-1, &[0, 8])],
        };

        assert_eq!(animation.length(), -1);
    }

    #[test]
    fn period_is_the_common_multiple_of_endless_spans() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 8]), modification(-1, &[0, 12])],
        };

        assert_eq!(animation.period(), Some(24));
    }

    #[test]
    fn declared_frames_ignores_the_realignment_interval() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 8]), modification(-1, &[0, 140])],
        };

        assert_eq!(animation.playback_frames(), 280);
        assert_eq!(animation.declared_frames(), 140);
    }

    #[test]
    fn declared_frames_counts_a_timeline_that_ends() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(1, &[0, 152])],
        };

        assert_eq!(animation.declared_frames(), 153);
    }

    #[test]
    fn period_rejects_a_timeline_that_stops() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 8]), modification(1, &[0, 12])],
        };

        assert_eq!(animation.period(), None);
    }
}
