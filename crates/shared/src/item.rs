//! Items, and carrying them in grids: the rules both sides check moves by. See
//! `docs/context/shared/item.md`.
//!
//! What a kind of item is (`ItemKind::def`) is a table here. An item itself is
//! its id and kind, and what is inside it. Everything a character carries is a
//! tree starting from their pockets: a grid of placed items, some of which have
//! grids of their own.

use serde::{Deserialize, Serialize};

use crate::creature::{CreatureKind, Layer, Location, Size};
use crate::id::Id;
use crate::rng::Rng;
use crate::world::WorldId;

/// The widest and tallest a shape can be.
pub const MAX_SHAPE_SIDE: u8 = 8;

/// How many grids deep things can go below the pockets: backpack, pouch, box.
pub const MAX_DEPTH: u32 = 3;

/// The pockets of a character saved before creature kinds, and of a new
/// `Inventory`. Otherwise they come from the kind of creature.
pub const POCKETS: (u8, u8) = (6, 4);

/// The cells something takes up, within its width and height: bit `y * 8 + x`
/// of `cells`, so 8 × 8 fits in one number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub width: u8,
    pub height: u8,
    cells: u64,
}

impl Shape {
    /// A rectangle.
    pub const fn rectangle(width: u8, height: u8) -> Self {
        let mut cells = 0;
        let mut y = 0;
        while y < height {
            let mut x = 0;
            while x < width {
                cells |= 1 << (y as u32 * 8 + x as u32);
                x += 1;
            }
            y += 1;
        }
        Self {
            width,
            height,
            cells,
        }
    }

    /// A rectangle without some of its cells (bits as in `cells`).
    const fn without(self, missing: u64) -> Self {
        Self {
            cells: self.cells & !missing,
            ..self
        }
    }

    pub fn has(self, x: u8, y: u8) -> bool {
        x < self.width && y < self.height && self.cells >> (y as u32 * 8 + x as u32) & 1 == 1
    }

    /// Its cells, as `(x, y)`.
    pub fn cells(self) -> impl Iterator<Item = (u8, u8)> {
        (0..self.height)
            .flat_map(move |y| (0..self.width).map(move |x| (x, y)))
            .filter(move |&(x, y)| self.has(x, y))
    }

    /// How many cells it takes up.
    pub fn size(self) -> u32 {
        self.cells.count_ones()
    }

    /// Turned this way.
    pub fn turned(self, rotation: Rotation) -> Self {
        let (width, height) = match rotation {
            Rotation::None | Rotation::Half => (self.width, self.height),
            Rotation::Quarter | Rotation::ThreeQuarters => (self.height, self.width),
        };
        let mut cells = 0;
        for (x, y) in self.cells() {
            let (new_x, new_y) = rotation.turn_cell(x, y, self.width, self.height);
            cells |= 1 << (new_y as u32 * 8 + new_x as u32);
        }
        Self {
            width,
            height,
            cells,
        }
    }

    /// Whether something of this shape can go inside something whose outside is
    /// `outer`: it has fewer cells, and fits within its sides, turned or not.
    /// Fewer every time is what stops things nesting for ever.
    pub fn smaller_than(self, outer: Shape) -> bool {
        let fits_as_it_is = self.width <= outer.width && self.height <= outer.height;
        let fits_turned = self.height <= outer.width && self.width <= outer.height;
        self.size() < outer.size() && (fits_as_it_is || fits_turned)
    }
}

/// How far something is turned, clockwise.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    None,
    Quarter,
    Half,
    ThreeQuarters,
}

impl Rotation {
    pub const ALL: [Self; 4] = [Self::None, Self::Quarter, Self::Half, Self::ThreeQuarters];

    /// A quarter turn further on.
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Quarter,
            Self::Quarter => Self::Half,
            Self::Half => Self::ThreeQuarters,
            Self::ThreeQuarters => Self::None,
        }
    }

    /// Where a cell of something `width` by `height` goes when it is turned.
    pub fn turn_cell(self, x: u8, y: u8, width: u8, height: u8) -> (u8, u8) {
        match self {
            Self::None => (x, y),
            Self::Quarter => (height - 1 - y, x),
            Self::Half => (width - 1 - x, height - 1 - y),
            Self::ThreeQuarters => (y, width - 1 - x),
        }
    }
}

/// The grid inside a container, and what it will take. Anything put in must also
/// be `Shape::smaller_than` the container's outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerDef {
    pub width: u8,
    pub height: u8,
    /// The most all its contents may weigh, in grams.
    pub max_total: Option<u32>,
    /// The most any one thing in it may weigh, in grams, with its contents.
    pub max_each: Option<u32>,
}

/// What a kind of item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDef {
    pub name: &'static str,
    /// In grams, when empty.
    pub weight: u32,
    pub shape: Shape,
    pub container: Option<ContainerDef>,
    pub wear: Option<WearDef>,
    /// Whether how well it is made matters: not for a coin.
    pub has_quality: bool,
}

/// How something is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WearDef {
    pub layer: Layer,
    pub locations: &'static [Location],
    /// It takes only one place of its one location: a ring on a finger.
    pub one_place: bool,
    /// How many sizes smaller and larger than it was made for it can be worn.
    pub down: u8,
    pub up: u8,
    /// It adjusts, so it fits properly anywhere in that range. Otherwise anything
    /// but its own size is loose.
    pub adjustable: bool,
}

/// A kind of item. Saved by name, so new ones can go anywhere in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Apple,
    Backpack,
    Barding,
    Belt,
    Boomerang,
    Boots,
    Cloak,
    Coin,
    FingerlessGloves,
    FullArmour,
    Gauntlets,
    Gloves,
    Hat,
    Lantern,
    Necklace,
    Pouch,
    Ring,
    Shoes,
    SmallBox,
    Socks,
    Stone,
    Sword,
    TShirt,
    Trousers,
}

