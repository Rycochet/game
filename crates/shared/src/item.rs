//! Items, and carrying them in grids: the rules both sides check moves by. See
//! `docs/context/shared/item.md`.
//!
//! What a kind of item is (`ItemKind::def`) is a table here. An item itself is
//! its id and kind, and what is inside it. Everything a character carries is a
//! tree starting from their pockets: a grid of placed items, some of which have
//! grids of their own.

use serde::{Deserialize, Serialize};

use crate::blade::Family;
use crate::creature::{CreatureKind, Layer, Location, Size};
use crate::energy::{dressing_cost, handling_cost};
use crate::id::Id;
use crate::item_form::ItemForm;
use crate::rng::Rng;
use crate::world::WorldId;

mod def;
mod drop;
mod shape;

pub use drop::{Drop, ODDS_OUT_OF, roll_drop};
pub use shape::{Rotation, Shape};

/// The widest and tallest a shape can be.
pub const MAX_SHAPE_SIDE: u8 = 8;

/// How many grids deep things can go below the pockets: backpack, pouch, box.
pub const MAX_DEPTH: u32 = 3;

/// The pockets of a character saved before creature kinds, and of a new
/// `Inventory`. Otherwise they come from the kind of creature.
pub const POCKETS: (u8, u8) = (6, 4);

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
    /// How long it takes to make beside other things, as a percentage.
    pub make_percent: u64,
}

/// How something is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WearDef {
    pub layer: Layer,
    pub locations: &'static [Location],
    /// It takes only one place of its one location: a ring on a finger.
    pub one_place: bool,
    /// Of its locations, where it also takes the Middle layer: armour with its own
    /// padding, worn all as one.
    pub padded: &'static [Location],
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
    ArmArmour,
    ArmingCap,
    ArmingVest,
    Axe,
    Backpack,
    Barding,
    Belt,
    BevoredHelm,
    Boomerang,
    Boots,
    Caparison,
    Cloak,
    Coif,
    Coin,
    Dagger,
    DemiGauntlets,
    FingerlessGloves,
    FullArmour,
    Gambeson,
    Gauntlets,
    Gem,
    Gloves,
    Hat,
    Helmet,
    Knife,
    Lantern,
    Longsword,
    Necklace,
    PaddedGloves,
    Polearm,
    Pouch,
    Ring,
    Shoes,
    ShortSword,
    ShoulderArmour,
    SmallBox,
    Socks,
    Spear,
    Stone,
    Surcoat,
    Sword,
    TShirt,
    Tabard,
    TorsoArmour,
    Trousers,
}

impl ItemKind {
    /// Every kind, alphabetically by name.
    pub const ALL: [Self; 46] = [
        Self::Apple,
        Self::ArmArmour,
        Self::ArmingCap,
        Self::ArmingVest,
        Self::Axe,
        Self::Backpack,
        Self::Barding,
        Self::Belt,
        Self::BevoredHelm,
        Self::Boomerang,
        Self::Boots,
        Self::Caparison,
        Self::Cloak,
        Self::Coif,
        Self::Coin,
        Self::Dagger,
        Self::DemiGauntlets,
        Self::FingerlessGloves,
        Self::FullArmour,
        Self::Gambeson,
        Self::Gauntlets,
        Self::Gem,
        Self::Gloves,
        Self::Hat,
        Self::Helmet,
        Self::Knife,
        Self::Lantern,
        Self::Longsword,
        Self::Necklace,
        Self::PaddedGloves,
        Self::Polearm,
        Self::Pouch,
        Self::Ring,
        Self::Shoes,
        Self::ShortSword,
        Self::ShoulderArmour,
        Self::SmallBox,
        Self::Socks,
        Self::Spear,
        Self::Stone,
        Self::Surcoat,
        Self::Sword,
        Self::TShirt,
        Self::Tabard,
        Self::TorsoArmour,
        Self::Trousers,
    ];

    /// Whether it can be made at other sizes: what is worn, gems and blades. Everything
    /// else is always `Medium`, whoever carries it.
    pub const fn comes_in_sizes(self) -> bool {
        self.def().wear.is_some() || matches!(self, Self::Gem) || self.blade_family().is_some()
    }

    /// The family of weapon this makes, for those rolled as blades.
    pub const fn blade_family(self) -> Option<Family> {
        match self {
            Self::Knife => Some(Family::Knife),
            Self::Dagger => Some(Family::Dagger),
            Self::ShortSword => Some(Family::ShortSword),
            Self::Sword => Some(Family::Sword),
            Self::Longsword => Some(Family::LongSword),
            Self::Axe => Some(Family::Axe),
            Self::Spear => Some(Family::Spear),
            Self::Polearm => Some(Family::Polearm),
            _ => None,
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
        make_percent: 100,
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
        padded: &[],
        down,
        up,
        adjustable,
    }
}

impl ItemDef {
    const fn taking(self, make_percent: u64) -> Self {
        Self {
            make_percent,
            ..self
        }
    }

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
    /// It resizes itself to fit whatever their fingers, and sizes as far from its own
    /// as its quality reaches (`Quality::magic_reach`), but not whatever their shape.
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

/// How good an item is, worst first. The better, the rarer. Being part of a set is
/// apart from this: see `Item::set` and `decisions.md`.
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

