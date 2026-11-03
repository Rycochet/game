//! Weapons made of a handle with a blade or head on it: knives through to polearms.
//! A form is rolled once, within a row of the table, in a cell the size grid allows,
//! and named from what it turned out like. See `docs/context/shared/blade.md`.

use serde::{Deserialize, Serialize};

use crate::creature::Size;
use crate::item::{ItemKind, MAX_SHAPE_SIDE, Quality, Shape};
use crate::item_form::Limits;
use crate::material::Material;
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

    /// The kind of item this family is made as.
    pub const fn kind(self) -> ItemKind {
        match self {
            Self::Knife => ItemKind::Knife,
            Self::Dagger => ItemKind::Dagger,
            Self::ShortSword => ItemKind::ShortSword,
            Self::Sword => ItemKind::Sword,
            Self::LongSword => ItemKind::Longsword,
            Self::Axe => ItemKind::Axe,
            Self::Spear => ItemKind::Spear,
            Self::Polearm => ItemKind::Polearm,
        }
    }

    /// The other families to try if this one can't be made: first the smaller ones in
    /// any order, as those are likely to fit, then the larger, nearest first.
    fn others(self, rng: &mut Rng) -> Vec<Self> {
        let mut smaller: Vec<(Self, u64)> = Self::ALL[..self as usize]
            .iter()
            .map(|&family| (family, 1))
            .collect();
        let mut others = Vec::new();
        while !smaller.is_empty() {
            let family = rng.pick_weighted(&smaller);
            smaller.retain(|&(other, _)| other != family);
            others.push(family);
        }
        others.extend_from_slice(&Self::ALL[self as usize + 1..]);
        others
    }
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

/// How much has been done to make it look good.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Decoration {
    Plain,
    Engraved,
    Inlaid,
    Tasselled,
}

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
        let fitting = ROWS
            .iter()
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

/// How much of a weapon's length takes one more cell.
const CELL_LENGTH: u32 = 200;
/// A handle's section, a little over 3 cm across.
const HANDLE_AREA_MM2: u64 = 700;
/// The share out of 100 of weapons asked for by a creature smaller than ordinary that
/// are scaled even though an ordinary one fits, and for one larger.
const SCALED_PERCENT_SMALL: u64 = 10;
const SCALED_PERCENT_LARGE: u64 = 40;