/// All but its top-right cell: an L.
const BOOMERANG: Shape = Shape::rectangle(2, 2).without(1 << 1);

impl ItemKind {
    /// Every kind, alphabetically by name.
    pub const ALL: [Self; 24] = [
        Self::Apple,
        Self::Backpack,
        Self::Barding,
        Self::Belt,
        Self::Boomerang,
        Self::Boots,
        Self::Cloak,
        Self::Coin,
        Self::FingerlessGloves,
        Self::FullArmour,
        Self::Gauntlets,
        Self::Gloves,
        Self::Hat,
        Self::Lantern,
        Self::Necklace,
        Self::Pouch,
        Self::Ring,
        Self::Shoes,
        Self::SmallBox,
        Self::Socks,
        Self::Stone,
        Self::Sword,
        Self::TShirt,
        Self::Trousers,
    ];

    pub const fn def(self) -> ItemDef {
        use Layer::{Middle, Outer, Skin};
        use Location::{
            Arms, Feet, Fingers, Forelegs, Hands, Head, Hindlegs, Legs, Neck, Shoulders, Torso,
            Waist,
        };
        let square = Shape::rectangle(1, 1);
        match self {
            Self::Apple => plain("Apple", 150, square).without_quality(),
            Self::Backpack => bag(
                "Backpack",
                1200,
                Shape::rectangle(3, 4),
                (10, 12),
                Some(25_000),
                None,
            )
            .worn(wear(Outer, &[Shoulders], (1, 1), true)),
            Self::Barding => plain("Barding", 15_000, Shape::rectangle(4, 4)).worn(wear(
                Outer,
                &[Neck, Torso, Forelegs, Hindlegs],
                (0, 0),
                false,
            )),
            Self::Belt => plain("Belt", 200, Shape::rectangle(1, 3)).worn(wear(
                Middle,
                &[Waist],
                (2, 2),
                true,
            )),
            Self::Boomerang => plain("Boomerang", 400, BOOMERANG),
            Self::Boots => plain("Boots", 1100, Shape::rectangle(2, 2)).worn(wear(
                Outer,
                &[Feet],
                (1, 0),
                false,
            )),
            Self::Cloak => plain("Cloak", 900, Shape::rectangle(2, 3)).worn(wear(
                Middle,
                &[Shoulders],
                (1, 1),
                true,
            )),
            Self::Coin => plain("Coin", 10, square).without_quality(),
            Self::FingerlessGloves => plain("Fingerless Gloves", 80, Shape::rectangle(1, 2))
                .worn(wear(Middle, &[Hands], (1, 1), true)),
            Self::FullArmour => plain("Full Armour", 20_000, Shape::rectangle(3, 4)).worn(wear(
                Outer,
                &[Shoulders, Arms, Torso, Legs],
                (0, 0),
                false,
            )),
            Self::Gauntlets => plain("Gauntlets", 1200, Shape::rectangle(2, 1)).worn(wear(
                Outer,
                &[Hands, Fingers],
                (0, 0),
                false,
            )),
            Self::Gloves => plain("Gloves", 100, Shape::rectangle(1, 2)).worn(wear(
                Middle,
                &[Hands, Fingers],
                (1, 0),
                false,
            )),
            Self::Hat => {
                plain("Hat", 150, Shape::rectangle(2, 1)).worn(wear(Outer, &[Head], (1, 0), false))
            }
            Self::Lantern => plain("Lantern", 800, Shape::rectangle(1, 2)),
            Self::Necklace => plain("Necklace", 40, square).worn(wear(Skin, &[Neck], (5, 5), true)),
            Self::Pouch => bag(
                "Pouch",
                80,
                Shape::rectangle(2, 2),
                (4, 4),
                Some(1500),
                Some(500),
            ),
            Self::Ring => plain("Ring", 10, square).worn(WearDef {
                one_place: true,
                ..wear(Skin, &[Fingers], (1, 1), true)
            }),
            Self::Shoes => plain("Shoes", 500, Shape::rectangle(2, 1)).worn(wear(
                Middle,
                &[Feet],
                (1, 0),
                false,
            )),
            Self::SmallBox => bag(
                "Small Box",
                100,
                Shape::rectangle(2, 1),
                (3, 2),
                Some(300),
                None,
            ),
            Self::Socks => plain("Socks", 50, square).worn(wear(Skin, &[Feet], (1, 1), true)),
            Self::Stone => plain("Stone", 5000, Shape::rectangle(2, 2)).without_quality(),
            Self::Sword => plain("Sword", 1400, Shape::rectangle(1, 5)),
            Self::TShirt => plain("T-Shirt", 200, Shape::rectangle(2, 2)).worn(wear(
                Skin,
                &[Torso, Arms],
                (1, 0),
                false,
            )),
            Self::Trousers => plain("Trousers", 500, Shape::rectangle(2, 3)).worn(wear(
                Middle,
                &[Legs],
                (1, 0),
                false,
            )),
        }
    }
}

const fn plain(name: &'static str, weight: u32, shape: Shape) -> ItemDef {
    ItemDef {
        name,
        weight,
        shape,
        container: None,
        wear: None,
        has_quality: true,
    }
}

const fn bag(
    name: &'static str,
    weight: u32,
    shape: Shape,
    (width, height): (u8, u8),
    max_total: Option<u32>,
    max_each: Option<u32>,
) -> ItemDef {
    ItemDef {
        container: Some(ContainerDef {
            width,
            height,
            max_total,
            max_each,
        }),
        ..plain(name, weight, shape)
    }
}

const fn wear(
    layer: Layer,
    locations: &'static [Location],
    (down, up): (u8, u8),
    adjustable: bool,
) -> WearDef {
    WearDef {
        layer,
        locations,
        one_place: false,
        down,
        up,
        adjustable,
    }
}

impl ItemDef {
    const fn without_quality(self) -> Self {
        Self {
            has_quality: false,
            ..self
        }
    }

