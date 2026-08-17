use crate::common::tools::file;
use crate::graphics::tools::math;

use super::RigError;

/// A single control point on an animation curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Keyframe {
    /// The frame at which this control point takes effect.
    pub frame: i32,
    /// The raw value the curve holds at this control point, in the units of the modified property.
    pub value: i32,
    /// The interpolation strategy applied between this control point and the next.
    pub ease_mode: i32,
    /// The exponent applied by the easing strategies that accept one.
    pub ease_power: i32,
}

/// A single animated property of one model part over time.
///
/// Each modification drives exactly one property of one part, so a part that
/// moves, rotates, and fades simultaneously is described by three separate
/// modifications sharing a part identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimModification {
    /// The index of the model part this modification drives.
    pub part_id: usize,
    /// The identifier of the property being modified, such as position, scale, or opacity.
    pub modification_type: i32,
    /// The repetition behavior of the curve, where a value of one plays it exactly once.
    pub loop_count: i32,
    /// The control points defining the curve, in ascending frame order.
    pub keyframes: Vec<Keyframe>,
    /// The first frame of the curve's declared active range.
    pub min_frame: i32,
    /// The last frame of the curve's declared active range.
    pub max_frame: i32,
}

/// A complete animation timeline for a unit's rig.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Animation {
    /// The property curves that make up the animation.
    pub curves: Vec<AnimModification>,
    /// The highest frame referenced by any curve, giving the timeline's nominal length.
    pub max_frame: i32,
}

impl Animation {
    /// Parses a `.maanim` byte stream into a structured animation timeline.
    ///
    /// The file declares a run of property curves, each introduced by a header
    /// row and followed by its own keyframe count and keyframe rows. Curves that
    /// resolve to no keyframes are discarded, and the timeline's nominal length
    /// is taken from the highest keyframe encountered.
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
        let delimiter = file::detect_separator(&content);
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
        if lines.is_empty() { return Err(RigError::EmptyFile); }

        fn parse_num<T: std::str::FromStr + Default>(input_string: &str) -> T {
            input_string.trim().parse().unwrap_or_default()
        }

        let mut curves = Vec::new();
        let mut line_idx = 0;

        if line_idx < lines.len() && lines[line_idx].trim().starts_with('[') { line_idx += 1; }
        if line_idx < lines.len() { line_idx += 1; }
        if line_idx < lines.len() { line_idx += 1; }

        while line_idx < lines.len() {
            let current_line = lines[line_idx];
            let parts: Vec<&str> = current_line.split(delimiter).collect();
            line_idx += 1;

            if parts.len() < 5 { continue; }

            let part_id: usize = parse_num(parts[0]);
            let mod_type: i32 = parse_num(parts[1]);
            let loop_behavior: i32 = parse_num(parts[2]);
            let min_frame: i32 = parse_num(parts[3]);
            let max_frame: i32 = parse_num(parts[4]);

            if line_idx >= lines.len() { break; }
            let count_line = lines[line_idx];
            line_idx += 1;

            let count_str = count_line.split(delimiter).next().unwrap_or_default();
            let keyframe_count: usize = parse_num(count_str);

            let mut keyframes = Vec::new();

            for _ in 0..keyframe_count {
                if line_idx >= lines.len() { break; }
                let keyframe_line = lines[line_idx];
                line_idx += 1;
                let keyframe_parts: Vec<&str> = keyframe_line.split(delimiter).collect();

                if keyframe_parts.len() >= 2 {
                    let frame: i32 = parse_num(keyframe_parts[0]);
                    let value: i32 = parse_num(keyframe_parts[1]);
                    let ease_mode = keyframe_parts.get(2).map_or(0, |s| parse_num(s));
                    let ease_power = keyframe_parts.get(3).map_or(0, |s| parse_num(s));

                    keyframes.push(Keyframe { frame, value, ease_mode, ease_power });
                }
            }

            if !keyframes.is_empty() {
                curves.push(AnimModification {
                    part_id, modification_type: mod_type, loop_count: loop_behavior,
                    keyframes, min_frame, max_frame,
                });
            }
        }

