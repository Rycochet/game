//! Places with no name of their own, described: what kind of place, and
//! whether it lies in ruins. Values each language words.

use serde::{Deserialize, Serialize};

use crate::lang::{Args, Lang, RenderError, Selector};
use crate::poi::PoiKind;

/// What kind of place a story speaks of, from its poi's kind (or the wilds,
/// for land with no poi).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PlaceKind {
    Cave,
    Crypt,
    Grave,
    Ruin,
    Tower,
    Stronghold,
    Dwelling,
    Store,
    Camp,
    Pyramid,
    Settlement,
    Farm,
    HerbalistHut,
    Oasis,
    Wilds,
    // New kinds go at the end: messages carry one by its place.
}

/// Whether a place is still whole or lies in ruins.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    #[default]
    Whole,
    Ruined,
}

/// An unnamed place as a story describes it ("a ruined tower").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaceDescription {
    pub kind: PlaceKind,
    pub condition: Condition,
}

impl PlaceKind {
    pub const ALL: [Self; 15] = [
        Self::Cave,
        Self::Crypt,
        Self::Grave,
        Self::Ruin,
        Self::Tower,
        Self::Stronghold,
        Self::Dwelling,
        Self::Store,
        Self::Camp,
        Self::Pyramid,
        Self::Settlement,
        Self::Farm,
        Self::HerbalistHut,
        Self::Oasis,
        Self::Wilds,
    ];

    /// The kind of place a poi is. Every poi kind is listed, so a new one has to
    /// be decided on.
    pub const fn of_poi(kind: PoiKind) -> Self {
        use PoiKind::*;
        match kind {
            GrassyCave | EarthCave | RockCave | IceCave => Self::Cave,
            Crypt | MossyCrypt | BloodyCrypt | Mausoleum => Self::Crypt,
            Grave => Self::Grave,
            Ruins | Boulders => Self::Ruin,
            StoneTower | WoodenTower => Self::Tower,
            Castle | Fort | ArcheryRange => Self::Stronghold,
            ThatchedHut | AdobeHouse | StiltHut | GreenStiltHut | RedRoofCottage
            | GreyRoofCottage | StoneCottage | SnowyCottage | IceShelter => Self::Dwelling,
            AdobeStore | Warehouse | Barrel | Barrels => Self::Store,
            RedTent | BlueTent | GreenTent => Self::Camp,
            StonePyramid | SandPyramid => Self::Pyramid,
            Hamlet | Village | Town | City => Self::Settlement,
            Farm => Self::Farm,
            HerbalistHut => Self::HerbalistHut,
            Oasis => Self::Oasis,
        }
    }

    /// Whether it can be said to lie in ruins: a ruin already does, and the
    /// wilds and an oasis are never built.
    pub const fn can_be_ruined(self) -> bool {
        !matches!(self, Self::Ruin | Self::Wilds | Self::Oasis)
    }
}

impl PlaceDescription {
    /// A place of `kind`, ruined if `ruined` and the kind can be.
    pub fn new(kind: PlaceKind, ruined: bool) -> Self {
        let condition = if ruined && kind.can_be_ruined() {
            Condition::Ruined
        } else {
            Condition::Whole
        };
        Self { kind, condition }
    }

    /// Its description in `lang`, with no article ("ruined tower"): the story
    /// adds one by the language's rules.
    pub fn describe(&self, lang: &Lang) -> Result<String, RenderError> {
        let kind = lang.phrase(&self.kind)?;
        lang.render(
            "place-unnamed",
            &Args::new()
                .text("place-kind", kind)
                .choice("condition", &self.condition),
        )
    }
}

impl Selector for PlaceKind {
    fn fact(&self) -> &'static str {
        "place-kind"
    }

    fn key(&self) -> &'static str {
        match self {
            Self::Cave => "cave",
            Self::Crypt => "crypt",
            Self::Grave => "grave",
            Self::Ruin => "ruin",
            Self::Tower => "tower",
            Self::Stronghold => "stronghold",
            Self::Dwelling => "dwelling",
            Self::Store => "store",
            Self::Camp => "camp",
            Self::Pyramid => "pyramid",
            Self::Settlement => "settlement",
            Self::Farm => "farm",
            Self::HerbalistHut => "herbalist-hut",
            Self::Oasis => "oasis",
            Self::Wilds => "wilds",
        }
    }
}

impl Selector for Condition {
    fn fact(&self) -> &'static str {
        "condition"
    }

    fn key(&self) -> &'static str {
        match self {
            Self::Whole => "whole",
            Self::Ruined => "ruined",
        }
    }
}