    const fn worn(self, wear: WearDef) -> Self {
        Self {
            wear: Some(wear),
            ..self
        }
    }
}

/// Who an item was made for. Part of the item itself: one kind of glove comes in
/// any size and number of fingers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Made {
    /// The size it fits.
    pub size: Size,
    /// The smallest and largest it can be worn by at all.
    pub smallest: Size,
    pub largest: Size,
    /// How many fingers it has, if it covers them.
    pub fingers: Option<u8>,
    /// It resizes itself to fit anyone, whatever their fingers, but not whatever
    /// their shape.
    pub magical: bool,
}

impl Made {
    /// A thing of this kind made for a creature of this size and these fingers.
    pub fn for_body(kind: ItemKind, size: Size, fingers: Option<u8>, magical: bool) -> Self {
        let wear = kind.def().wear;
        let covers_fingers =
            wear.is_some_and(|wear| !wear.one_place && wear.locations.contains(&Location::Fingers));
        Self {
            size,
            smallest: size.stepped(-(wear.map_or(0, |wear| wear.down) as i8)),
            largest: size.stepped(wear.map_or(0, |wear| wear.up) as i8),
            fingers: if covers_fingers { fingers } else { None },
            magical,
        }
    }
}

/// How good an item is, worst first. The better, the rarer. Set items aren't
/// here: they aren't rolled, and come from stories (see `decisions.md`).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Quality {
    Poor,
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Artifact,
}

impl Quality {
    pub const ALL: [Quality; 7] = [
        Self::Poor,
        Self::Common,
        Self::Uncommon,
        Self::Rare,
        Self::Epic,
        Self::Legendary,
        Self::Artifact,
    ];

    /// How likely a roll is to give it, out of `ODDS_OUT_OF`. First guesses.
    pub const fn odds(self) -> u64 {
        match self {
            Self::Poor => 2500,
            Self::Common => 5000,
            Self::Uncommon => 1700,
            Self::Rare => 600,
            Self::Epic => 150,
            Self::Legendary => 45,
            Self::Artifact => 5,
        }
    }

    /// The first of a drop's two rolls: how good it will be.
    pub fn roll(rng: &mut Rng) -> Self {
        let mut left = rng.range_u64(0, ODDS_OUT_OF);
        for quality in Self::ALL {
            if left < quality.odds() {
                return quality;
            }
            left -= quality.odds();
        }
        Self::Common
    }
}

/// What `Quality::odds` add up to.
pub const ODDS_OUT_OF: u64 = 10_000;

/// One line of a drop table: this kind can drop at this quality, more likely the
/// more it weighs against the others at that quality.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drop {
    pub quality: Quality,
    pub kind: ItemKind,
    pub weight: u64,
}

/// Both rolls of a drop: first the quality, then the item from those at that
/// quality. If the table has none so good, it takes the best it has below that,
/// or failing that the worst above. `None` if the table is empty. A kind without
/// quality comes back without one.
pub fn roll_drop(rng: &mut Rng, table: &[Drop]) -> Option<(ItemKind, Option<Quality>)> {
    let rolled = Quality::roll(rng);
    let at = table
        .iter()
        .map(|drop| drop.quality)
        .filter(|&quality| quality <= rolled)
        .max()
        .or_else(|| table.iter().map(|drop| drop.quality).min())?;
    let lines = || table.iter().filter(|drop| drop.quality == at);
    let total: u64 = lines().map(|drop| drop.weight).sum();
    let mut left = rng.range_u64(0, total.max(1));
    let mut chosen = None;
    for drop in lines() {
        chosen = Some(drop.kind);
        if left < drop.weight {
            break;
        }
        left -= drop.weight;
    }
    let kind = chosen?;
    Some((kind, kind.def().has_quality.then_some(at)))
}

/// When and where an item was made. Each world has its own clock, so a minute
/// means nothing without its world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MadeAt {
    pub world: WorldId,
    /// Game minutes on that world's clock.
    pub minute: u64,
}

/// One item: it keeps its id for good, wherever it goes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: Id,
    pub kind: ItemKind,
    /// What is in its grid, if it has one.
    pub contents: Vec<Placed>,
    pub made: Made,
    /// How good it is: it can change later, and it is still the same item. `None`
    /// for kinds without quality, and things saved before quality.
    #[serde(default)]
    pub quality: Option<Quality>,
    /// `None` for things saved before this was kept.
    #[serde(default)]
    pub made_at: Option<MadeAt>,
}

impl Item {
    pub fn new(id: Id, kind: ItemKind) -> Self {
        Self {
            id,
            kind,
            contents: Vec::new(),
            made: Made::for_body(kind, Size::Medium, Some(10), false),
            quality: None,
            made_at: None,
        }
    }

    /// What it weighs with everything in it, in grams.
    pub fn total_weight(&self) -> u32 {
        self.kind.def().weight + weight_of(&self.contents)
    }

    /// How many grids deep it is, itself and what could be in it now: 0 if it
    /// has no grid.
    fn grids_deep(&self) -> u32 {
        if self.kind.def().container.is_none() {
            return 0;
        }
        let inside = self.contents.iter().map(|placed| placed.item.grids_deep());
        1 + inside.max().unwrap_or(0)
    }

    /// Whether `id` is this item or somewhere inside it.
    fn holds_or_is(&self, id: Id) -> bool {
        self.id == id
            || self
                .contents
                .iter()
                .any(|placed| placed.item.holds_or_is(id))
    }
}

fn weight_of(placed: &[Placed]) -> u32 {
    placed.iter().map(|placed| placed.item.total_weight()).sum()
}

/// An item in a grid: the cell its shape's top left is on, and how it is turned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placed {
    pub item: Item,
    pub x: u8,
    pub y: u8,
    pub rotation: Rotation,
}

impl Placed {
    /// Its shape as it lies.
    pub fn shape(&self) -> Shape {
        self.item.kind.def().shape.turned(self.rotation)
    }

