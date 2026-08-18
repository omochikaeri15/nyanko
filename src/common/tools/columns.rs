//! Table-driven declaration of a flat delimited row's column layout.
//!
//! A parser for a flat row declares its layout once as a slice of [`Column`],
//! built with the [`columns!`] macro, and reads a row by walking that table
//! rather than by a hand-written run of indexed lookups. The table is then the
//! single source of truth for the layout: it can be published so a consumer
//! reads the mapping instead of restating it, and it makes the mapping testable
//! rather than assumed.

use std::fmt;

use serde::Serialize;

/// The arithmetic a raw column value passes through on its way into a field.
///
/// Some engine columns are stored in units the rest of the crate does not use:
/// a few durations are recorded at half their frame count, and several
/// distances are quadrupled. The conversion belongs to the column rather than
/// to the field it lands in, since one layout may scale a column that another
/// layout stores raw.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum Scale {
    /// The column is stored exactly as it is read.
    #[default]
    Raw,
    /// The column is doubled, converting a half-frame duration into frames.
    Double,
    /// The column is divided by four, converting a quadrupled distance into engine units.
    Quarter,
}

impl Scale {
    /// Applies the conversion to one raw column value.
    ///
    /// # Arguments
    /// * `value` - The integer read from the column, or the column's default when it was absent or unparseable.
    ///
    /// # Returns
    /// An `i32` holding the converted value.
    pub const fn apply(self, value: i32) -> i32 {
        match self {
            Self::Raw => value,
            Self::Double => value.saturating_mul(2),
            Self::Quarter => value / 4,
        }
    }
}

/// The definition of one column of a raw row, and where it lands in `T`.
///
/// A layout publishes its full mapping as a slice of these, in the order the
/// parser applies them. The highest [`Column::index`] in a table is the last
/// column that layout understands; anything past it is trailing data the table
/// does not describe.
#[derive(Serialize)]
#[serde(bound = "")]
pub struct Column<T> {
    /// The name of the `T` field this column populates, matching its serialized key.
    pub field: &'static str,
    /// The zero-based position of this column within the raw row.
    pub index: usize,
    /// The conversion applied to the raw value before it is stored.
    pub scale: Scale,
    /// The value used when the column is absent from the row or does not parse as an integer.
    pub default: i32,
    #[serde(skip)]
    store: fn(&mut T, i32),
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Column<T> {}

impl<T> fmt::Debug for Column<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Column")
            .field("field", &self.field)
            .field("index", &self.index)
            .field("scale", &self.scale)
            .field("default", &self.default)
            .finish()
    }
}

impl<T> Column<T> {
    pub(crate) const fn new(
        field: &'static str,
        index: usize,
        scale: Scale,
        default: i32,
        store: fn(&mut T, i32),
    ) -> Self {
        Self { field, index, scale, default, store }
    }

    /// Reads this column out of a raw row and stores it into the target.
    ///
    /// A column the row does not reach, or whose text does not parse as an
    /// integer, contributes [`Column::default`] instead.
    ///
    /// # Arguments
    /// * `row` - The already-split columns of one raw row.
    /// * `target` - The structure the column's value is stored into.
    pub fn read(&self, row: &[&str], target: &mut T) {
        let raw = row
            .get(self.index)
            .and_then(|value| value.trim().parse::<i32>().ok())
            .unwrap_or(self.default);

        (self.store)(target, self.scale.apply(raw));
    }
}

/// Reads every column of a table out of a raw row.
///
/// # Arguments
/// * `row` - The already-split columns of one raw row.
/// * `table` - The layout to apply, in the order it is declared.
/// * `target` - The structure the columns are stored into.
///
/// # Returns
/// A `usize` holding the first index past the widest column the table
/// describes, which is where a caller retaining trailing columns begins.
pub fn apply<T>(row: &[&str], table: &[Column<T>], target: &mut T) -> usize {
    let mut widest = 0;

    for column in table {
        widest = widest.max(column.index);
        column.read(row, target);
    }

    widest + 1
}

/// Declares a column table as a slice of [`Column`].
///
/// Each entry is `field: index`, optionally followed by a [`Scale`] variant and
/// then the value to fall back on when the column is missing or unparseable. A
/// table whose columns nearly all share one fallback can state it once with a
/// leading `absent <literal>;`, which any entry may still override.
macro_rules! columns {
    (@scale) => { $crate::common::tools::columns::Scale::Raw };
    (@scale $scale:ident) => { $crate::common::tools::columns::Scale::$scale };
    (@default $absent:literal,) => { $absent };
    (@default $absent:literal, $default:literal) => { $default };
    (@table $absent:literal; $($field:ident : $index:literal $(, $scale:ident $(, $default:literal)?)?);* $(;)?) => {
        &[$($crate::common::tools::columns::Column::new(
            stringify!($field),
            $index,
            $crate::common::tools::columns::columns!(@scale $($scale)?),
            $crate::common::tools::columns::columns!(@default $absent, $($($default)?)?),
            |target, value| target.$field = value,
        )),*]
    };
    (absent $absent:literal; $($body:tt)*) => {
        $crate::common::tools::columns::columns!(@table $absent; $($body)*)
    };
    ($($body:tt)*) => {
        $crate::common::tools::columns::columns!(@table 0; $($body)*)
    };
}

pub(crate) use columns;
