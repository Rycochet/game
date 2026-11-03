//! The size grid: which lengths of blade and handle make which cell, and what each cell is called.

use super::Build;

/// How much of a weapon's length takes one more cell.
pub(super) const CELL_LENGTH: u32 = 200;
/// A handle's section, a little over 3 cm across.
pub(super) const HANDLE_AREA_MM2: u64 = 700;
/// The share out of 100 of weapons asked for by a creature smaller than ordinary that
/// are scaled even though an ordinary one fits, and for one larger.
pub(super) const SCALED_PERCENT_SMALL: u64 = 10;
pub(super) const SCALED_PERCENT_LARGE: u64 = 40;

/// How long it is from end to end, as far as carrying it goes: a head at the side adds
/// nothing to that.
pub(super) fn length_of(build: Build, blade: u16, handle: u16) -> u16 {
    match build {
        Build::Along => blade + handle,
        Build::Side => handle,
    }
}

/// From here on a handle is a shaft with a socketed head rather than a grip with a tang.
pub(super) const SHAFT_FROM: u16 = 400;

/// A range of lengths: from the first, up to but not including the last.
pub(super) type Span = (u16, u16);

/// One place in a size grid that may be made, and what it is usually called. Anything
/// not listed is never made. With no generic name, only named weapons fit it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Cell {
    pub(super) build: Build,
    pub(super) blade: Span,
    pub(super) handle: Span,
    pub(super) generic: Option<&'static str>,
}

pub(super) const fn along(blade: Span, handle: Span, generic: Option<&'static str>) -> Cell {
    Cell {
        build: Build::Along,
        blade,
        handle,
        generic,
    }
}

pub(super) const fn side(blade: Span, handle: Span, generic: Option<&'static str>) -> Cell {
    Cell {
        build: Build::Side,
        blade,
        handle,
        generic,
    }
}

pub(super) const SHORT: Span = (50, 350);
pub(super) const MEDIUM: Span = (350, 600);
pub(super) const LONG: Span = (600, 1000);
pub(super) const GREAT: Span = (1000, 1500);

pub(super) const GRIP: Span = (80, 200);
pub(super) const LONG_GRIP: Span = (200, 400);
pub(super) const HAFT: Span = (400, 1000);
pub(super) const STAFF: Span = (1000, 1700);
pub(super) const POLE: Span = (1700, 2800);
pub(super) const GREAT_POLE: Span = (2800, 7000);

pub(super) const SMALL_EDGE: Span = (50, 150);
pub(super) const BROAD_EDGE: Span = (150, 350);
pub(super) const LONG_EDGE: Span = (350, 700);

pub(super) const AXE_HAFT: Span = (250, 600);
pub(super) const AXE_LONG_HAFT: Span = (600, 1200);
pub(super) const AXE_STAFF: Span = (1200, 1800);
pub(super) const AXE_POLE: Span = (1800, 2800);

/// The size grids. Between them, these say what is never made: a short blade on a long
/// grip, say, or a great blade on a pole.
pub(super) const CELLS: [Cell; 20] = [
    along(SHORT, GRIP, Some("Knife")),
    along(SHORT, HAFT, Some("Stabbing Spear")),
    along(SHORT, STAFF, Some("Short Spear")),
    along(SHORT, POLE, Some("Spear")),
    along(SHORT, GREAT_POLE, Some("Pike")),
    along(MEDIUM, GRIP, Some("Short Sword")),
    along(MEDIUM, STAFF, Some("Polearm")),
    along(MEDIUM, POLE, Some("Polearm")),
    along(LONG, GRIP, Some("Sword")),
    along(LONG, LONG_GRIP, Some("Longsword")),
    along(LONG, HAFT, None),
    along(GREAT, GRIP, None),
    along(GREAT, LONG_GRIP, Some("Greatsword")),
    side(SMALL_EDGE, AXE_HAFT, Some("Hatchet")),
    side(SMALL_EDGE, AXE_LONG_HAFT, Some("Axe")),
    side(BROAD_EDGE, AXE_LONG_HAFT, Some("Battle Axe")),
    side(BROAD_EDGE, AXE_STAFF, Some("Great Axe")),
    side(BROAD_EDGE, AXE_POLE, Some("Polearm")),
    side(LONG_EDGE, AXE_STAFF, Some("Polearm")),
    side(LONG_EDGE, AXE_POLE, Some("Polearm")),
];

pub(super) fn within(span: Span, length: u16) -> bool {
    span.0 <= length && length < span.1
}

pub(super) fn cell_of(build: Build, blade: u16, handle: u16) -> Option<&'static Cell> {
    CELLS.iter().find(|cell| {
        cell.build == build && within(cell.blade, blade) && within(cell.handle, handle)
    })
}

/// The lengths two spans share, if any.
pub(super) fn overlap(one: Span, other: Span) -> Option<Span> {
    let span = (one.0.max(other.0), one.1.min(other.1));
    (span.0 < span.1).then_some(span)
}