    /// The grid cells it takes up.
    pub fn cells(&self) -> impl Iterator<Item = (u8, u8)> {
        let (left, top) = (self.x, self.y);
        self.shape().cells().map(move |(x, y)| (left + x, top + y))
    }
}

/// A grid things can be in: a character's pockets or a container's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Holder {
    Pockets,
    Inside(Id),
}

/// Why something can't go where it was asked to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveRefused {
    /// The item isn't there to move.
    NoSuchItem,
    /// The grid isn't there, or what was named has none.
    NoSuchGrid,
    /// It would be inside itself.
    IntoItself,
    /// It is too big for the container's outside.
    TooBig,
    /// It would go too many grids deep.
    TooDeep,
    /// It would stick out of the grid, or lie on something else.
    NoRoom,
    /// It is too heavy for a container it would end up in.
    TooHeavy,
    /// It is worn, under something on a higher layer.
    Covered,
}

/// Everything a character carries: what is in their pockets and what they wear.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// What kind of creature is carrying it all, which decides what they can wear.
    pub creature: CreatureKind,
    /// They have had their first clothes.
    pub outfitted: bool,
    pub pockets: Vec<Placed>,
    /// How big the pockets are, from their kind of creature.
    pub pockets_size: (u8, u8),
    /// What they have on: see `shared::wear`.
    pub worn: Vec<Item>,
    /// What they last threw away: it can be taken back until something else is
    /// thrown away, which destroys it. It weighs nothing, being left behind.
    #[serde(default)]
    pub bin: Option<Item>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            creature: CreatureKind::default(),
            outfitted: false,
            pockets: Vec::new(),
            pockets_size: POCKETS,
            worn: Vec::new(),
            bin: None,
        }
    }
}

impl Inventory {
    /// What it all weighs, in grams.
    pub fn total_weight(&self) -> u32 {
        weight_of(&self.pockets) + self.worn.iter().map(Item::total_weight).sum::<u32>()
    }

    /// What the creature weighs with all it carries and wears, in grams.
    pub fn weight_with_creature(&self) -> u32 {
        self.creature.def().weight + self.total_weight()
    }

    /// The item with this id, wherever it is.
    pub fn find(&self, id: Id) -> Option<&Item> {
        fn search(placed: &[Placed], id: Id) -> Option<&Item> {
            placed.iter().find_map(|placed| {
                if placed.item.id == id {
                    Some(&placed.item)
                } else {
                    search(&placed.item.contents, id)
                }
            })
        }
        let carried = search(&self.pockets, id).or_else(|| {
            self.worn.iter().find_map(|item| {
                if item.id == id {
                    Some(item)
                } else {
                    search(&item.contents, id)
                }
            })
        });
        carried.or_else(|| self.bin.as_ref().filter(|item| item.id == id))
    }

    /// What is in this grid: `None` if there is no such grid.
    pub fn grid(&self, holder: Holder) -> Option<&[Placed]> {
        match holder {
            Holder::Pockets => Some(&self.pockets),
            Holder::Inside(id) => {
                let item = self.find(id)?;
                item.kind.def().container?;
                Some(&item.contents)
            }
        }
    }

    /// How wide and tall this grid is.
    pub fn grid_size(&self, holder: Holder) -> Option<(u8, u8)> {
        match holder {
            Holder::Pockets => Some(self.pockets_size),
            Holder::Inside(id) => {
                let container = self.find(id)?.kind.def().container?;
                Some((container.width, container.height))
            }
        }
    }

    /// The containers leading to `id` from the pockets, outermost first (not
    /// `id` itself): `None` if it isn't here.
    fn containers_around(&self, id: Id) -> Option<Vec<Id>> {
        fn search(placed: &[Placed], id: Id, around: &mut Vec<Id>) -> bool {
            placed.iter().any(|placed| {
                if placed.item.id == id {
                    return true;
                }
                around.push(placed.item.id);
                let found = search(&placed.item.contents, id, around);
                if !found {
                    around.pop();
                }
                found
            })
        }
        let mut around = Vec::new();
        if search(&self.pockets, id, &mut around) {
            return Some(around);
        }
        for item in &self.worn {
            if item.id == id {
                return Some(Vec::new());
            }
            let mut around = vec![item.id];
            if search(&item.contents, id, &mut around) {
                return Some(around);
            }
        }
        None
    }

    /// Which grid an item is in, and how it lies there. None if it isn't in a
    /// grid, as when it is worn.
    pub fn placed(&self, id: Id) -> Option<(Holder, &Placed)> {
        if self.is_worn(id) || self.in_bin(id) {
            return None;
        }
        let holder = match self.containers_around(id)?.last() {
            Some(&container) => Holder::Inside(container),
            None => Holder::Pockets,
        };
        let placed = self
            .grid(holder)?
            .iter()
            .find(|placed| placed.item.id == id)?;
        Some((holder, placed))
    }

    /// Whether `shape` fits at `(x, y)` in this grid, not counting `moving`,
    /// which is lifted out.
    fn has_room(&self, holder: Holder, shape: Shape, x: u8, y: u8, moving: Option<Id>) -> bool {
        let (Some(grid), Some((width, height))) = (self.grid(holder), self.grid_size(holder))
        else {
            return false;
        };
        let inside = x as u32 + shape.width as u32 <= width as u32
            && y as u32 + shape.height as u32 <= height as u32;
        inside
            && !grid
                .iter()
                .filter(|placed| Some(placed.item.id) != moving)
                .flat_map(Placed::cells)
                .any(|(cell_x, cell_y)| {
                    cell_x >= x && cell_y >= y && shape.has(cell_x - x, cell_y - y)
                })
    }

