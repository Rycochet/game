//! What can fill a Dev Area: the plain layouts, a biome's wild land, or a
//! kind of poi's site, and their names for lists and for typing.

use serde::{Deserialize, Serialize};

use crate::climate::Biome;
use crate::poi::PoiKind;

/// What fills an area inside its walls.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum DevLayout {
    /// Open ground.
    #[default]
    Empty,
    /// Deep water up the middle, two cells wide, with a gap at each end: it
    /// can be seen across but not walked.
    Moat,
    /// The same of wall: nothing can be seen through it.
    Cover,
    /// The wilderness's ground, grass and trees as they grow in this biome.
    Biome(Biome),
    /// The ground floor of this kind of poi's site, as its generator makes it
    /// (no stairs), in the middle; cut off where it is bigger than the area.
    Building(PoiKind),
}

impl DevLayout {
    pub const PLAIN: [Self; 3] = [Self::Empty, Self::Moat, Self::Cover];

    /// Every layout, in the order lists show them: the plain ones, the biomes,
    /// then every kind of poi.
    pub fn all() -> Vec<Self> {
        let biomes = Biome::ALL.into_iter().map(Self::Biome);
        let kinds = PoiKind::ALL.into_iter().map(Self::Building);
        Self::PLAIN.into_iter().chain(biomes).chain(kinds).collect()
    }

    /// Its name in lists: "Empty", "Boreal Forest", "Stone Tower".
    pub fn name(self) -> String {
        match self {
            Self::Empty => "Empty".into(),
            Self::Moat => "Moat".into(),
            Self::Cover => "Cover".into(),
            Self::Biome(biome) => words(&format!("{biome:?}")),
            Self::Building(kind) => words(&format!("{kind:?}")),
        }
    }

    /// The one named so, in any case, with or without spaces.
    pub fn named(name: &str) -> Option<Self> {
        let squashed = |text: &str| text.replace([' ', '_', '-'], "").to_lowercase();
        let wanted = squashed(name);
        Self::all()
            .into_iter()
            .find(|layout| squashed(&layout.name()) == wanted)
    }

    /// One number for each layout, for `DevArea::key`.
    pub(super) fn number(self) -> i64 {
        match self {
            Self::Empty => 0,
            Self::Moat => 1,
            Self::Cover => 2,
            Self::Biome(biome) => 100 + biome as i64,
            Self::Building(kind) => 1000 + kind as i64,
        }
    }
}

/// A name in CamelCase as words: "BorealForest" to "Boreal Forest".
fn words(camel: &str) -> String {
    let mut name = String::new();
    for letter in camel.chars() {
        if letter.is_uppercase() && !name.is_empty() {
            name.push(' ');
        }
        name.push(letter);
    }
    name
}
