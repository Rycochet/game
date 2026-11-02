//! The named weapons. Sizes and popularity came from research recorded in
//! `docs/context/research/bladed-weapons.md`. A generic row is only there to roll from:
//! such a weapon takes the generic name of its cell. A specific row's name goes on
//! anything that fits it, however it was rolled.

use super::{
    BROAD_EDGE, Back, BladeForm, BladeShape, BladeWidth, Build, CELLS, Curve, Edges, Family, Guard,
    HandleShape, Hands, Point, Pommel, Ricasso, Section, SidePart, Span, Tips, Top, overlap,
    within,
};

/// What a name asks for. An empty list means anything will do.
#[derive(Clone, Copy, Debug)]
pub(super) struct Row {
    pub name: &'static str,
    pub family: Family,
    pub generic: bool,
    /// How often this is rolled in its family, from how well known the name is.
    pub weight: u64,
    pub build: Build,
    pub blade: Span,
    pub handle: Span,
    pub width: &'static [BladeWidth],
    pub edges: &'static [Edges],
    pub curve: &'static [Curve],
    pub point: &'static [Point],
    pub shape: &'static [BladeShape],
    pub section: &'static [Section],
    pub fuller: &'static [bool],
    pub hands: &'static [Hands],
    pub guard: &'static [Guard],
    pub pommel: &'static [Pommel],
    pub ricasso: &'static [Ricasso],
    pub langets: &'static [bool],
    pub handle_shape: &'static [HandleShape],
    pub side_one: &'static [SidePart],
    pub side_two: &'static [SidePart],
    pub top: &'static [Top],
    pub back: &'static [Back],
    pub tips: &'static [Tips],
}

impl Row {
    /// Every part of a size grid this row's lengths reach.
    pub fn places(&self) -> impl Iterator<Item = (Span, Span)> + '_ {
        let cells = CELLS.iter().filter(|cell| cell.build == self.build);
        cells.filter_map(|cell| {
            let blade = overlap(self.blade, cell.blade)?;
            let handle = overlap(self.handle, cell.handle)?;
            Some((blade, handle))
        })
    }

    pub fn fits(&self, form: &BladeForm) -> bool {
        fn open<T: PartialEq>(allowed: &[T], value: T) -> bool {
            allowed.is_empty() || allowed.contains(&value)
        }
        self.build == form.build
            && within(self.blade, form.blade)
            && within(self.handle, form.handle)
            && open(self.width, form.width)
            && open(self.edges, form.edges)
            && open(self.curve, form.curve)
            && open(self.point, form.point)
            && open(self.shape, form.shape)
            && open(self.section, form.section)
            && open(self.fuller, form.fuller)
            && open(self.hands, form.hands)
            && open(self.guard, form.guard)
            && open(self.pommel, form.pommel)
            && open(self.ricasso, form.ricasso)
            && open(self.langets, form.langets)
            && open(self.handle_shape, form.handle_shape)
            && open(self.side_one, form.sides[0])
            && open(self.side_two, form.sides[1])
            && open(self.top, form.top)
            && open(self.back, form.back)
            && open(self.tips, form.tips)
    }

    /// How many things this asks for, so the name asking for the most wins.
    pub fn specifics(&self) -> usize {
        [
            self.width.len(),
            self.edges.len(),
            self.curve.len(),
            self.point.len(),
            self.shape.len(),
            self.section.len(),
            self.fuller.len(),
            self.hands.len(),
            self.guard.len(),
            self.pommel.len(),
            self.ricasso.len(),
            self.langets.len(),
            self.handle_shape.len(),
            self.side_one.len(),
            self.side_two.len(),
            self.top.len(),
            self.back.len(),
            self.tips.len(),
        ]
        .iter()
        .filter(|allowed| **allowed > 0)
        .count()
    }
}

/// How often a row is rolled, by its popularity in books.
const COMMON: u64 = 8;
const KNOWN: u64 = 4;
const RARE: u64 = 2;
/// A family's generic row: most of what is made has no special name.
const USUAL: u64 = 16;

const BLANK: Row = Row {
    name: "",
    family: Family::Knife,
    generic: false,
    weight: KNOWN,
    build: Build::Along,
    blade: (0, 0),
    handle: (0, 0),
    width: &[],
    edges: &[],
    curve: &[],
    point: &[],
    shape: &[],
    section: &[],
    fuller: &[],
    hands: &[],
    guard: &[],
    pommel: &[],
    ricasso: &[],
    langets: &[],
    handle_shape: &[],
    side_one: &[],
    side_two: &[],
    top: &[],
    back: &[],
    tips: &[],
};