    /// Whether this carried item may go in this grid, wherever in it: all but
    /// the room.
    fn may_hold(&self, holder: Holder, item: &Item) -> Result<(), MoveRefused> {
        let Holder::Inside(target) = holder else {
            return Ok(());
        };
        if item.holds_or_is(target) {
            return Err(MoveRefused::IntoItself);
        }
        let target_def = self.find(target).ok_or(MoveRefused::NoSuchGrid)?.kind.def();
        if target_def.container.is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        if !item.kind.def().shape.smaller_than(target_def.shape) {
            return Err(MoveRefused::TooBig);
        }
        let mut chain = self
            .containers_around(target)
            .ok_or(MoveRefused::NoSuchGrid)?;
        chain.push(target);
        if chain.len() as u32 + item.grids_deep() > MAX_DEPTH {
            return Err(MoveRefused::TooDeep);
        }
        // From the inside out: each container it would end up in, with what
        // would lie directly in that one on the way to the item. A container it
        // is in already gains nothing.
        let weight = item.total_weight();
        let mut lying_in_it = weight;
        for &container in chain.iter().rev() {
            let holding = self.find(container).ok_or(MoveRefused::NoSuchGrid)?;
            let def = holding
                .kind
                .def()
                .container
                .ok_or(MoveRefused::NoSuchGrid)?;
            let gained = if holding.holds_or_is(item.id) {
                0
            } else {
                weight
            };
            let total = weight_of(&holding.contents) + gained;
            if def.max_total.is_some_and(|most| total > most)
                || def.max_each.is_some_and(|most| lying_in_it > most)
            {
                return Err(MoveRefused::TooHeavy);
            }
            lying_in_it = holding.total_weight() + gained;
        }
        Ok(())
    }

    /// Whether this carried item can go here, turned this way.
    pub fn check_move(
        &self,
        id: Id,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        let item = self.find(id).ok_or(MoveRefused::NoSuchItem)?;
        if self.covered_by(id).is_some() {
            return Err(MoveRefused::Covered);
        }
        self.may_hold(holder, item)?;
        let shape = item.kind.def().shape.turned(rotation);
        if self.grid(holder).is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        if self.has_room(holder, shape, x, y, Some(id)) {
            Ok(())
        } else {
            Err(MoveRefused::NoRoom)
        }
    }

    /// Moves a carried item here, if it can go.
    pub fn move_item(
        &mut self,
        id: Id,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        self.check_move(id, holder, x, y, rotation)?;
        let item = self.take(id).ok_or(MoveRefused::NoSuchItem)?;
        self.grid_mut(holder)
            .expect("checked above, and not inside what was lifted out")
            .push(Placed {
                item,
                x,
                y,
                rotation,
            });
        Ok(())
    }

    /// Throws a carried item away, with everything in it, destroying what was thrown
    /// away before. `false` if it isn't carried, is worn under something, or is
    /// there already.
    pub fn to_bin(&mut self, id: Id) -> bool {
        if self.in_bin(id) || self.covered_by(id).is_some() {
            return false;
        }
        let Some(item) = self.take(id) else {
            return false;
        };
        self.bin = Some(item);
        true
    }

    /// Whether this is what was last thrown away.
    pub fn in_bin(&self, id: Id) -> bool {
        self.bin.as_ref().is_some_and(|item| item.id == id)
    }

    /// Takes an item out, wherever it is.
    pub(crate) fn take(&mut self, id: Id) -> Option<Item> {
        fn search(placed: &mut Vec<Placed>, id: Id) -> Option<Item> {
            if let Some(index) = placed.iter().position(|placed| placed.item.id == id) {
                return Some(placed.remove(index).item);
            }
            placed
                .iter_mut()
                .find_map(|placed| search(&mut placed.item.contents, id))
        }
        if self.in_bin(id) {
            return self.bin.take();
        }
        if let Some(index) = self.worn.iter().position(|item| item.id == id) {
            return Some(self.worn.remove(index));
        }
        search(&mut self.pockets, id).or_else(|| {
            self.worn
                .iter_mut()
                .find_map(|item| search(&mut item.contents, id))
        })
    }

    fn grid_mut(&mut self, holder: Holder) -> Option<&mut Vec<Placed>> {
        fn search(placed: &mut [Placed], id: Id) -> Option<&mut Vec<Placed>> {
            placed.iter_mut().find_map(|placed| {
                if placed.item.id == id {
                    Some(&mut placed.item.contents)
                } else {
                    search(&mut placed.item.contents, id)
                }
            })
        }
        match holder {
            Holder::Pockets => Some(&mut self.pockets),
            Holder::Inside(id) => search(&mut self.pockets, id).or_else(|| {
                self.worn.iter_mut().find_map(|item| {
                    if item.id == id {
                        Some(&mut item.contents)
                    } else {
                        search(&mut item.contents, id)
                    }
                })
            }),
        }
    }

    /// The first place in this grid where an item of this shape has room,
    /// starting with how it is turned now: rows from the top, then the next
    /// quarter turn. `moving` is lifted out, if it is carried here already.
    fn first_room(
        &self,
        holder: Holder,
        shape: Shape,
        turned_from: Rotation,
        moving: Option<Id>,
    ) -> Option<(u8, u8, Rotation)> {
        let (width, height) = self.grid_size(holder)?;
        let mut rotation = turned_from;
        for _ in Rotation::ALL {
            let turned = shape.turned(rotation);
            for y in 0..height {
                for x in 0..width {
                    if self.has_room(holder, turned, x, y, moving) {
                        return Some((x, y, rotation));
                    }
                }
            }
            rotation = rotation.next();
        }
        None
    }

    /// Where this carried item would go if dropped on this grid without saying
    /// where (on a closed bag): the first place it can.
    pub fn first_place_for(
        &self,
        id: Id,
        holder: Holder,
        turned_from: Rotation,
    ) -> Option<(u8, u8, Rotation)> {
        let item = self.find(id)?;
        self.may_hold(holder, item).ok()?;
        self.first_room(holder, item.kind.def().shape, turned_from, Some(id))
    }

