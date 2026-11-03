//! `Item`: one item, with its id, kind and what is inside it.

use serde::{Deserialize, Serialize};

use crate::creature::{Location, Size};
use crate::id::Id;
use crate::item::{Favour, ItemKind, Known, Made, Placed, Quality, Shape};
use crate::item_form::ItemForm;
use crate::world::WorldId;

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
    pub(super) fn grids_deep(&self) -> u32 {
        if self.kind.def().container.is_none() {
            return 0;
        }
        let inside = self.contents.iter().map(|placed| placed.item.grids_deep());
        1 + inside.max().unwrap_or(0)
    }

    /// Whether `id` is this item or somewhere inside it.
    pub(super) fn holds_or_is(&self, id: Id) -> bool {
        self.id == id
            || self
                .contents
                .iter()
                .any(|placed| placed.item.holds_or_is(id))
            || self.set_gems().any(|gem| gem.holds_or_is(id))
    }
}

pub(super) fn weight_of(placed: &[Placed]) -> u32 {
    placed.iter().map(|placed| placed.item.total_weight()).sum()
}
