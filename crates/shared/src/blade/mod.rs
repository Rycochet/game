//! Weapons made of a handle with a blade or head on it: knives through to polearms.
//! A form is rolled once, within a row of the table, in a cell the size grid allows,
//! and named from what it turned out like. See `docs/context/shared/blade.md`.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

mod table;

use table::{ROWS, Row};

/// The groups of weapon, each with its generic names and specific ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Family {
    Knife,
    Dagger,
    ShortSword,
    Sword,
    LongSword,
    Axe,
    Spear,
    Polearm,
}

impl Family {
    pub const ALL: [Self; 8] = [
        Self::Knife,
        Self::Dagger,
        Self::ShortSword,
        Self::Sword,
        Self::LongSword,
        Self::Axe,
        Self::Spear,
        Self::Polearm,
    ];
}

/// How the blade meets the handle, which decides how its length is measured and which
/// size grid it uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Build {
    /// A blade carrying on along the handle's line, measured from where they meet.
    Along,
    /// A head on the side of the handle, measured along its cutting edge.
    Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BladeWidth {
    Narrow,
    Normal,
    Broad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edges {
    PointOnly,
    Single,
    Double,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Curve {
    Straight,
    Slight,
    Strong,
    /// Bent forward, towards the edge, as on a kukri.
    Inward,
    Wavy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Point {
    Sharp,
    Rounded,
    None,
    Hooked,
    Clip,
    Notched,
    Flared,
}

/// A blade's outline, seen from the side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BladeShape {
    Parallel,
    Tapered,
    Leaf,
    Widening,
}

/// A blade's shape, cut across.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Section {
    Flat,
    Lenticular,
    Diamond,
    Midrib,
    Triangular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hands {
    One,
    OneOrTwo,
    Two,
}

/// Follows from the build and handle, so it is worked out rather than rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Attachment {
    Tang,
    Socket,
    SideMounted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Guard {
    None,
    Cross,
    Disc,
    Cup,
    KnuckleBow,
    Basket,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pommel {
    None,
    Plain,
    Ring,
    Wheel,
    Disc,
    Hooked,
}

/// The unsharpened part of a blade above the guard.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ricasso {
    None,
    Plain,
    WithHooks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleShape {
    Straight,
    Curved,
    SCurved,
    Angled,
}

/// What sticks out on one side of the handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SidePart {
    None,
    AxeBlade,
    Hook,
    Spike,
    Beak,
    Lug,
}

/// What is on the very end of the handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Top {
    None,
    Blade,
    Spike,
}

/// What is behind an axe's head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Back {
    None,
    Spike,
    Hammer,
    Pipe,
}

/// How a spear's point divides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tips {
    Single,
    Barbed,
    TwoTines,
    ThreeTines,
}

/// What one weapon is like. Lengths are whole millimetres, at the size of an ordinary
/// person's weapon: an item's own size goes on top of this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BladeForm {
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
        // Equally specific names go to the one earlier in the table, the better known.
        let fitting = ROWS
            .iter()
            .enumerate()
            .filter(|(_, row)| !row.generic && row.fits(self))
            .max_by_key(|(place, row)| (row.specifics(), std::cmp::Reverse(*place)));
        if let Some((_, row)) = fitting {
            return row.name;
        }
        match cell_of(self.build, self.blade, self.handle).and_then(|cell| cell.generic) {
            Some("Knife") if self.edges != Edges::Single => "Dagger",
            Some(generic) => generic,
            None => "Blade",
        }
    }
}

/// From here on a handle is a shaft with a socketed head rather than a grip with a tang.
const SHAFT_FROM: u16 = 400;

/// A range of lengths: from the first, up to but not including the last.
type Span = (u16, u16);

/// One place in a size grid that may be made, and what it is usually called. Anything
/// not listed is never made. With no generic name, only named weapons fit it.
#[derive(Clone, Copy, Debug)]
struct Cell {
    build: Build,
    blade: Span,
    handle: Span,
    generic: Option<&'static str>,
}

const fn along(blade: Span, handle: Span, generic: Option<&'static str>) -> Cell {
    Cell {
        build: Build::Along,
        blade,
        handle,
        generic,
    }
}