    /// Puts a new item in the first place it fits in the pockets. Gives it back
    /// if there is no room.
    pub fn give(&mut self, item: Item) -> Result<(), Item> {
        let shape = item.kind.def().shape;
        match self.first_room(Holder::Pockets, shape, Rotation::None, None) {
            Some((x, y, rotation)) => {
                self.pockets.push(Placed {
                    item,
                    x,
                    y,
                    rotation,
                });
                Ok(())
            }
            None => Err(item),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Seed;

    #[test]
    fn the_odds_add_up_and_better_is_rarer_past_common() {
        let total: u64 = Quality::ALL.iter().map(|quality| quality.odds()).sum();
        assert_eq!(total, ODDS_OUT_OF);
        for pair in Quality::ALL[1..].windows(2) {
            assert!(pair[0].odds() > pair[1].odds(), "{pair:?}");
        }
    }

    #[test]
    fn rolls_of_quality_follow_the_odds() {
        let mut rng = Seed::new(12).rng();
        let mut counts = [0u64; 7];
        for _ in 0..100_000 {
            counts[Quality::roll(&mut rng) as usize] += 1;
        }
        for quality in Quality::ALL {
            let expected = quality.odds() * 10;
            let got = counts[quality as usize];
            assert!(
                got.abs_diff(expected) <= expected / 4 + 10,
                "{quality:?}: {got}"
            );
        }
    }

    #[test]
    fn the_same_seed_rolls_the_same_quality() {
        let rolls = |seed| {
            let mut rng = Seed::new(seed).rng();
            [(); 16].map(|_| Quality::roll(&mut rng))
        };
        assert_eq!(rolls(1), rolls(1));
        assert_ne!(rolls(1), rolls(2));
    }

    #[test]
    fn a_drop_takes_the_nearest_quality_the_table_has() {
        let line = |quality, kind| Drop {
            quality,
            kind,
            weight: 1,
        };
        assert_eq!(roll_drop(&mut Seed::new(1).rng(), &[]), None);
        // Only Rare in the table, whatever is rolled.
        let table = [line(Quality::Rare, ItemKind::Sword)];
        for seed in 0..200 {
            let dropped = roll_drop(&mut Seed::new(seed).rng(), &table);
            assert_eq!(dropped, Some((ItemKind::Sword, Some(Quality::Rare))));
        }
        // Each roll stays within what is at or below it, once any is.
        let table = [
            line(Quality::Poor, ItemKind::Hat),
            line(Quality::Epic, ItemKind::Sword),
        ];
        let mut swords = 0;
        for seed in 0..2000 {
            match roll_drop(&mut Seed::new(seed).rng(), &table) {
                Some((ItemKind::Hat, Some(Quality::Poor))) => {}
                Some((ItemKind::Sword, Some(Quality::Epic))) => swords += 1,
                other => panic!("{other:?}"),
            }
        }
        assert!((1..100).contains(&swords), "{swords}");
    }

    #[test]
    fn a_kind_without_quality_drops_without_one() {
        let table = [Drop {
            quality: Quality::Common,
            kind: ItemKind::Coin,
            weight: 1,
        }];
        let dropped = roll_drop(&mut Seed::new(3).rng(), &table);
        assert_eq!(dropped, Some((ItemKind::Coin, None)));
        assert!(ItemKind::Sword.def().has_quality);
    }

    /// Inventory with these kinds in the pockets, ids from 1, left to right.
    fn carrying(kinds: &[ItemKind]) -> Inventory {
        let mut inventory = Inventory::default();
        for (index, &kind) in kinds.iter().enumerate() {
            let item = Item::new(id(index as u128 + 1), kind);
            inventory.give(item).expect("room in the pockets");
        }
        inventory
    }

    fn id(n: u128) -> Id {
        Id::for_test(n)
    }

    const NONE: Rotation = Rotation::None;

    #[test]
    fn every_kind_has_a_sensible_shape_and_is_listed_alphabetically() {
        for kind in ItemKind::ALL {
            let def = kind.def();
            assert!(def.shape.size() > 0, "{}", def.name);
            assert!(def.shape.width <= MAX_SHAPE_SIDE && def.shape.height <= MAX_SHAPE_SIDE);
            assert!(def.weight > 0);
        }
        let names: Vec<_> = ItemKind::ALL.iter().map(|kind| kind.def().name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn a_quarter_turn_swaps_the_sides_and_four_come_back() {
        let shape = ItemKind::Boomerang.def().shape;
        // An L: everything but the top right.
        assert!(shape.has(0, 0) && !shape.has(1, 0) && shape.has(0, 1) && shape.has(1, 1));
        let turned = shape.turned(Rotation::Quarter);
        // Clockwise, the missing corner goes to the bottom right.
        assert!(turned.has(0, 0) && turned.has(1, 0) && turned.has(0, 1) && !turned.has(1, 1));

        let sword = ItemKind::Sword.def().shape.turned(Rotation::Quarter);
        assert_eq!((sword.width, sword.height), (5, 1));
        let mut all_round = ItemKind::Boomerang.def().shape;
        for _ in 0..4 {
            all_round = all_round.turned(Rotation::Quarter);
        }
        assert_eq!(all_round, ItemKind::Boomerang.def().shape);
    }

    #[test]
    fn things_cant_lie_on_each_other_or_stick_out() {
        let mut inventory = carrying(&[ItemKind::Stone, ItemKind::Apple]);
        assert_eq!(
            inventory.check_move(id(2), Holder::Pockets, 1, 1, NONE),
            Err(MoveRefused::NoRoom)
        );
        assert_eq!(
            inventory.check_move(id(1), Holder::Pockets, 5, 0, NONE),
            Err(MoveRefused::NoRoom)
        );
        // An item isn't in its own way.
        assert_eq!(
            inventory.check_move(id(1), Holder::Pockets, 1, 0, NONE),
            Err(MoveRefused::NoRoom),
            "the apple is at (2, 0)"
        );
        assert_eq!(
            inventory.move_item(id(2), Holder::Pockets, 5, 3, NONE),
            Ok(())
        );
        assert_eq!(
            inventory.move_item(id(1), Holder::Pockets, 1, 0, NONE),
            Ok(())
        );
        assert_eq!(inventory.placed(id(1)).unwrap().1.x, 1);
    }

    #[test]
    fn a_shape_with_a_gap_leaves_it_free() {
        let inventory = carrying(&[ItemKind::Boomerang, ItemKind::Coin]);
        // In the boomerang's missing corner.
        assert_eq!(inventory.placed(id(2)).unwrap().1.x, 1);
    }

    #[test]
    fn only_a_smaller_shape_goes_in() {
        let inventory = carrying(&[ItemKind::Pouch, ItemKind::Stone]);
        // A 2 × 2 stone in a 2 × 2 pouch: the same size isn't smaller.
        assert_eq!(
            inventory.check_move(id(2), Holder::Inside(id(1)), 0, 0, NONE),
            Err(MoveRefused::TooBig)
        );
        // The outer shape matters, not the bigger grid inside.
        let sword = Item::new(id(9), ItemKind::Sword);
        let mut with_sword = carrying(&[ItemKind::Pouch]);
        with_sword.give(sword).unwrap();
        assert_eq!(
            with_sword.check_move(id(9), Holder::Inside(id(1)), 0, 0, NONE),
            Err(MoveRefused::TooBig)
        );
    }

    #[test]
    fn a_pouch_holds_more_than_its_outside() {
        let mut inventory = carrying(&[ItemKind::Pouch]);
        for ring in 0..16u8 {
            let ring_id = id(100 + ring as u128);
            inventory
                .give(Item::new(ring_id, ItemKind::Ring))
                .unwrap_or_else(|_| panic!("the pockets are full"));
            inventory
                .move_item(ring_id, Holder::Inside(id(1)), ring % 4, ring / 4, NONE)
                .unwrap();
        }
        assert_eq!(inventory.find(id(1)).unwrap().contents.len(), 16);
        assert_eq!(inventory.pockets.len(), 1);
        assert_eq!(inventory.total_weight(), 80 + 16 * 10);
    }

    #[test]
    fn the_creature_counts_with_what_it_carries_and_wears_but_not_the_bin() {
        let mut inventory = carrying(&[ItemKind::Pouch, ItemKind::Ring, ItemKind::Ring]);
        inventory.creature = CreatureKind::Horse;
        assert_eq!(inventory.weight_with_creature(), 500_000 + 80 + 10 + 10);
        inventory
            .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
            .unwrap();
        assert!(inventory.to_bin(id(3)));
        inventory.worn.push(Item::new(id(4), ItemKind::Stone));
        let stone = ItemKind::Stone.def().weight;
        assert_eq!(inventory.weight_with_creature(), 500_000 + 80 + 10 + stone);
    }

    #[test]
    fn nothing_goes_inside_itself() {
        let mut inventory = carrying(&[ItemKind::Backpack, ItemKind::Pouch]);
        inventory
            .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
            .unwrap();
        assert_eq!(
            inventory.check_move(id(1), Holder::Inside(id(1)), 0, 0, NONE),
            Err(MoveRefused::IntoItself)
        );
        assert_eq!(
            inventory.check_move(id(1), Holder::Inside(id(2)), 0, 0, NONE),
            Err(MoveRefused::IntoItself)
        );
    }

    /// A backpack holding a pouch holding a box, in the pockets, ids 1 to 3.
    fn nested() -> Inventory {
        let mut inventory = carrying(&[ItemKind::Backpack, ItemKind::Pouch, ItemKind::SmallBox]);
        inventory
            .move_item(id(3), Holder::Inside(id(2)), 0, 0, NONE)
            .unwrap();
        inventory
            .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
            .unwrap();
        inventory
    }

    #[test]
    fn three_grids_deep_is_the_limit() {
        // The backpack, pouch and box are three grids, and a ring may go in the last.
        let mut inventory = nested();
        inventory.give(Item::new(id(4), ItemKind::Ring)).unwrap();
        assert_eq!(
            inventory.check_move(id(4), Holder::Inside(id(3)), 0, 0, NONE),
            Ok(())
        );

        // Nothing small enough has a grid today, so a box holding a pouch is made
        // by hand: going into the pouch, its grids would be third and fourth.
        inventory
            .give(Item::new(id(5), ItemKind::SmallBox))
            .unwrap();
        let (_, placed) = inventory.placed(id(5)).unwrap();
        let (x, y) = (placed.x, placed.y);
        let index = inventory
            .pockets
            .iter()
            .position(|placed| placed.x == x && placed.y == y);
        inventory.pockets[index.unwrap()]
            .item
            .contents
            .push(Placed {
                item: Item::new(id(6), ItemKind::Pouch),
                x: 0,
                y: 0,
                rotation: NONE,
            });
        assert_eq!(
            inventory.check_move(id(5), Holder::Inside(id(2)), 0, 1, NONE),
            Err(MoveRefused::TooDeep)
        );
        // One grid further out, it fits.
        assert_eq!(
            inventory.check_move(id(5), Holder::Inside(id(1)), 5, 0, NONE),
            Ok(())
        );
    }

    #[test]
    fn a_container_limits_the_weight_of_each_thing_and_of_all() {
        // A lantern is 800 g, and a pouch takes 500 g each.
        let inventory = carrying(&[ItemKind::Pouch, ItemKind::Lantern]);
        assert_eq!(
            inventory.check_move(id(2), Holder::Inside(id(1)), 0, 0, NONE),
            Err(MoveRefused::TooHeavy)
        );
        // Ten apples of 150 g are the 1500 g it takes in all.
        let mut inventory = carrying(&[ItemKind::Pouch]);
        for apple in 0..11u8 {
            let apple_id = id(50 + apple as u128);
            inventory
                .give(Item::new(apple_id, ItemKind::Apple))
                .unwrap();
            let result =
                inventory.move_item(apple_id, Holder::Inside(id(1)), apple % 4, apple / 4, NONE);
            let wanted = if apple < 10 {
                Ok(())
            } else {
                Err(MoveRefused::TooHeavy)
            };
            assert_eq!(result, wanted);
        }
    }

    #[test]
    fn weight_limits_look_at_the_containers_further_out() {
        // A box in a pouch, with nine apples beside it: 1450 g of the pouch's
        // 1500 g. Another apple is fine for the box, but not for the pouch.
        let mut inventory = carrying(&[ItemKind::Pouch, ItemKind::SmallBox]);
        inventory
            .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
            .unwrap();
        for apple in 0..9u8 {
            let apple_id = id(50 + apple as u128);
            inventory
                .give(Item::new(apple_id, ItemKind::Apple))
                .unwrap();
            inventory
                .move_item(
                    apple_id,
                    Holder::Inside(id(1)),
                    apple % 4,
                    1 + apple / 4,
                    NONE,
                )
                .unwrap();
        }
        let last = id(70);
        inventory.give(Item::new(last, ItemKind::Apple)).unwrap();
        assert_eq!(
            inventory.check_move(last, Holder::Inside(id(2)), 0, 0, NONE),
            Err(MoveRefused::TooHeavy)
        );

        // A pouch made by hand inside a pouch can't grow past 500 g in the outer.
        let mut inventory = carrying(&[ItemKind::Pouch, ItemKind::Apple]);
        let mut inner = Item::new(id(3), ItemKind::Pouch);
        for apple in 0..3 {
            inner.contents.push(Placed {
                item: Item::new(id(10 + apple as u128), ItemKind::Apple),
                x: apple,
                y: 0,
                rotation: NONE,
            });
        }
        inventory.pockets[0].item.contents.push(Placed {
            item: inner,
            x: 0,
            y: 0,
            rotation: NONE,
        });
        assert_eq!(
            inventory.check_move(id(2), Holder::Inside(id(3)), 3, 0, NONE),
            Err(MoveRefused::TooHeavy)
        );
    }

    #[test]
    fn moving_about_inside_a_bag_doesnt_count_its_weight_twice() {
        let mut inventory = carrying(&[ItemKind::Pouch]);
        for apple in 0..10u8 {
            let apple_id = id(50 + apple as u128);
            inventory
                .give(Item::new(apple_id, ItemKind::Apple))
                .unwrap();
            inventory
                .move_item(apple_id, Holder::Inside(id(1)), apple % 4, apple / 4, NONE)
                .unwrap();
        }
        // It holds all it can, and an apple can still move to a free place.
        assert_eq!(
            inventory.move_item(id(50), Holder::Inside(id(1)), 3, 3, NONE),
            Ok(())
        );
    }

    #[test]
    fn dropped_on_a_closed_bag_it_takes_the_first_place_turning_if_it_must() {
        let inventory = carrying(&[ItemKind::Backpack, ItemKind::Lantern]);
        let place = inventory.first_place_for(id(2), Holder::Inside(id(1)), NONE);
        assert_eq!(place, Some((0, 0, NONE)));

        // A box 2 × 1 in a pouch filled but for the last column.
        let mut pouch = carrying(&[ItemKind::Pouch]);
        for y in 0..4u8 {
            for x in 0..3u8 {
                let coin = id(100 + (y as u128) * 10 + x as u128);
                pouch.give(Item::new(coin, ItemKind::Coin)).unwrap();
                pouch
                    .move_item(coin, Holder::Inside(id(1)), x, y, NONE)
                    .unwrap();
            }
        }
        pouch.give(Item::new(id(2), ItemKind::SmallBox)).unwrap();
        let place = pouch.first_place_for(id(2), Holder::Inside(id(1)), NONE);
        assert_eq!(place, Some((3, 0, Rotation::Quarter)));

        // Not a place at all when it isn't allowed in.
        assert_eq!(
            inventory.first_place_for(id(1), Holder::Inside(id(1)), NONE),
            None
        );
    }

    #[test]
    fn the_bin_keeps_the_last_thing_thrown_away_until_the_next() {
        let mut inventory = nested();
        inventory.give(Item::new(id(4), ItemKind::Coin)).unwrap();
        assert!(inventory.to_bin(id(2)));
        // Gone from the backpack, but it and what is in it can still be had.
        assert!(inventory.find(id(1)).unwrap().contents.is_empty());
        assert!(inventory.find(id(2)).is_some());
        assert!(inventory.placed(id(2)).is_none());
        // It weighs nothing there.
        assert_eq!(inventory.total_weight(), 1200 + 10);
        // Thrown away twice, nothing changes.
        assert!(!inventory.to_bin(id(2)));
        // Taken back, it goes in a grid again.
        assert_eq!(
            inventory.move_item(id(2), Holder::Pockets, 4, 0, NONE),
            Ok(())
        );
        assert!(inventory.bin.is_none());
        // Another thrown away destroys the first.
        assert!(inventory.to_bin(id(2)));
        assert!(inventory.to_bin(id(4)));
        assert!(inventory.find(id(2)).is_none());
        assert!(inventory.find(id(3)).is_none());
        assert!(inventory.find(id(4)).is_some());
    }

    #[test]
    fn a_new_item_goes_in_the_first_place_or_comes_back() {
        let mut inventory = Inventory::default();
        for n in 0..24 {
            inventory.give(Item::new(id(n), ItemKind::Coin)).unwrap();
        }
        let refused = inventory.give(Item::new(id(99), ItemKind::Coin));
        assert_eq!(refused.unwrap_err().id, id(99));
    }

    #[test]
    fn an_inventory_reads_back_from_a_message() {
        let inventory = nested();
        let bytes = postcard::to_allocvec(&inventory).unwrap();
        assert_eq!(
            postcard::from_bytes::<Inventory>(&bytes).unwrap(),
            inventory
        );
    }
}