    /// How many sizes from its own something magical of this quality fits perfectly.
    /// Nothing below uncommon can be magical.
    pub const fn magic_reach(self) -> u8 {
        match self {
            Self::Poor | Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::Epic => 3,
            Self::Legendary => 4,
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

/// A placeholder name for an artifact of this kind: artifacts are always named.
pub fn artifact_name(rng: &mut Rng, kind: ItemKind) -> String {
    const WORDS: [&str; 12] = [
        "Ashen",
        "Dawning",
        "Endless",
        "Forgotten",
        "Gilded",
        "Hollow",
        "Last",
        "Quiet",
        "Sunken",
        "Thousandth",
        "Wandering",
        "Whispering",
    ];
    let word = WORDS[rng.range_u64(0, WORDS.len() as u64) as usize];
    format!("The {word} {}", kind.def().name)
}

/// How likely something that can be magical is to be, in percent. First guess.
const MAGICAL_PERCENT: u64 = 5;
/// How likely something wearable is to be blessed, and to be cursed, in percent.
/// First guesses.
const BLESSED_PERCENT: u64 = 5;
const CURSED_PERCENT: u64 = 5;

/// What can lie on an item besides magic: never both, so there is one field for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Favour {
    /// Only its wearer can take it off.
    Blessed,
    /// Nobody can take it off, once it is on.
    Cursed,
}

/// What has been found out about an item, for everyone from then on. What hasn't
/// isn't sent to clients: see `Item::hide_secrets`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Known {
    pub magic: bool,
    pub favour: bool,
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
    /// Its own name, if it has one: every artifact does.
    #[serde(default)]
    pub name: Option<String>,
    /// Part of a set, of whatever quality it is.
    #[serde(default)]
    pub set: bool,
    /// What it is physically like, for kinds rolled that way, and not for things
    /// saved before this was kept.
    #[serde(default)]
    pub form: Option<ItemForm>,
    /// Blessed, cursed or neither.
    #[serde(default)]
    pub favour: Option<Favour>,
    /// Which of its secrets have been found out.
    #[serde(default)]
    pub known: Known,
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
            name: None,
            set: false,
            form: None,
            favour: None,
            known: Known::default(),
        }
    }

    /// Whether this could be magical: only what is worn, better than common.
    pub fn can_be_magical(&self) -> bool {
        self.kind.def().wear.is_some() && self.quality.is_some_and(|q| q.magic_reach() > 0)
    }

    /// Whether it is magical. What is flagged but can't be counts for nothing.
    pub fn is_magical(&self) -> bool {
        self.made.magical && self.can_be_magical()
    }

    /// Rolls whether a new thing to wear is magical, and blessed or cursed, going by
    /// its id. Only for what is found: not for what is asked for by name.
    pub fn roll_secrets(&mut self) {
        if self.kind.def().wear.is_none() {
            return;
        }
        let mut rng = self.id.seed().derive("item secrets").rng();
        self.made.magical = rng.range_u64(0, 100) < MAGICAL_PERCENT && self.can_be_magical();
        let favour = rng.range_u64(0, 100);
        self.favour = if favour < BLESSED_PERCENT {
            Some(Favour::Blessed)
        } else if favour < BLESSED_PERCENT + CURSED_PERCENT {
            Some(Favour::Cursed)
        } else {
            None
        };
    }

    /// As a client may see it, and everything in it: what hasn't been found out
    /// looks as if it wasn't there.
    pub fn hide_secrets(&mut self) {
        if !self.known.magic {
            self.made.magical = false;
        }
        if !self.known.favour {
            self.favour = None;
        }
        for placed in &mut self.contents {
            placed.item.hide_secrets();
        }
    }

    /// What it is called: its own name, or by its form or kind, saying its size
    /// unless that is `Medium`, and whether it is blessed, cursed or magical. For a
    /// client, whose copy says only what is known of those: not for the server to
    /// send, as its own copy has them all.
    pub fn name(&self) -> String {
        let favour = self.favour.map(|favour| match favour {
            Favour::Blessed => "Blessed",
            Favour::Cursed => "Cursed",
        });
        let magical = self.is_magical().then_some("Magical");
        let states: Vec<&str> = favour.into_iter().chain(magical).collect();
        if let Some(name) = &self.name {
            return match states.is_empty() {
                true => name.clone(),
                false => format!("{name} ({})", states.join(", ")),
            };
        }
        let plain = self.plain_name();
        match states.is_empty() {
            true => plain,
            false => format!("{} {plain}", states.join(" ")),
        }
    }

