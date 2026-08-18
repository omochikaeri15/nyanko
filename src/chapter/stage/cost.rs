//! Decoding of the packed entry cost a stage's metadata carries.
//!
//! A stage's cost column is not always the plain energy it appears to be. Two
//! separate schemes pack a currency and a quantity into the one value, and they
//! are selected by unrelated means: the Catamin scheme applies to every stage of
//! the Catamin chapter and nowhere else, while the item scheme is switched on
//! per map by a flag in that map's metadata header. They are never both in
//! effect, so each has its own entry point here and neither infers the other's
//! condition.

use serde::{Deserialize, Serialize};

const CURRENCY_SCALE: u32 = 1000;

/// The grade of Catamin a Catamin stage charges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CataminGrade {
    /// Catamin A, selected by a packed key of zero.
    A,
    /// Catamin B, selected by a packed key of one.
    B,
    /// Catamin C, selected by a packed key of two or above.
    C,
}

/// The Catamin a stage of the Catamin chapter charges to attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CataminCost {
    /// The grade of Catamin charged.
    pub grade: CataminGrade,
    /// The number of that grade charged.
    pub quantity: u32,
}

/// The item a stage charges to attempt in place of energy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemCost {
    /// The identifier of the item charged.
    pub item_id: u32,
    /// The number of that item charged.
    pub quantity: u32,
}

/// Decodes a stage's cost column as the Catamin it charges.
///
/// Applies only to stages of [`Category::CataminStages`], for which the engine
/// hardcodes the scheme rather than reading any flag. The engine performs no
/// validation here, so neither does this: a quantity of zero decodes as a
/// quantity of zero, and any key past two decodes as grade C.
///
/// # Arguments
/// * `cost` - The raw cost column of the stage's metadata row.
///
/// # Returns
/// A `CataminCost` holding the grade and quantity packed into the column.
///
/// [`Category::CataminStages`]: crate::chapter::Category::CataminStages
pub fn catamin_cost(cost: u32) -> CataminCost {
    let grade = match cost / CURRENCY_SCALE {
        0 => CataminGrade::A,
        1 => CataminGrade::B,
        _ => CataminGrade::C,
    };

    CataminCost { grade, quantity: cost % CURRENCY_SCALE }
}

/// Decodes a stage's cost column as the item it charges.
///
/// Applies only to maps whose metadata header declares [`CostType::Item`], which
/// any map may do regardless of the chapter it belongs to. A column that packs
/// no item, or none of one, is not an item cost at all and leaves the value to
/// be read as the plain energy it is.
///
/// # Arguments
/// * `cost` - The raw cost column of the stage's metadata row.
///
/// # Returns
/// An `Option` holding the item and quantity packed into the column, or `None`
/// when the column names no item or no quantity of one.
///
/// [`CostType::Item`]: super::CostType::Item
pub fn item_cost(cost: u32) -> Option<ItemCost> {
    let item_id = cost / CURRENCY_SCALE;
    let quantity = cost % CURRENCY_SCALE;

    if item_id == 0 || quantity == 0 {
        return None;
    }

    Some(ItemCost { item_id, quantity })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catamin_keys_select_the_grade() {
        assert_eq!(catamin_cost(30).grade, CataminGrade::A);
        assert_eq!(catamin_cost(1030).grade, CataminGrade::B);
        assert_eq!(catamin_cost(2030).grade, CataminGrade::C);
        assert_eq!(catamin_cost(9030).grade, CataminGrade::C);
    }

    #[test]
    fn catamin_quantity_is_never_guarded() {
        assert_eq!(catamin_cost(0), CataminCost { grade: CataminGrade::A, quantity: 0 });
        assert_eq!(catamin_cost(2000), CataminCost { grade: CataminGrade::C, quantity: 0 });
        assert_eq!(catamin_cost(1005).quantity, 5);
    }

    #[test]
    fn item_costs_unpack_the_identifier_and_quantity() {
        assert_eq!(item_cost(28003), Some(ItemCost { item_id: 28, quantity: 3 }));
    }

    #[test]
    fn a_column_naming_no_item_is_not_an_item_cost() {
        assert_eq!(item_cost(300), None);
        assert_eq!(item_cost(28000), None);
        assert_eq!(item_cost(0), None);
    }
}