const GENERIC: Row = Row {
    generic: true,
    weight: USUAL,
    ..BLANK
};

/// An axe's or side-headed polearm's defaults.
const SIDE_HEAD: Row = Row {
    build: Build::Side,
    edges: &[Edges::Single],
    side_one: &[SidePart::AxeBlade],
    ..BLANK
};

const ONE: &[Hands] = &[Hands::One];
const TWO: &[Hands] = &[Hands::Two];
const EITHER: &[Hands] = &[Hands::OneOrTwo];
const SINGLE: &[Edges] = &[Edges::Single];
const DOUBLE: &[Edges] = &[Edges::Double];
const THRUST: &[Edges] = &[Edges::PointOnly];
const STRAIGHT: &[Curve] = &[Curve::Straight];
const SLIGHT: &[Curve] = &[Curve::Slight];
const STRONG: &[Curve] = &[Curve::Strong];
const INWARD: &[Curve] = &[Curve::Inward];
const LEAF: &[BladeShape] = &[BladeShape::Leaf];

// Best known first within each family.
pub(super) static ROWS: [Row; 85] = [
    // Knives
    Row {
        name: "Knife",
        family: Family::Knife,
        blade: (80, 300),
        handle: (80, 150),
        edges: SINGLE,
        ..GENERIC
    },
    Row {
        name: "Machete",
        family: Family::Knife,
        weight: COMMON,
        blade: (300, 660),
        handle: (100, 150),
        edges: SINGLE,
        curve: STRAIGHT,
        width: &[BladeWidth::Broad],
        guard: &[Guard::None],
        point: &[Point::Rounded, Point::Sharp],
        ..BLANK
    },
    Row {
        name: "Hunting Knife",
        family: Family::Knife,
        weight: COMMON,
        blade: (100, 200),
        handle: (100, 130),
        edges: SINGLE,
        curve: SLIGHT,
        point: &[Point::Sharp],
        guard: &[Guard::None, Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Bowie Knife",
        family: Family::Knife,
        weight: COMMON,
        blade: (240, 301),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        point: &[Point::Clip],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Parang",
        family: Family::Knife,
        blade: (250, 910),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        width: &[BladeWidth::Broad],
        shape: &[BladeShape::Widening],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Throwing Knife",
        family: Family::Knife,
        blade: (150, 300),
        handle: (80, 120),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::None],
        pommel: &[Pommel::None],
        hands: ONE,
        shape: &[BladeShape::Tapered],
        ..BLANK
    },
    Row {
        name: "Kukri",
        family: Family::Knife,
        blade: (260, 380),
        handle: (100, 150),
        edges: SINGLE,
        curve: INWARD,
        shape: &[BladeShape::Widening],
        point: &[Point::Sharp, Point::Notched],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Combat Knife",
        family: Family::Knife,
        blade: (150, 200),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        point: &[Point::Sharp, Point::Clip],
        ..BLANK
    },
    Row {
        name: "Seax",
        family: Family::Knife,
        weight: RARE,
        blade: (80, 750),
        handle: (100, 200),
        edges: SINGLE,
        curve: STRAIGHT,
        guard: &[Guard::None],
        pommel: &[Pommel::None, Pommel::Plain],
        point: &[Point::Clip],
        ..BLANK
    },
    Row {
        name: "Fighting Knife",
        family: Family::Knife,
        weight: RARE,
        blade: (150, 250),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Tapered],
        guard: &[Guard::Cross],
        ..BLANK
    },
    // Daggers
    Row {
        name: "Dagger",
        family: Family::Dagger,
        blade: (180, 300),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        ..GENERIC
    },
    Row {
        name: "Tanto",
        family: Family::Dagger,
        weight: COMMON,
        blade: (150, 300),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::None, Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Stiletto",
        family: Family::Dagger,
        weight: COMMON,
        blade: (200, 300),
        handle: (90, 110),
        edges: THRUST,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        section: &[Section::Diamond],
        ..BLANK
    },
    Row {
        name: "Dirk",
        family: Family::Dagger,
        weight: COMMON,
        blade: (200, 350),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Tapered],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Kris",
        family: Family::Dagger,
        weight: COMMON,
        blade: (300, 600),
        handle: (80, 120),
        edges: DOUBLE,
        curve: &[Curve::Wavy],
        handle_shape: &[HandleShape::Angled, HandleShape::Curved],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Poignard",
        family: Family::Dagger,
        weight: KNOWN,
        blade: (150, 300),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::Cross],
        section: &[Section::Diamond, Section::Lenticular],
        ..BLANK
    },
    Row {
        name: "Athame",
        family: Family::Dagger,
        weight: KNOWN,
        blade: (100, 200),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        pommel: &[Pommel::Plain],
        fuller: &[false],
        shape: &[BladeShape::Leaf, BladeShape::Tapered],
        ..BLANK
    },
    Row {
        name: "Kunai",
        family: Family::Dagger,
        blade: (200, 300),
        handle: (100, 150),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        guard: &[Guard::None],
        pommel: &[Pommel::Ring],
        ..BLANK
    },
    Row {
        name: "Rondel Dagger",
        family: Family::Dagger,
        blade: (300, 350),
        handle: (100, 200),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Disc],
        pommel: &[Pommel::Disc],
        ..BLANK
    },
    Row {
        name: "Pugio",
        family: Family::Dagger,
        blade: (180, 280),
        handle: (100, 131),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        width: &[BladeWidth::Broad],
        guard: &[Guard::Cross, Guard::None],
        ..BLANK
    },
    Row {
        name: "Parrying Dagger",
        family: Family::Dagger,
        blade: (350, 400),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cup, Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Misericorde",
        family: Family::Dagger,
        weight: RARE,
        blade: (200, 300),
        handle: (90, 110),
        edges: THRUST,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        section: &[Section::Triangular],
        guard: &[Guard::None, Guard::Cross],
        ..BLANK
    },
    // Short swords
    Row {
        name: "Short Sword",
        family: Family::ShortSword,
        blade: (350, 600),
        handle: (100, 150),
        ..GENERIC
    },
    Row {
        name: "Cutlass",
        family: Family::ShortSword,
        weight: COMMON,
        blade: (610, 760),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Cup],
        ..BLANK
    },
    Row {
        name: "Gladius",
        family: Family::ShortSword,
        weight: COMMON,
        blade: (450, 680),
        handle: (130, 170),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        guard: &[Guard::None, Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Falcata",
        family: Family::ShortSword,
        weight: KNOWN,
        blade: (350, 550),
        handle: (100, 130),
        edges: SINGLE,
        curve: INWARD,
        pommel: &[Pommel::Hooked],
        ..BLANK
    },
    Row {
        name: "Wakizashi",
        family: Family::ShortSword,
        blade: (300, 600),
        handle: (150, 200),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Yataghan",
        family: Family::ShortSword,
        blade: (550, 650),
        handle: (130, 170),
        edges: SINGLE,
        curve: INWARD,
        guard: &[Guard::None],
        pommel: &[Pommel::Disc, Pommel::Hooked],
        ..BLANK
    },
    Row {
        name: "Kopis",
        family: Family::ShortSword,
        weight: RARE,
        blade: (480, 650),
        handle: (100, 130),
        edges: SINGLE,
        curve: INWARD,
        guard: &[Guard::None, Guard::Cross],
        pommel: &[Pommel::Plain, Pommel::None],
        ..BLANK
    },
    Row {
        name: "Xiphos",
        family: Family::ShortSword,
        weight: RARE,
        blade: (450, 600),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        section: &[Section::Midrib, Section::Diamond, Section::Lenticular],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Khopesh",
        family: Family::ShortSword,
        weight: RARE,
        blade: (350, 450),
        handle: (120, 150),
        edges: SINGLE,
        curve: STRONG,
        guard: &[Guard::None],
        ..BLANK
    },
    // Swords
    Row {
        name: "Sword",
        family: Family::Sword,
        blade: (600, 1000),
        handle: (100, 200),
        ..GENERIC
    },
    Row {
        name: "Sabre",
        family: Family::Sword,
        weight: COMMON,
        blade: (700, 900),
        handle: (120, 150),
        edges: SINGLE,
        curve: &[Curve::Slight, Curve::Strong],
        guard: &[Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Rapier",
        family: Family::Sword,
        weight: COMMON,
        blade: (1000, 1300),
        handle: (120, 150),
        width: &[BladeWidth::Narrow],
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cup, Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Scimitar",
        family: Family::Sword,
        weight: COMMON,
        blade: (750, 900),
        handle: (120, 150),
        edges: SINGLE,
        curve: STRONG,
        guard: &[Guard::Cross],
        width: &[BladeWidth::Normal, BladeWidth::Broad],
        ..BLANK
    },
    Row {
        name: "Broadsword",
        family: Family::Sword,
        weight: COMMON,
        blade: (750, 900),
        handle: (120, 150),
        edges: DOUBLE,
        curve: STRAIGHT,
        width: &[BladeWidth::Broad],
        guard: &[Guard::Basket],
        ..BLANK
    },
    Row {
        name: "Katana",
        family: Family::Sword,
        weight: COMMON,
        blade: (600, 800),
        handle: (250, 300),
        edges: SINGLE,
        curve: SLIGHT,
        hands: TWO,
        guard: &[Guard::Disc],
        pommel: &[Pommel::Plain],
        ..BLANK
    },
    Row {
        name: "Falchion",
        family: Family::Sword,
        weight: KNOWN,
        blade: (700, 800),
        handle: (120, 150),
        edges: SINGLE,
        curve: SLIGHT,
        shape: &[BladeShape::Widening],
        width: &[BladeWidth::Broad],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Small Sword",
        family: Family::Sword,
        weight: KNOWN,
        blade: (600, 850),
        handle: (100, 130),
        edges: THRUST,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::Cup],
        ..BLANK
    },
    Row {
        name: "Tulwar",
        family: Family::Sword,
        weight: KNOWN,
        blade: (700, 850),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRONG,
        pommel: &[Pommel::Disc],
        guard: &[Guard::Cross, Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Spatha",
        family: Family::Sword,
        blade: (600, 750),
        handle: (180, 200),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Parallel],
        guard: &[Guard::Cross, Guard::None],
        ..BLANK
    },
    Row {
        name: "Backsword",
        family: Family::Sword,
        weight: RARE,
        blade: (750, 900),
        handle: (120, 150),
        edges: SINGLE,
        curve: STRAIGHT,
        guard: &[Guard::KnuckleBow, Guard::Basket],
        ..BLANK
    },
    Row {
        name: "Arming Sword",
        family: Family::Sword,
        weight: RARE,
        blade: (690, 810),
        handle: (100, 200),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        hands: ONE,
        pommel: &[Pommel::Wheel, Pommel::Disc],
        ..BLANK
    },
    Row {
        name: "Shamshir",
        family: Family::Sword,
        weight: RARE,
        blade: (750, 900),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRONG,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Jian",
        family: Family::Sword,
        blade: (450, 800),
        handle: (150, 250),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Parallel, BladeShape::Tapered],
        guard: &[Guard::Disc, Guard::Cross],
        pommel: &[Pommel::Plain, Pommel::Ring],
        ..BLANK
    },
    Row {
        name: "Dao",
        family: Family::Sword,
        blade: (650, 900),
        handle: (150, 200),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Disc],
        point: &[Point::Clip, Point::Flared],
        ..BLANK
    },
    // Long swords
    Row {
        name: "Longsword",
        family: Family::LongSword,
        blade: (800, 1100),
        handle: (200, 300),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        ..GENERIC
    },
    Row {
        name: "Greatsword",
        family: Family::LongSword,
        blade: (1000, 1500),
        handle: (300, 400),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: TWO,
        guard: &[Guard::Cross],
        ..GENERIC
    },
    Row {
        name: "Claymore",
        family: Family::LongSword,
        weight: COMMON,
        blade: (1000, 1150),
        handle: (300, 360),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: TWO,
        guard: &[Guard::Cross],
        pommel: &[Pommel::Wheel, Pommel::Disc],
        width: &[BladeWidth::Normal],
        ricasso: &[Ricasso::None],
        ..BLANK
    },
    Row {
        name: "Bastard Sword",
        family: Family::LongSword,
        weight: COMMON,
        blade: (750, 950),
        handle: (200, 280),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: EITHER,
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Odachi",
        family: Family::LongSword,
        weight: RARE,
        blade: (900, 1200),
        handle: (300, 400),
        edges: SINGLE,
        curve: SLIGHT,
        hands: TWO,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Zweihänder",
        family: Family::LongSword,
        weight: RARE,
        blade: (1000, 1500),
        handle: (300, 400),
        edges: DOUBLE,
        curve: &[Curve::Straight, Curve::Wavy],
        hands: TWO,
        ricasso: &[Ricasso::WithHooks],
        guard: &[Guard::Cross],
        ..BLANK
    },
    // Axes
    Row {
        name: "Axe",
        family: Family::Axe,
        blade: (80, 200),
        handle: (300, 900),
        edges: SINGLE,
        side_one: &[SidePart::AxeBlade],
        build: Build::Side,
        ..GENERIC
    },
    Row {
        name: "Battle Axe",
        family: Family::Axe,
        blade: (150, 250),
        handle: (600, 1200),
        edges: SINGLE,
        side_one: &[SidePart::AxeBlade],
        build: Build::Side,
        ..GENERIC
    },
    Row {
        name: "Hatchet",
        family: Family::Axe,
        weight: COMMON,
        blade: (60, 120),
        handle: (250, 400),
        hands: ONE,
        back: &[Back::None],
        side_two: &[SidePart::None],
        ..SIDE_HEAD
    },
    Row {
        name: "Tomahawk",
        family: Family::Axe,
        weight: COMMON,
        blade: (60, 101),
        handle: (400, 611),
        hands: ONE,
        back: &[Back::Spike, Back::Pipe],
        ..SIDE_HEAD
    },
    Row {
        name: "Broadaxe",
        family: Family::Axe,
        blade: (150, 300),
        handle: (600, 800),
        width: &[BladeWidth::Broad],
        back: &[Back::None],
        side_two: &[SidePart::None],
        ..SIDE_HEAD
    },
    Row {
        name: "Labrys",
        family: Family::Axe,
        blade: (100, 250),
        handle: (600, 1000),
        side_two: &[SidePart::AxeBlade],
        ..SIDE_HEAD
    },
    Row {
        name: "Throwing Axe",
        family: Family::Axe,
        blade: (80, 120),
        handle: (300, 450),
        hands: ONE,
        handle_shape: &[HandleShape::Straight],
        back: &[Back::None],
        width: &[BladeWidth::Narrow],
        ..SIDE_HEAD
    },
    Row {
        name: "Francisca",
        family: Family::Axe,
        weight: RARE,
        blade: (90, 110),
        handle: (400, 451),
        hands: ONE,
        curve: SLIGHT,
        back: &[Back::None],
        ..SIDE_HEAD
    },
    Row {
        name: "Tabar",
        family: Family::Axe,
        weight: RARE,
        blade: (150, 250),
        handle: (600, 1000),
        curve: STRONG,
        ..SIDE_HEAD
    },
    Row {
        name: "Dane Axe",
        family: Family::Axe,
        weight: RARE,
        blade: (200, 301),
        handle: (900, 1701),
        hands: TWO,
        curve: SLIGHT,
        width: &[BladeWidth::Narrow],
        back: &[Back::None],
        ..SIDE_HEAD
    },
    // Spears
    Row {
        name: "Spear",
        family: Family::Spear,
        blade: (100, 300),
        handle: (1500, 2200),
        ..GENERIC
    },
    Row {
        name: "Lance",
        family: Family::Spear,
        weight: COMMON,
        blade: (80, 150),
        handle: (2500, 3900),
        edges: DOUBLE,
        hands: ONE,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Javelin",
        family: Family::Spear,
        weight: COMMON,
        blade: (50, 150),
        handle: (1700, 2200),
        hands: ONE,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Harpoon",
        family: Family::Spear,
        weight: COMMON,
        blade: (100, 200),
        handle: (1200, 2000),
        tips: &[Tips::Barbed],
        ..BLANK
    },
    Row {
        name: "Trident",
        family: Family::Spear,
        weight: COMMON,
        blade: (150, 250),
        handle: (1200, 2000),
        edges: THRUST,
        tips: &[Tips::ThreeTines],
        ..BLANK
    },
    Row {
        name: "Pike",
        family: Family::Spear,
        weight: COMMON,
        blade: (150, 250),
        handle: (2800, 6800),
        edges: DOUBLE,
        hands: TWO,
        shape: LEAF,
        tips: &[Tips::Single],
        langets: &[true],
        ..BLANK
    },
    Row {
        name: "Assegai",
        family: Family::Spear,
        weight: KNOWN,
        blade: (200, 300),
        handle: (600, 700),
        hands: ONE,
        edges: DOUBLE,
        shape: LEAF,
        ..BLANK
    },
    Row {
        name: "Pilum",
        family: Family::Spear,
        weight: KNOWN,
        blade: (250, 350),
        handle: (1000, 1400),
        edges: THRUST,
        width: &[BladeWidth::Narrow],
        section: &[Section::Diamond, Section::Triangular],
        hands: ONE,
        tips: &[Tips::Single, Tips::Barbed],
        ..BLANK
    },
    Row {
        name: "Sarissa",
        family: Family::Spear,
        weight: KNOWN,
        blade: (200, 350),
        handle: (4500, 6800),
        edges: DOUBLE,
        hands: TWO,
        shape: LEAF,
        langets: &[false],
        ..BLANK
    },
    Row {
        name: "Yari",
        family: Family::Spear,
        blade: (150, 600),
        handle: (1000, 6000),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: TWO,
        section: &[Section::Triangular, Section::Diamond],
        langets: &[false],
        ..BLANK
    },
    // Polearms: blades along the pole
    Row {
        name: "Polearm",
        family: Family::Polearm,
        weight: KNOWN,
        blade: (350, 600),
        handle: (1200, 2500),
        edges: SINGLE,
        ..GENERIC
    },
    Row {
        name: "Glaive",
        family: Family::Polearm,
        weight: COMMON,
        blade: (400, 500),
        handle: (1700, 2000),
        edges: SINGLE,
        curve: SLIGHT,
        shape: &[BladeShape::Widening],
        ..BLANK
    },
    Row {
        name: "Naginata",
        family: Family::Polearm,
        weight: KNOWN,
        blade: (300, 600),
        handle: (1200, 2400),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Spontoon",
        family: Family::Polearm,
        blade: (150, 300),
        handle: (1600, 2100),
        edges: DOUBLE,
        shape: LEAF,
        side_one: &[SidePart::Lug],
        side_two: &[SidePart::Lug],
        ..BLANK
    },
    Row {
        name: "Guandao",
        family: Family::Polearm,
        weight: RARE,
        blade: (300, 460),
        handle: (1000, 1800),
        edges: SINGLE,
        curve: STRONG,
        width: &[BladeWidth::Broad],
        ..BLANK
    },
    Row {
        name: "Nagamaki",
        family: Family::Polearm,
        weight: RARE,
        blade: (600, 900),
        handle: (600, 900),
        edges: SINGLE,
        curve: SLIGHT,
        hands: TWO,
        guard: &[Guard::Disc],
        ..BLANK
    },
    // Polearms: heads at the side of the pole
    Row {
        name: "Halberd",
        family: Family::Polearm,
        weight: COMMON,
        blade: (200, 300),
        handle: (1800, 2200),
        side_two: &[SidePart::Spike, SidePart::Hook],
        top: &[Top::Spike],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Poleaxe",
        family: Family::Polearm,
        blade: BROAD_EDGE,
        handle: (1500, 2000),
        side_one: &[SidePart::AxeBlade],
        side_two: &[SidePart::Spike, SidePart::Beak],
        back: &[Back::Hammer, Back::Spike],
        top: &[Top::Spike],
        hands: TWO,
        langets: &[true],
        ..SIDE_HEAD
    },
    Row {
        name: "Bill",
        family: Family::Polearm,
        blade: (200, 400),
        handle: (1200, 2700),
        curve: INWARD,
        side_one: &[SidePart::Hook],
        side_two: &[SidePart::Spike],
        top: &[Top::Spike],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Lochaber Axe",
        family: Family::Polearm,
        weight: RARE,
        blade: (400, 500),
        handle: (1500, 1800),
        top: &[Top::None],
        side_two: &[SidePart::Hook],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Voulge",
        family: Family::Polearm,
        weight: RARE,
        blade: (300, 600),
        handle: (1800, 2500),
        side_two: &[SidePart::None],
        top: &[Top::Blade],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Guisarme",
        family: Family::Polearm,
        weight: RARE,
        blade: (200, 400),
        handle: (1800, 2500),
        curve: INWARD,
        side_one: &[SidePart::Hook],
        side_two: &[SidePart::Spike, SidePart::None],
        top: &[Top::Blade],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Bardiche",
        family: Family::Polearm,
        weight: RARE,
        blade: (600, 700),
        handle: (1200, 1501),
        curve: SLIGHT,
        side_two: &[SidePart::None],
        top: &[Top::None],
        hands: TWO,
        ..SIDE_HEAD
    },
];
