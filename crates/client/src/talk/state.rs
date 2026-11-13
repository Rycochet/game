//! The talk panels open, who keeps each shop, and whose door the player is
//! walking to.

use bevy::prelude::*;
use shared::id::Id;
use shared::poi::ChunkPos;
use shared::protocol::Offer;
use shared::quest::QuestId;
use shared::shop::beside_door;
use shared::site::BuildingLook;
use shared::spot::Spot;
use shared::story::Ending;
use std::collections::BTreeMap;

use crate::common::MapPosition;
use crate::widgets::StockGrid;

/// The panel's two tabs, in the tab bar's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tab {
    Quests,
    Trade,
}

/// A talk panel: one building's door on this level, whoever is behind it.
pub(super) struct OpenTalk {
    pub(super) building: u16,
    /// Its document, which its panel goes with.
    pub(super) panel: Entity,
    /// The door: what the player must stay near.
    pub(super) place: (i64, i64),
    pub(super) tab: Tab,
    /// The shop's stock, once the server has sent it (shops only), and the
    /// panel's `object-grid` it goes in, once built.
    pub(super) grid: Option<StockGrid>,
    pub(super) holder: Option<Entity>,
    /// What the giver has, as last told (`QuestNews::Talk`), and the town.
    pub(super) town: Option<ChunkPos>,
    pub(super) offers: Vec<Offer>,
    /// Whether the giver has answered since the panel opened.
    pub(super) heard: bool,
    /// The quest shown in full; none for the list.
    pub(super) chosen: Option<QuestId>,
    /// A quest just handed in: what the giver says now.
    pub(super) ended: Option<(QuestId, Ending)>,
    /// What the giver says to something that couldn't be done.
    pub(super) refused: Option<String>,
    /// The quests page wants writing out again.
    pub(super) dirty: bool,
}

impl OpenTalk {
    pub(super) fn new(building: u16, panel: Entity, place: (i64, i64), tab: Tab) -> Self {
        Self {
            building,
            panel,
            place,
            tab,
            grid: None,
            holder: None,
            town: None,
            offers: Vec::new(),
            heard: false,
            chosen: None,
            ended: None,
            refused: None,
            dirty: true,
        }
    }

    /// What its document is known by: `talk#<key>/...`.
    pub(super) fn key(&self) -> Id {
        key_of(self.building)
    }

    /// The shop's keeper, once its stock has come.
    pub(super) fn keeper(&self) -> Option<Id> {
        self.grid.as_ref().map(|grid| grid.with)
    }
}

/// What a building's panel is known by in its ids. Not anyone's real id: a
/// stand-in, numbered by the building (only one level's doors are open at
/// once).
pub(super) fn key_of(building: u16) -> Id {
    Id::stand_in(u64::from(building))
}

/// The panels open, in the order they were opened.
#[derive(Resource, Default)]
pub(super) struct OpenTalks(pub(super) Vec<OpenTalk>);

impl OpenTalks {
    pub(super) fn of(&mut self, building: u16) -> Option<&mut OpenTalk> {
        self.0.iter_mut().find(|talk| talk.building == building)
    }

    pub(super) fn by_key(&mut self, key: Id) -> Option<&mut OpenTalk> {
        self.0.iter_mut().find(|talk| talk.key() == key)
    }

    pub(super) fn trading_with(&mut self, keeper: Id) -> Option<&mut OpenTalk> {
        self.0.iter_mut().find(|talk| talk.keeper() == Some(keeper))
    }

    pub(super) fn is_open(&self, building: u16) -> bool {
        self.0.iter().any(|talk| talk.building == building)
    }
}

/// Who keeps each shop on this level, by building.
#[derive(Resource, Default)]
pub(super) struct Keepers(pub(super) BTreeMap<u16, Id>);

/// The building whose door the player is walking to, to talk on arrival, and
/// the cell in front of it.
#[derive(Resource, Default)]
pub(super) struct Heading(pub(super) Option<(u16, (i64, i64))>);

/// Whether someone here is near enough to this door. The server checks this too.
pub(super) fn beside(from: MapPosition, door: (i64, i64)) -> bool {
    beside_door(Spot::middle_of((from.x, from.y)), door)
}

pub(super) fn front_of(building: &BuildingLook) -> (i64, i64) {
    (
        building.door.0 + building.faces.0,
        building.door.1 + building.faces.1,
    )
}
