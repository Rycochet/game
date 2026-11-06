//! Weapons made of a handle with a blade or head on it: knives through to polearms.
//! A form is rolled once, within a row of the table, in a cell the size grid allows,
//! and named from what it turned out like. See `docs/context/shared/blade.md`.

mod cells;
mod family;
mod form;
mod names;
mod parts;
mod properties;
mod roll;
mod table;
mod within;

pub use family::Family;
pub use form::BladeForm;
pub use names::weapon_names;
pub use parts::{
    Attachment, Back, BladeShape, BladeWidth, Build, Curve, Decoration, Edges, Guard, HandleShape,
    Hands, Point, Pommel, Ricasso, Section, SidePart, Tips, Top,
};
pub use roll::{
    Rolled, commission_blade, limits_for_wielder, roll_blade, roll_blade_at, roll_blade_at_within,
    roll_blade_within,
};

// What the files here share, as `super::...`.
use cells::{
    AXE_LONG_HAFT, BROAD_EDGE, CELL_LENGTH, CELLS, GREAT, GRIP, HAFT, HANDLE_AREA_MM2,
    SCALED_PERCENT_LARGE, SCALED_PERCENT_SMALL, SHAFT_FROM, Span, cell_of, length_of, overlap,
    within,
};
pub(crate) use properties::blade_metals;
use properties::roll_properties;
use within::{pick, roll_row_within};

#[cfg(test)]
mod tests;
