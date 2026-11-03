//! What a farm poi asks for, and how a field looks.

use serde::{Deserialize, Serialize};

use super::super::building::BuildingOptions;
use super::super::town::Rect;

/// Options for the farm generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FarmOptions {
    pub house: BuildingOptions,
    /// How many of the places round the house are used, an inclusive range.
    pub fields: (i64, i64),
}

/// What is needed to choose what grows in a field. Which stage it is at is
/// the client's choice: it is only looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldLook {
    /// The cells sown, inside the fence.
    pub rect: Rect,
    /// Grazed rather than cropped.
    pub livestock: bool,
    /// A number for looks, so fields differ.
    pub variety: u32,
}

impl FieldLook {
    /// Whether a cell is in the field.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        let Rect {
            x: left,
            y: top,
            width,
            height,
        } = self.rect;
        (left..left + width).contains(&x) && (top..top + height).contains(&y)
    }
}