const fn side(blade: Span, handle: Span, generic: Option<&'static str>) -> Cell {
    Cell {
        build: Build::Side,
        blade,
        handle,
        generic,
    }
}

const SHORT: Span = (50, 350);
const MEDIUM: Span = (350, 600);
const LONG: Span = (600, 1000);
const GREAT: Span = (1000, 1500);

const GRIP: Span = (80, 200);
const LONG_GRIP: Span = (200, 400);
const HAFT: Span = (400, 1000);
const STAFF: Span = (1000, 1700);
const POLE: Span = (1700, 2800);
const GREAT_POLE: Span = (2800, 7000);

const SMALL_EDGE: Span = (50, 150);
const BROAD_EDGE: Span = (150, 350);
const LONG_EDGE: Span = (350, 700);

const AXE_HAFT: Span = (250, 600);
const AXE_LONG_HAFT: Span = (600, 1200);
const AXE_STAFF: Span = (1200, 1800);
const AXE_POLE: Span = (1800, 2800);

/// The size grids. Between them, these say what is never made: a short blade on a long
/// grip, say, or a great blade on a pole.
const CELLS: [Cell; 20] = [
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

fn within(span: Span, length: u16) -> bool {
    span.0 <= length && length < span.1
}

fn cell_of(build: Build, blade: u16, handle: u16) -> Option<&'static Cell> {
    CELLS.iter().find(|cell| {
        cell.build == build && within(cell.blade, blade) && within(cell.handle, handle)
    })
}

/// The lengths two spans share, if any.
fn overlap(one: Span, other: Span) -> Option<Span> {
    let span = (one.0.max(other.0), one.1.min(other.1));
    (span.0 < span.1).then_some(span)
}

/// A new weapon of a family. Picks a row of the table, by how common it is, then a cell
/// it may be in, then each property in turn, within what the row allows.
pub fn roll_blade(rng: &mut Rng, family: Family) -> BladeForm {
    let rows: Vec<(&Row, u64)> = ROWS
        .iter()
        .filter(|row| row.family == family)
        .map(|row| (row, row.weight))
        .collect();
    let row = rng.pick_weighted(&rows);
    let places: Vec<((Span, Span), u64)> = row.places().map(|place| (place, 1)).collect();
    let (blade, handle) = rng.pick_weighted(&places);
    let blade = rng.range_u64(blade.0 as u64, blade.1 as u64) as u16;
    let handle = rng.range_u64(handle.0 as u64, handle.1 as u64) as u16;
    roll_properties(rng, row, blade, handle)
}

/// Picks by the usual weights among what the row allows. A row may allow only things
/// that are usually never seen on such a weapon, so those then count equally.
fn pick<T: Copy + PartialEq>(rng: &mut Rng, allowed: &[T], usual: &[(T, u64)]) -> T {
    let open = |option: &T| allowed.is_empty() || allowed.contains(option);
    let options: Vec<(T, u64)> = usual
        .iter()
        .copied()
        .filter(|(option, weight)| *weight > 0 && open(option))
        .collect();
    if options.is_empty() {
        let only: Vec<(T, u64)> = allowed.iter().map(|option| (*option, 1)).collect();
        rng.pick_weighted(&only)
    } else {
        rng.pick_weighted(&options)
    }
}