/// How long it is from end to end, as far as carrying it goes: a head at the side adds
/// nothing to that.
fn length_of(build: Build, blade: u16, handle: u16) -> u16 {
    match build {
        Build::Along => blade + handle,
        Build::Side => handle,
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

/// A weapon rolled for a request, with the size it must be made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rolled {
    pub form: BladeForm,
    pub size: Size,
}

/// A new ordinary weapon of a family. Picks a row of the table, by how common it is, then
/// a cell it may be in, then each property in turn, within what the row allows.
pub fn roll_blade(rng: &mut Rng, family: Family, quality: Quality) -> BladeForm {
    roll_ordinary(rng, family, quality, &Limits::NONE).expect("nothing limits it")
}

/// A weapon that must be this size: if it isn't the ordinary one, it is scaled, and so
/// has only its generic name.
pub fn roll_blade_at(rng: &mut Rng, family: Family, quality: Quality, size: Size) -> BladeForm {
    if size == Size::Medium {
        return roll_blade(rng, family, quality);
    }
    roll_scaled(rng, family, quality, &Limits::NONE, size).expect("nothing limits it")
}

/// A weapon within limits. An ordinary one if any will do, of this family or else the
/// nearest that fits, so a tiny creature asking for a sword usually gets a dagger.
/// Sometimes, or if nothing ordinary fits, one of this family scaled, nearest its
/// wielder's size. `None` if nothing can be made.
pub fn roll_blade_within(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
) -> Option<Rolled> {
    let wielder = limits.wielder.unwrap_or(Size::Medium);
    let percent = if wielder < Size::Medium {
        SCALED_PERCENT_SMALL
    } else {
        SCALED_PERCENT_LARGE
    };
    let scale_first = wielder != Size::Medium && rng.range_u64(0, 100) < percent;
    if scale_first {
        let rolled = scaled_near(rng, family, quality, limits, wielder);
        if rolled.is_some() {
            return rolled;
        }
    }
    let mut families = vec![family];
    families.extend(family.others(rng));
    for other in families {
        if let Some(form) = roll_ordinary(rng, other, quality, limits) {
            let size = Size::Medium;
            return Some(Rolled { form, size });
        }
    }
    scaled_near(rng, family, quality, limits, wielder)
}

/// A specific weapon at any size, keeping its name: only when asked for, such as a
/// commission from a shop. `None` if there is no such name, or it can't keep to limits.
pub fn commission_blade(
    rng: &mut Rng,
    name: &str,
    quality: Quality,
    size: Size,
    limits: &Limits,
) -> Option<Rolled> {
    let row = ROWS.iter().find(|row| row.name == name)?;
    let form = roll_row_within(rng, row, quality, limits, size)?;
    Some(Rolled { form, size })
}

/// A wielder's limits: what it can lift without being burdened, and what suits its size.
/// Those smaller than ordinary can't manage anything long, and the largest have no use
/// for anything short.
pub fn limits_for_wielder(size: Size, spare_grams: u32) -> Limits {
    let (longest, shortest) = match size {
        Size::Tiny => (Some(2), None),
        Size::Small => (Some(4), None),
        Size::Medium | Size::Large => (None, None),
        Size::Huge => (None, Some(5)),
        Size::Giant => (None, Some(6)),
    };
    Limits {
        fits: longest.map(|longest| (2, longest)),
        at_least_long: shortest,
        most_grams: Some(spare_grams),
        least_grams: None,
        wielder: Some(size),
    }
}

/// An ordinary weapon of one family: rows that can't fit are passed over.
fn roll_ordinary(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
) -> Option<BladeForm> {
    let mut rows: Vec<(&Row, u64)> = ROWS
        .iter()
        .filter(|row| row.family == family)
        .map(|row| (row, row.weight))
        .collect();
    while !rows.is_empty() {
        let row = rng.pick_weighted(&rows);
        let form = roll_row_within(rng, row, quality, limits, Size::Medium);
        if form.is_some() {
            return form;
        }
        rows.retain(|(other, _)| other.name != row.name);
    }
    None
}

/// A generic weapon of one family at another size.
fn roll_scaled(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
    size: Size,
) -> Option<BladeForm> {
    let mut generic = ROWS
        .iter()
        .filter(|row| row.family == family && row.generic);
    let mut form = generic.find_map(|row| roll_row_within(rng, row, quality, limits, size))?;
    form.generic_only = true;
    Some(form)
}

/// Scaled, trying the sizes in order from the wielder's own outwards.
fn scaled_near(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
    wielder: Size,
) -> Option<Rolled> {
    let mut sizes: Vec<Size> = Size::ALL
        .into_iter()
        .filter(|&size| size != Size::Medium)
        .collect();
    sizes.sort_by_key(|&size| (size as i32 - wielder as i32).abs());
    sizes.into_iter().find_map(|size| {
        let form = roll_scaled(rng, family, quality, limits, size)?;
        Some(Rolled { form, size })
    })
}

/// The lengths at its ordinary size that keep a weapon scaled to `size` within limits.
/// `None` if none do.
fn lengths_within(limits: &Limits, size: Size) -> Option<Span> {
    let long_at = |cells: u8| Shape::rectangle(1, cells).scaled(size).height;
    let longest = match limits.fits {
        Some((one, other)) => {
            let room = one.max(other);
            (1..=MAX_SHAPE_SIDE)
                .rev()
                .find(|&cells| long_at(cells) <= room)?
        }
        None => MAX_SHAPE_SIDE,
    };
    let shortest = match limits.at_least_long {
        Some(least) => (1..=MAX_SHAPE_SIDE).find(|&cells| long_at(cells) >= least)?,
        None => 1,
    };
    let cell = CELL_LENGTH as u16;
    let from = match shortest {
        1 => 0,
        _ => shortest as u16 * cell - cell / 2,
    };
    let to = match longest {
        MAX_SHAPE_SIDE => u16::MAX,
        _ => longest as u16 * cell + cell / 2,
    };
    (from < to).then_some((from, to))
}

/// The handles from which some blade in `blade` gives a length within `lengths`.
fn handles_within(build: Build, blade: Span, handle: Span, lengths: Span) -> Option<Span> {
    match build {
        Build::Side => overlap(handle, lengths),
        Build::Along => {
            let from = lengths.0.saturating_sub(blade.1 - 1);
            let to = lengths.1.saturating_sub(blade.0);
            overlap(handle, (from, to))
        }
    }
}

/// The blades that, with this handle, give a length within `lengths`.
fn blades_within(build: Build, blade: Span, handle: u16, lengths: Span) -> Option<Span> {
    match build {
        Build::Side => Some(blade),
        Build::Along => {
            let from = lengths.0.saturating_sub(handle);
            let to = lengths.1.saturating_sub(handle);
            overlap(blade, (from, to))
        }
    }
}

/// The first number in a span that passes, given that all those after it pass too, or
/// the span's end if none do.
fn first_passing(span: Span, passes: impl Fn(u16) -> bool) -> u16 {
    let (mut low, mut high) = span;
    while low < high {
        let middle = low + (high - low) / 2;
        if passes(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// The blades in a span that keep this weapon's weight within limits: a longer one is
/// never lighter.
fn blades_by_weight(form: &BladeForm, blade: Span, size: Size, limits: &Limits) -> Option<Span> {
    let grams = |length: u16| {
        let longer = BladeForm {
            blade: length,
            ..*form
        };
        longer.grams_at(size)
    };
    let from = match limits.least_grams {
        Some(least) => first_passing(blade, |length| grams(length) >= least),
        None => blade.0,
    };
    let to = match limits.most_grams {
        Some(most) => first_passing(blade, |length| grams(length) > most),
        None => blade.1,
    };
    (from < to).then_some((from, to))
}

/// One row within limits, at a size. Lengths are cut down to what fits rather than
/// rolled again. If its weight still can't fit, it tries the lightest it could be made
/// of before giving up on this row.
fn roll_row_within(
    rng: &mut Rng,
    row: &Row,
    quality: Quality,
    limits: &Limits,
    size: Size,
) -> Option<BladeForm> {
    let lengths = lengths_within(limits, size)?;
    let places: Vec<((Span, Span), u64)> = row
        .places()
        .filter_map(|(blade, handle)| {
            let handle = handles_within(row.build, blade, handle, lengths)?;
            Some(((blade, handle), 1))
        })
        .collect();
    if places.is_empty() {
        return None;
    }
    let (blade, handle) = rng.pick_weighted(&places);
    let handle = rng.range_u64(handle.0 as u64, handle.1 as u64) as u16;
    let blade = blades_within(row.build, blade, handle, lengths)?;
    let mut form = roll_properties(rng, row, quality, blade.0, handle);
    let span = blades_by_weight(&form, blade, size, limits).or_else(|| {
        form.width = row.width.first().copied().unwrap_or(BladeWidth::Narrow);
        form.blade_metal = Material::Iron;
        form.handle_material = Material::Wood;
        blades_by_weight(&form, blade, size, limits)
    })?;
    form.blade = rng.range_u64(span.0 as u64, span.1 as u64) as u16;
    limits
        .allows(form.shape().scaled(size), form.grams_at(size))
        .then_some(form)
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
fn roll_properties(
    rng: &mut Rng,
    row: &Row,
    quality: Quality,
    blade: u16,
    handle: u16,
) -> BladeForm {
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
    // Better work goes into finer things, more often.
    let fine = quality as u64;
    let blade_metal = rng.pick_weighted(&[
        (Material::Iron, 80),
        (Material::Copper, 10),
        (Material::Silver, 2 + fine * 3),
        (Material::Gold, fine),
    ]);
    let handle_material = if held {
        rng.pick_weighted(&[
            (Material::Wood, 60),
            (Material::Bone, 25),
            (Material::Iron, 15),
        ])
    } else {
        Material::Wood
    };
    let fittings = rng.pick_weighted(&[
        (Material::Iron, 60),
        (Material::Copper, 25),
        (Material::Silver, 5 + fine * 5),
        (Material::Gold, 1 + fine * 3),
    ]);
    let decoration = rng.pick_weighted(&[
        (Decoration::Plain, 70),
        (Decoration::Engraved, 10 + fine * 8),
        (Decoration::Inlaid, 2 + fine * 5),
        (Decoration::Tasselled, 5),
    ]);
    BladeForm {
        family: row.family,
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
        blade_metal,
        handle_material,
        fittings,
        decoration,
        generic_only: false,
    }
}

#[cfg(test)]
mod tests;
