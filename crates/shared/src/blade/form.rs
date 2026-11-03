//! `BladeForm`: one blade as it was made, its name, cells and weight.

use serde::{Deserialize, Serialize};

use super::table::rows;
use super::{
    Attachment, BROAD_EDGE, Back, BladeShape, BladeWidth, Build, CELL_LENGTH, Curve, Decoration,
    Edges, Family, Guard, HANDLE_AREA_MM2, HandleShape, Hands, Point, Pommel, Ricasso, SHAFT_FROM,
    Section, SidePart, Tips, Top, cell_of, length_of,
};
use crate::creature::Size;
use crate::item::{MAX_SHAPE_SIDE, Shape};
use crate::material::Material;

/// What one weapon is like. Lengths are whole millimetres, at the size of an ordinary
/// person's weapon: an item's own size goes on top of this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BladeForm {
    pub family: Family,
    pub build: Build,
    pub blade: u16,
    pub handle: u16,
    pub width: BladeWidth,
    pub edges: Edges,
    pub curve: Curve,
    pub point: Point,
    pub shape: BladeShape,
    pub section: Section,
    pub fuller: bool,
    pub hands: Hands,
    pub guard: Guard,
    pub pommel: Pommel,
    pub ricasso: Ricasso,
    pub langets: bool,
    pub handle_shape: HandleShape,
    /// One entry for each side of the handle.
    pub sides: [SidePart; 2],
    pub top: Top,
    pub back: Back,
    pub tips: Tips,
    pub blade_metal: Material,
    pub handle_material: Material,
    pub fittings: Material,
    pub decoration: Decoration,
    /// Scaled by a roll, so too unusual to count as any specific weapon: only its
    /// generic name. A weapon made on request at another size keeps its name.
    pub generic_only: bool,
}

impl BladeForm {
    pub fn attachment(&self) -> Attachment {
        match self.build {
            Build::Side => Attachment::SideMounted,
            Build::Along if self.handle < SHAFT_FROM => Attachment::Tang,
            Build::Along => Attachment::Socket,
        }
    }

    /// The most specific name that fits it, or else the generic name for its size.
    pub fn name(&self) -> &'static str {
        if self.generic_only {
            return self.generic_name();
        }
        // Equally specific names go to the one earlier in the table, the better known.
        let fitting = rows()
            .enumerate()
            .filter(|(_, row)| !row.generic && row.fits(self))
            .max_by_key(|(place, row)| (row.specifics(), std::cmp::Reverse(*place)));
        if let Some((_, row)) = fitting {
            return row.name;
        }
        self.generic_name()
    }

    fn generic_name(&self) -> &'static str {
        match cell_of(self.build, self.blade, self.handle).and_then(|cell| cell.generic) {
            Some("Knife") if self.edges != Edges::Single => "Dagger",
            Some(generic) => generic,
            None => "Blade",
        }
    }

    /// The cells it takes at its ordinary size, an item's size going on top.
    pub fn shape(&self) -> Shape {
        let length = length_of(self.build, self.blade, self.handle) as u32;
        let long = (length + CELL_LENGTH / 2) / CELL_LENGTH;
        let long = long.clamp(1, MAX_SHAPE_SIDE as u32) as u8;
        let wide = self.build == Build::Side && self.blade >= BROAD_EDGE.0;
        Shape::rectangle(if wide { 2 } else { 1 }, long)
    }

    /// What it weighs at its ordinary size, in grams, from how much of what it is made.
    pub fn grams(&self) -> u32 {
        let head_mm3 = match self.build {
            Build::Along => {
                let width = match self.width {
                    BladeWidth::Narrow => 25,
                    BladeWidth::Normal => 45,
                    BladeWidth::Broad => 65,
                };
                self.blade as u64 * width * 3
            }
            Build::Side => {
                let blade = self.blade as u64;
                let heads = 1 + (self.sides[1] == SidePart::AxeBlade) as u64;
                heads * blade * (blade / 2).min(150) * 4
            }
        };
        let handle_mm3 = self.handle as u64 * HANDLE_AREA_MM2;
        let head = head_mm3 * self.blade_metal.density() as u64 / 10_000;
        let handle = handle_mm3 * self.handle_material.density() as u64 / 10_000;
        let guard = match self.guard {
            Guard::None => 0,
            Guard::Disc => 80,
            Guard::Cross => 120,
            Guard::KnuckleBow => 150,
            Guard::Cup => 250,
            Guard::Basket => 350,
        };
        let pommel = if self.pommel == Pommel::None { 0 } else { 100 };
        let part = |there: bool| if there { 150 } else { 0 };
        let extras = part(!matches!(
            self.sides[0],
            SidePart::None | SidePart::AxeBlade
        )) + part(!matches!(
            self.sides[1],
            SidePart::None | SidePart::AxeBlade
        )) + part(self.top == Top::Spike)
            + part(self.back != Back::None);
        (head + handle) as u32 + guard + pommel + extras
    }

    /// What it weighs at this size: size in each direction, so twice as big is eight
    /// times as heavy.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters * quarters / 64) as u32
    }
}