    /// What it is called by its form or kind, with its size.
    fn plain_name(&self) -> String {
        let plain = match &self.form {
            Some(form) => form.name(),
            None => self.kind.def().name.to_owned(),
        };
        match self.made.size {
            Size::Medium => plain,
            size => format!("{} {plain}", size.name()),
        }
    }

    /// The cells it takes up, by its kind and the size it was made.
    pub fn shape(&self) -> Shape {
        let shape = self.form.as_ref().and_then(ItemForm::shape);
        let shape = shape.unwrap_or(self.kind.def().shape);
        shape.scaled(self.made.size)
    }

    /// What it weighs with everything in it, in grams.
    pub fn total_weight(&self) -> u32 {
        let set: u32 = self.set_gems().map(Item::total_weight).sum();
        let own = self
            .form
            .as_ref()
            .and_then(|form| form.grams_at(self.made.size));
        let own = own.unwrap_or(self.kind.def().weight);
        own + weight_of(&self.contents) + set
    }

    /// Where it also takes the Middle layer: its kind's padding, or torso armour that
    /// is lined or padding itself.
    pub fn padded(&self) -> &'static [Location] {
        match &self.form {
            Some(ItemForm::Torso(torso)) if torso.padded() => &[Location::Torso],
            Some(ItemForm::Arms(arms)) if arms.padded() => &[Location::Arms],
            _ => self.kind.def().wear.map_or(&[], |wear| wear.padded),
        }
    }

    /// The gems set in it.
    pub fn set_gems(&self) -> impl Iterator<Item = &Item> {
        self.form.iter().flat_map(ItemForm::gems)
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
            || self.set_gems().any(|gem| gem.holds_or_is(id))
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
        self.item.shape().turned(self.rotation)
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
        let target_item = self.find(target).ok_or(MoveRefused::NoSuchGrid)?;
        if target_item.kind.def().container.is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        if !item.shape().smaller_than(target_item.shape()) {
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
        let shape = item.shape().turned(rotation);
        if self.grid(holder).is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        if self.has_room(holder, shape, x, y, Some(id)) {
            Ok(())
        } else {
            Err(MoveRefused::NoRoom)
        }
    }

    /// Whether something not carried could be put here.
    pub fn check_add(
        &self,
        item: &Item,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        self.may_hold(holder, item)?;
        if self.grid(holder).is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        let shape = item.shape().turned(rotation);
        if self.has_room(holder, shape, x, y, None) {
            Ok(())
        } else {
            Err(MoveRefused::NoRoom)
        }
    }

    /// Puts something not carried here, if it can go.
    pub fn add(
        &mut self,
        item: Item,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        self.check_add(&item, holder, x, y, rotation)?;
        self.grid_mut(holder).expect("checked above").push(Placed {
            item,
            x,
            y,
            rotation,
        });
        Ok(())
    }

    /// The first place something not carried fits in this container, turning if it
    /// must, as `first_place_for`.
    pub fn first_place_for_new(
        &self,
        item: &Item,
        holder: Holder,
        turned_from: Rotation,
    ) -> Option<(u8, u8, Rotation)> {
        self.may_hold(holder, item).ok()?;
        self.first_room(holder, item.shape(), turned_from, None)
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

    /// Throws away something not carried, destroying what was thrown away before.
    pub fn to_bin_new(&mut self, item: Item) {
        self.bin = Some(item);
    }

    /// What moving this carried item to this grid costs: nothing within a grid.
    pub fn move_cost(&self, id: Id, to: Holder) -> u32 {
        let Some(item) = self.find(id) else {
            return 0;
        };
        let grams = item.total_weight();
        if self.is_worn(id) {
            dressing_cost(grams)
        } else if self.placed(id).is_some_and(|(from, _)| from == to) {
            0
        } else {
            handling_cost(grams)
        }
    }

    /// What putting this carried item down costs, or handing it over: taking it off
    /// first, if it is worn.
    pub fn put_down_cost(&self, id: Id) -> u32 {
        let Some(item) = self.find(id) else {
            return 0;
        };
        let grams = item.total_weight();
        let taking_off = if self.is_worn(id) {
            dressing_cost(grams)
        } else {
            0
        };
        taking_off + handling_cost(grams)
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
        self.first_room(holder, item.shape(), turned_from, Some(id))
    }

    /// Puts a new item in the first place it fits in the pockets. Gives it back
    /// if there is no room.
    #[allow(clippy::result_large_err)] // Handing it back whole is the point, and rare.
    pub fn give(&mut self, item: Item) -> Result<(), Item> {
        let shape = item.shape();
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
mod add_tests;

#[cfg(test)]
mod tests;
