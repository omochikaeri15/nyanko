use serde::Serialize;

use crate::common::tools::columns::{self, Column};
use crate::common::tools::file;

use super::RigError;

/// One model part in its rest pose, before any animation is applied.
///
/// Every transform is an integer in the model's own units; the divisors on
/// [`Model`] convert them to the ratios the engine applies.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ModelPart {
    /// The index of the part this one hangs off, which is minus one for the root.
    pub parent: i32,
    /// The owning entity's identifier, which is minus one on a part the engine never draws.
    pub id: i32,
    /// The index of the sprite region this part draws from the atlas.
    pub sprite: i32,
    /// The depth layer, where higher values draw in front.
    pub z: i32,
    /// The resting offset from the parent along the X axis.
    pub x: i32,
    /// The resting offset from the parent along the Y axis.
    pub y: i32,
    /// The rotation anchor within the sprite along the X axis.
    pub pivot_x: i32,
    /// The rotation anchor within the sprite along the Y axis.
    pub pivot_y: i32,
    /// The resting scale along the X axis, divided by the model's scale unit.
    pub scale_x: i32,
    /// The resting scale along the Y axis, divided by the model's scale unit.
    pub scale_y: i32,
    /// The resting rotation, divided by the model's angle unit to give turns.
    pub angle: i32,
    /// The resting opacity, divided by the model's opacity unit.
    pub opacity: i32,
    /// The blending mode index, where zero is ordinary alpha blending.
    pub glow: i32,
    /// The part's declared name.
    pub name: String,
}

impl ModelPart {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `.mamodel` part row from
    /// the parser's own table instead of restating it. [`ModelPart::name`] is
    /// the row's trailing text rather than a column.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        parent   :  0;
        id       :  1;
        sprite   :  2;
        z        :  3;
        x        :  4;
        y        :  5;
        pivot_x  :  6;
        pivot_y  :  7;
        scale_x  :  8;
        scale_y  :  9;
        angle    : 10;
        opacity  : 11;
        glow     : 12;
    };
}

/// One row of the trailing block that positions a model against the world.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Alignment {
    /// The first column, whose meaning the engine does not read.
    pub unknown_0: i32,
    /// The second column, whose meaning the engine does not read.
    pub unknown_1: i32,
    /// The horizontal offset subtracted from the root part's pivot.
    pub x: i32,
    /// The vertical offset subtracted from the root part's pivot.
    pub y: i32,
    /// The fifth column, whose meaning the engine does not read.
    pub unknown_4: i32,
    /// The sixth column, whose meaning the engine does not read.
    pub unknown_5: i32,
    /// The row's declared name.
    pub name: String,
}

impl Alignment {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `.mamodel` alignment row
    /// from the parser's own table instead of restating it. [`Alignment::name`]
    /// is the row's trailing text rather than a column.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        unknown_0 : 0;
        unknown_1 : 1;
        x         : 2;
        y         : 3;
        unknown_4 : 4;
        unknown_5 : 5;
    };
}

/// A unit's part hierarchy in its rest pose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    /// The declared format version.
    pub version: i32,
    /// The parts, in the order the file declares them, with the root first.
    pub parts: Vec<ModelPart>,
    /// The divisor turning a part's raw scale into a ratio.
    pub scale_unit: i32,
    /// The divisor turning a part's raw angle into turns.
    pub angle_unit: i32,
    /// The divisor turning a part's raw opacity into a ratio.
    pub opacity_unit: i32,
    /// The fourth unit column, present only in the second format revision.
    pub unknown_3: Option<i32>,
    /// The trailing alignment rows, of which the engine reads only the first.
    pub alignment: Vec<Alignment>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            version: 0,
            parts: Vec::new(),
            scale_unit: 1000,
            angle_unit: 3600,
            opacity_unit: 1000,
            unknown_3: None,
            alignment: Vec::new(),
        }
    }
}

impl Model {
    /// Parses a `.mamodel` byte stream into a structured `Model` hierarchy.
    ///
    /// The file opens with a tag line, a version, and a part count, after which
    /// each part occupies one row. A units row follows the parts, then a count
    /// and that many alignment rows.
    ///
    /// # Arguments
    /// * `bytes` - The raw byte data of the `.mamodel` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `Model` on success, or a `RigError` if
    /// the file was empty or declared no usable part count.
    pub fn parse(bytes: impl AsRef<[u8]>) -> Result<Self, RigError> {
        Self::parse_inner(bytes.as_ref())
    }

    fn parse_inner(bytes: &[u8]) -> Result<Self, RigError> {
        let content = file::scrub(bytes);
        let delimiter = file::detect_separator(&content);
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();

        if lines.is_empty() { return Err(RigError::EmptyFile); }

        let mut cursor = usize::from(lines[0].trim_start().starts_with('['));

        let version = lines.get(cursor).and_then(|line| line.trim().parse().ok()).unwrap_or(0);
        cursor += 1;

        let count = lines.get(cursor)
            .and_then(|line| line.trim().parse::<usize>().ok())
            .ok_or(RigError::NoPartHeader)?;
        cursor += 1;

        if count == 0 { return Err(RigError::NoPartHeader); }

        let declared = count.min(lines.len().saturating_sub(cursor));
        let mut parts = Vec::with_capacity(declared);

        for index in 0..declared {
            let row: Vec<&str> = lines[cursor + index].split(delimiter).collect();
            let mut part = ModelPart::default();

            let trailing = columns::apply(&row, ModelPart::COLUMNS, &mut part);
            part.name = row.get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            parts.push(part);
        }

        if parts.is_empty() { return Err(RigError::NoPartHeader); }
        cursor += declared;

        let mut model = Model { version, parts, ..Model::default() };

        if let Some(units) = lines.get(cursor) {
            let columns: Vec<&str> = units.split(delimiter).collect();
            let column = |at: usize| columns.get(at).and_then(|text| text.trim().parse::<i32>().ok());

            model.scale_unit = column(0).unwrap_or(model.scale_unit);
            model.angle_unit = column(1).unwrap_or(model.angle_unit);
            model.opacity_unit = column(2).unwrap_or(model.opacity_unit);
            model.unknown_3 = column(3);
            cursor += 1;
        }

        let alignment_count = lines.get(cursor)
            .and_then(|line| line.trim().parse::<usize>().ok())
            .unwrap_or(0);
        cursor += 1;

        let declared = alignment_count.min(lines.len().saturating_sub(cursor));
        model.alignment = Vec::with_capacity(declared);

        for index in 0..declared {
            let row: Vec<&str> = lines[cursor + index].split(delimiter).collect();
            let mut alignment = Alignment::default();

            let trailing = columns::apply(&row, Alignment::COLUMNS, &mut alignment);
            alignment.name = row.get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            model.alignment.push(alignment);
        }

        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_columns_map_one_field_each() {
        columns::assert_one_field_per_column(ModelPart::COLUMNS);
    }

    #[test]
    fn alignment_columns_map_one_field_each() {
        columns::assert_one_field_per_column(Alignment::COLUMNS);
    }
}