/// First guesses at how often each is seen. Properties are rolled in this order.
fn roll_properties(rng: &mut Rng, row: &Row, blade: u16, handle: u16) -> BladeForm {
    let build = row.build;
    let is_side = build == Build::Side;
    let held = !is_side && handle < SHAFT_FROM;
    let on_a_grip = !is_side && within(GRIP, handle);

    // A blade this long needs to be slim to be held in one hand.
    let must_be_narrow = on_a_grip && blade >= GREAT.0;
    let width = if must_be_narrow {
        BladeWidth::Narrow
    } else {
        pick(
            rng,
            row.width,
            &[
                (BladeWidth::Narrow, 25),
                (BladeWidth::Normal, 60),
                (BladeWidth::Broad, 15),
            ],
        )
    };
    let edges = pick(
        rng,
        row.edges,
        &match (is_side, held) {
            (true, _) => [
                (Edges::Single, 100),
                (Edges::Double, 0),
                (Edges::PointOnly, 0),
            ],
            (false, true) => [
                (Edges::Single, 45),
                (Edges::Double, 45),
                (Edges::PointOnly, 10),
            ],
            (false, false) => [
                (Edges::Single, 30),
                (Edges::Double, 60),
                (Edges::PointOnly, 10),
            ],
        },
    );
    let curve = pick(
        rng,
        row.curve,
        &if held {
            [
                (Curve::Straight, 55),
                (Curve::Slight, 25),
                (Curve::Strong, 10),
                (Curve::Inward, 7),
                (Curve::Wavy, 3),
            ]
        } else {
            [
                (Curve::Straight, 80),
                (Curve::Slight, 20),
                (Curve::Strong, 0),
                (Curve::Inward, 0),
                (Curve::Wavy, 0),
            ]
        },
    );
    let point = pick(
        rng,
        row.point,
        &if is_side {
            [
                (Point::None, 100),
                (Point::Sharp, 0),
                (Point::Rounded, 0),
                (Point::Hooked, 0),
                (Point::Clip, 0),
                (Point::Notched, 0),
                (Point::Flared, 0),
            ]
        } else {
            [
                (Point::None, 5),
                (Point::Sharp, 75),
                (Point::Rounded, 5),
                (Point::Hooked, 5),
                (Point::Clip, 4),
                (Point::Notched, 3),
                (Point::Flared, 3),
            ]
        },
    );
    let shape = pick(
        rng,
        row.shape,
        &if is_side {
            [
                (BladeShape::Widening, 100),
                (BladeShape::Parallel, 0),
                (BladeShape::Tapered, 0),
                (BladeShape::Leaf, 0),
            ]
        } else {
            [
                (BladeShape::Widening, 5),
                (BladeShape::Parallel, 40),
                (BladeShape::Tapered, 40),
                (BladeShape::Leaf, 15),
            ]
        },
    );
    let section = pick(
        rng,
        row.section,
        &if is_side {
            [
                (Section::Flat, 100),
                (Section::Lenticular, 0),
                (Section::Diamond, 0),
                (Section::Midrib, 0),
                (Section::Triangular, 0),
            ]
        } else {
            [
                (Section::Flat, 20),
                (Section::Lenticular, 40),
                (Section::Diamond, 25),
                (Section::Midrib, 10),
                (Section::Triangular, 5),
            ]
        },
    );
    let fuller = pick(
        rng,
        row.fuller,
        &[(true, u64::from(held) * 40), (false, 60)],
    );
    let hands = pick(
        rng,
        row.hands,
        &if on_a_grip || (is_side && handle < AXE_LONG_HAFT.0) {
            [(Hands::One, 100), (Hands::OneOrTwo, 0), (Hands::Two, 0)]
        } else if handle < HAFT.1 {
            [(Hands::One, 0), (Hands::OneOrTwo, 50), (Hands::Two, 50)]
        } else {
            [(Hands::One, 0), (Hands::OneOrTwo, 0), (Hands::Two, 100)]
        },
    );
    let guard = pick(
        rng,
        row.guard,
        &if on_a_grip {
            [
                (Guard::None, 15),
                (Guard::Cross, 50),
                (Guard::Disc, 15),
                (Guard::Cup, 8),
                (Guard::KnuckleBow, 8),
                (Guard::Basket, 4),
            ]
        } else if held {
            [
                (Guard::None, 0),
                (Guard::Cross, 80),
                (Guard::Disc, 20),
                (Guard::Cup, 0),
                (Guard::KnuckleBow, 0),
                (Guard::Basket, 0),
            ]
        } else {
            [
                (Guard::None, 100),
                (Guard::Cross, 0),
                (Guard::Disc, 0),
                (Guard::Cup, 0),
                (Guard::KnuckleBow, 0),
                (Guard::Basket, 0),
            ]
        },
    );
    let pommel = pick(
        rng,
        row.pommel,
        &if held {
            [
                (Pommel::None, 5),
                (Pommel::Plain, 55),
                (Pommel::Disc, 15),
                (Pommel::Wheel, 15),
                (Pommel::Ring, 5),
                (Pommel::Hooked, 5),
            ]
        } else {
            [
                (Pommel::None, 100),
                (Pommel::Plain, 0),
                (Pommel::Disc, 0),
                (Pommel::Wheel, 0),
                (Pommel::Ring, 0),
                (Pommel::Hooked, 0),
            ]
        },
    );
    let ricasso = pick(
        rng,
        row.ricasso,
        &[
            (Ricasso::None, 90),
            (Ricasso::Plain, u64::from(held) * 10),
            (Ricasso::WithHooks, 0),
        ],
    );
    let langets = pick(
        rng,
        row.langets,
        &[(true, u64::from(!held) * 25), (false, 75)],
    );
    let handle_shape = pick(
        rng,
        row.handle_shape,
        &if held {
            [
                (HandleShape::Straight, 80),
                (HandleShape::Curved, 15),
                (HandleShape::SCurved, 3),
                (HandleShape::Angled, 2),
            ]
        } else {
            [
                (HandleShape::Straight, 85),
                (HandleShape::Curved, 5),
                (HandleShape::SCurved, 10),
                (HandleShape::Angled, 0),
            ]
        },
    );
    let side_parts = [
        pick(
            rng,
            row.side_one,
            &[
                (SidePart::AxeBlade, u64::from(is_side)),
                (SidePart::None, u64::from(!is_side)),
            ],
        ),
        pick(rng, row.side_two, &[(SidePart::None, 1)]),
    ];
    let top = pick(
        rng,
        row.top,
        &[
            (Top::None, u64::from(is_side)),
            (Top::Blade, u64::from(!is_side)),
        ],
    );
    let back = pick(
        rng,
        row.back,
        &[
            (Back::None, 85),
            (Back::Spike, u64::from(is_side) * 8),
            (Back::Hammer, u64::from(is_side) * 7),
        ],
    );
    let tips = pick(rng, row.tips, &[(Tips::Single, 1)]);
    BladeForm {
        build,
        blade,
        handle,
        width,
        edges,
        curve,
        point,
        shape,
        section,
        fuller,
        hands,
        guard,
        pommel,
        ricasso,
        langets,
        handle_shape,
        sides: side_parts,
        top,
        back,
        tips,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Seed;
    use std::collections::BTreeSet;

    fn rolled(family: Family) -> impl Iterator<Item = BladeForm> {
        (0..6000).map(move |seed| roll_blade(&mut Seed::new(seed).rng(), family))
    }

    fn all_names() -> BTreeSet<&'static str> {
        Family::ALL
            .into_iter()
            .flat_map(rolled)
            .map(|form| form.name())
            .collect()
    }

    fn sword(blade: u16, handle: u16) -> BladeForm {
        roll_properties(
            &mut Seed::new(1).rng(),
            ROWS.iter().find(|row| row.name == "Sword").unwrap(),
            blade,
            handle,
        )
    }

    #[test]
    fn the_same_seed_rolls_the_same_weapon() {
        // Golden: changing the rolls is fine, but should be noticed.
        let form = roll_blade(&mut Seed::new(1).rng(), Family::Sword);
        assert_eq!(form, roll_blade(&mut Seed::new(1).rng(), Family::Sword));
        assert_eq!((form.blade, form.handle, form.name()), (794, 272, "Katana"));
    }

    #[test]
    fn every_row_can_be_made_somewhere() {
        for row in &ROWS {
            assert!(row.places().next().is_some(), "{} has no cell", row.name);
        }
    }

    #[test]
    fn every_specific_name_turns_up() {
        let names = all_names();
        for row in ROWS.iter().filter(|row| !row.generic) {
            assert!(names.contains(row.name), "{} never turns up", row.name);
        }
    }

    #[test]
    fn nothing_is_made_outside_the_grids() {
        for form in Family::ALL.into_iter().flat_map(rolled) {
            assert!(
                cell_of(form.build, form.blade, form.handle).is_some(),
                "{form:?}"
            );
            assert_ne!(form.name(), "Blade", "{form:?}");
        }
    }

    #[test]
    fn a_short_blade_is_never_on_a_long_grip() {
        assert!(cell_of(Build::Along, 200, 300).is_none());
        assert!(cell_of(Build::Along, 500, 300).is_none());
    }

    #[test]
    fn short_blades_are_named_by_their_handle() {
        let named = |handle| sword(200, handle).name();
        assert!(["Knife", "Dagger"].contains(&named(120)));
        assert_eq!(
            cell_of(Build::Along, 200, 700).unwrap().generic,
            Some("Stabbing Spear")
        );
        assert_eq!(
            cell_of(Build::Along, 200, 2000).unwrap().generic,
            Some("Spear")
        );
        assert_eq!(
            cell_of(Build::Along, 200, 4000).unwrap().generic,
            Some("Pike")
        );
    }

    #[test]
    fn a_knife_with_two_edges_is_a_dagger() {
        let mut form = sword(200, 120);
        form.guard = Guard::Cup;
        form.pommel = Pommel::Wheel;
        form.edges = Edges::Single;
        form.curve = Curve::Wavy;
        assert_eq!(form.name(), "Knife");
        form.edges = Edges::Double;
        assert_eq!(form.name(), "Dagger");
    }

    #[test]
    fn a_katana_and_a_bastard_sword_share_a_cell_but_not_a_name() {
        let cell = |name: &str| {
            let family = ROWS.iter().find(|row| row.name == name).unwrap().family;
            rolled(family)
                .find(|form| form.name() == name && form.blade < 800)
                .map(|form| cell_of(form.build, form.blade, form.handle).unwrap().blade)
                .unwrap()
        };
        assert_eq!(cell("Katana"), cell("Bastard Sword"));
        let katana = rolled(Family::Sword)
            .find(|f| f.name() == "Katana")
            .unwrap();
        let bastard = rolled(Family::LongSword)
            .find(|f| f.name() == "Bastard Sword")
            .unwrap();
        assert_eq!((katana.edges, katana.curve), (Edges::Single, Curve::Slight));
        assert_eq!(
            (bastard.edges, bastard.curve),
            (Edges::Double, Curve::Straight)
        );
    }

    #[test]
    fn a_great_blade_in_one_hand_is_narrow() {
        for form in rolled(Family::Sword) {
            if form.blade >= GREAT.0 && form.handle < GRIP.1 {
                assert_eq!(form.width, BladeWidth::Narrow);
            }
        }
    }

    #[test]
    fn specifics_win_over_generics() {
        let rapier = rolled(Family::Sword).find(|f| f.blade >= GREAT.0).unwrap();
        assert_eq!(rapier.name(), "Rapier");
    }

    #[test]
    fn axes_have_a_head_at_the_side_and_nothing_along_the_handle() {
        for form in rolled(Family::Axe) {
            assert_eq!(form.build, Build::Side);
            assert_eq!(form.attachment(), Attachment::SideMounted);
            assert_ne!(form.top, Top::Blade);
            assert!(form.sides.contains(&SidePart::AxeBlade), "{form:?}");
        }
    }

    #[test]
    fn a_labrys_has_a_blade_each_side_and_a_halberd_differs() {
        let labrys = rolled(Family::Axe).find(|f| f.name() == "Labrys").unwrap();
        assert_eq!(labrys.sides, [SidePart::AxeBlade, SidePart::AxeBlade]);
        let halberd = rolled(Family::Polearm)
            .find(|f| f.name() == "Halberd")
            .unwrap();
        assert_eq!(halberd.sides[0], SidePart::AxeBlade);
        assert_ne!(halberd.sides[1], SidePart::None);
        assert_eq!(halberd.top, Top::Spike);
    }

    #[test]
    fn long_shafts_are_socketed_and_grips_have_tangs() {
        assert_eq!(sword(800, 150).attachment(), Attachment::Tang);
        let spear = rolled(Family::Spear).next().unwrap();
        assert_eq!(spear.attachment(), Attachment::Socket);
    }

    #[test]
    fn a_form_survives_being_saved() {
        let form = roll_blade(&mut Seed::new(3).rng(), Family::Polearm);
        let bytes = postcard::to_allocvec(&form).unwrap();
        assert_eq!(postcard::from_bytes::<BladeForm>(&bytes).unwrap(), form);
    }
}
