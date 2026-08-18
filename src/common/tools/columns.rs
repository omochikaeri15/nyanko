//! Table-driven declaration of a flat row's column layout.
//!
//! A parser declares its layout once as a slice of [`Column`] and reads a row by
//! walking that table. The table can be published, so a consumer reads the
//! mapping from the parser itself rather than restating it.

use std::fmt;

use serde::Serialize;

/// The arithmetic a raw column value passes through on its way into a field.
///
/// The engine stores some values in units the rest of the crate does not use:
/// a few durations are recorded at half their frame count, and several
/// distances are quadrupled. The conversion is part of a column's definition
/// rather than of the field it lands in, since the same field may be scaled in
/// one layout and raw in another.
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

/// The definition of one column of a raw row, and the field it lands in.
///
/// A layout's full column mapping is published as a slice of these, so a
/// consumer needing to know which index feeds which field, how it is scaled, or
/// what it falls back to can read that from the same table the parser itself
/// runs on, rather than mirroring the parser by hand.
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
/// describes, where a caller retaining trailing columns begins.
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
/// then the value the column falls back to when it is missing or unparseable. A
/// table whose columns share one fallback states it once as a leading
/// `absent <literal>;`, which any entry may still override.
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