        let mut max_len = 0;
        for curve in &curves {
            if let Some(last_keyframe) = curve.keyframes.last()
                && last_keyframe.frame > max_len { max_len = last_keyframe.frame; }
        }

        Ok(Self { curves, max_frame: max_len })
    }

    /// Calculates the frame count after which every looping curve realigns.
    ///
    /// The result is the least common multiple of the individual curve
    /// durations, which is the shortest interval over which the whole timeline
    /// repeats. An animation containing any curve that plays exactly once has no
    /// such interval, and neither does one whose combined period is
    /// implausibly long or falls short of the timeline's own declared length.
    ///
    /// # Returns
    /// An `Option` containing the combined loop length in frames, or `None` if
    /// the animation does not loop coherently.
    pub fn calculate_true_loop(&self) -> Option<i32> {
        let mut overall_lcm: i64 = 1;
        let mut found_looping_part = false;

        for curve in &self.curves {
            if curve.loop_count == 1 { return None; }

            let first_keyframe = match curve.keyframes.first() {
                Some(k) => k,
                None => continue,
            };

            let last_keyframe = match curve.keyframes.last() {
                Some(k) => k,
                None => continue,
            };

            let duration = last_keyframe.frame - first_keyframe.frame;
            if duration <= 0 { continue; }

            overall_lcm = math::lcm(overall_lcm as i32, duration);
            if overall_lcm > 999_999 { return None; }

            found_looping_part = true;
        }

        if !found_looping_part {
            return Some(self.max_frame);
        }

        if (overall_lcm as i32) < self.max_frame {
            return None;
        }

        Some(overall_lcm as i32)
    }

    /// Measures an animation's played length without building the full timeline.
    ///
    /// This walks the curve headers and keyframe bounds directly, accounting for
    /// each curve's repetition count, and is substantially cheaper than
    /// [`Animation::parse`] followed by inspection.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the unit's `.maanim` file.
    ///
    /// # Returns
    /// An `Option` containing the animation's length in frames, or `None` if the
    /// file contained no readable lines.
    pub fn scan_duration(bytes: impl AsRef<[u8]>) -> Option<i32> {
        Self::scan_duration_inner(bytes.as_ref())
    }

    fn scan_duration_inner(bytes: &[u8]) -> Option<i32> {
        let content = file::scrub(bytes);
        let delimiter = file::detect_separator(&content);

        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
        if lines.is_empty() { return None; }

        let mut max_frame_count = 0;
        let mut line_idx = 0;

        if line_idx < lines.len() && lines[line_idx].trim().starts_with('[') { line_idx += 1; }
        if line_idx < lines.len() { line_idx += 1; }
        if line_idx < lines.len() { line_idx += 1; }

        while line_idx < lines.len() {
            let parts: Vec<&str> = lines[line_idx].split(delimiter).collect();
            line_idx += 1;

            if parts.len() < 5 { continue; }

            let loop_count: i32 = parts[2].trim().parse().unwrap_or(1);
            let repeats = std::cmp::max(loop_count, 1);

            if line_idx >= lines.len() { break; }

            let count_line = lines[line_idx];
            let keyframe_count: usize = count_line.split(delimiter)
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or_default();
            line_idx += 1;

            if keyframe_count > 0 {
                let Some(first_frame_line) = lines.get(line_idx) else { break; };
                let first_frame: i32 = first_frame_line.split(delimiter)
                    .next()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or_default();

                let last_idx = line_idx + keyframe_count - 1;
                let last_frame: i32 = lines.get(last_idx)
                    .and_then(|line| line.split(delimiter).next())
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or_default();

                let duration = last_frame - first_frame;
                max_frame_count = std::cmp::max((duration * repeats) + first_frame, max_frame_count);

                line_idx += keyframe_count;
            }
        }

        Some(max_frame_count)
    }
}