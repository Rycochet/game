//! What a connected `Client` can do: its position, speed, inventory, saving and leaving.

use bevy::prelude::*;
use shared::difficulty::Difficulty;
use shared::energy::{Energy, NORMAL_SPEED, dressing_cost};
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::protocol::{CharacterId, ServerLink, ServerMessage};
use shared::sheet::Sheet;
use shared::wear::Remover;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{Client, Phase, SAVES_TROUBLE};
use crate::ground::GroundTold;
use crate::sites::{self, Visit};
use crate::storage::{CharacterWorldSave, Everywhere};
use crate::{ServerWorld, world_map};

impl Client {
    pub(crate) fn new(link: ServerLink, character_id: Option<CharacterId>) -> Self {
        Self {
            link,
            // Playing a character without logging in: the embedded server's own.
            host: character_id.is_some(),
            asked_online: None,
            phase: Phase::Greeting,
            makes_characters: false,
            world: None,
            going_to: None,
            account: None,
            character_id,
            name: String::new(),
            gone: false,
            character: CharacterWorldSave::default(),
            other_worlds: BTreeMap::new(),
            inventory: Inventory::default(),
            detect_monsters: false,
            god_mode: false,
            truesight: false,
            seen_entities: BTreeMap::new(),
            seen_feelings: BTreeMap::new(),
            seen_worn: BTreeMap::new(),
            sheet: Sheet::default(),
            action_bar: Default::default(),
            gates: BTreeSet::new(),
            ground_told: GroundTold::default(),
            activity: super::playtime::Activity::default(),
            vitals_told: None,
            energy: Energy::default(),
            speed: NORMAL_SPEED,
            waiting: VecDeque::new(),
            fight: Default::default(),
            dev_gifts: Vec::new(),
            learning: Default::default(),
            starting_as: None,
            difficulty: None,
            asked_difficulties: None,
            unsaved: false,
            visit: None,
            world_chunks: BTreeSet::new(),
            sent_roads: BTreeSet::new(),
            reveal_pois: world_map::ALL_POIS_KNOWN,
            map: 0,
            hall_origin: 0,
            last_step: None,
            looking_up: Vec::new(),
        }
    }

    /// Whether it is being disconnected.
    pub(crate) fn is_gone(&self) -> bool {
        self.gone
    }

    /// Where the player is on the world map (at the poi, in a site).
    pub(crate) fn world_position(&self) -> (i64, i64) {
        self.character.world_position
    }

    /// The player is being put on another map (the message saying so is
    /// sent next): steps the client took on the last one no longer count.
    pub(crate) fn changed_map(&mut self) {
        self.map += 1;
        self.last_step = None;
        self.waiting.clear();
        // The client forgets them too.
        self.seen_entities.clear();
        self.seen_feelings.clear();
        self.seen_worn.clear();
        self.ground_told = GroundTold::default();
    }

    /// What it carries has changed, which can change how fast it goes.
    pub(crate) fn update_speed(&mut self, tick: u64) {
        self.energy.settle(tick, self.speed);
        self.speed = self.sheet.speed(&self.inventory);
    }

    /// Tells the client what it carries, as far as it knows.
    pub(crate) fn send_inventory(&mut self) -> bool {
        // Part of a stack lifted out for a move that didn't take it goes back.
        if self.inventory.settle_parts() {
            self.unsaved = true;
        }
        let mut seen = self.inventory.clone();
        if !self.truesight {
            seen.hide_secrets();
        }
        self.link.send(&ServerMessage::Inventory(seen))
    }

    /// These items, as far as the client knows.
    pub(crate) fn as_seen(&self, mut items: Vec<Item>) -> Vec<Item> {
        if !self.truesight {
            items.iter_mut().for_each(Item::hide_secrets);
        }
        items
    }

    /// Whether the player's own attempt to take this off gets anywhere. A curse
    /// holding it is found out by trying, unless something on top stopped the attempt.
    pub(crate) fn may_take_off(&mut self, item: shared::id::Id) -> bool {
        if self.god_mode
            || self.inventory.covered_by(item).is_some()
            || self.inventory.may_remove(item, Remover::Wearer)
        {
            return true;
        }
        self.inventory.favour_found_out(item);
        self.unsaved = true;
        false
    }

    /// It has put this on or taken it off, which takes longer the heavier it is.
    pub(super) fn spend_dressing(&mut self, item: shared::id::Id, tick: u64) {
        let grams = self.inventory.find(item).map_or(0, Item::total_weight);
        self.energy.spend(tick, self.speed, dressing_cost(grams));
    }

    /// If steps arrived this tick, tells the client where the player is
    /// after the last of them. `false` if the client has gone.
    pub(super) fn send_position(&mut self) -> bool {
        let Some(step) = self.last_step.take() else {
            return true;
        };
        let (x, y) = match (&self.visit, self.character.hall) {
            (Some(visit), _) => visit.position(),
            (None, Some((x, y))) => (x - self.hall_origin, y),
            (None, None) => self.world_position(),
        };
        let map = self.map;
        self.link.send(&ServerMessage::Position { map, step, x, y })
    }

    /// Saves the character if anything changed (only once welcomed: before
    /// that nothing has been loaded).
    pub(super) fn save(&mut self, world: &mut ServerWorld) {
        let Some(id) = &self.character_id else {
            return;
        };
        if self.phase != Phase::Playing || !self.unsaved {
            return;
        }
        self.character.visit = self.visit.as_ref().map(Visit::save);
        world.storage.save_character(
            id,
            world.seed.id(),
            &self.character,
            &self.other_worlds,
            Everywhere {
                inventory: self.inventory.clone(),
                sheet: self.sheet.clone(),
                gates: self.gates.clone(),
                playtime: self.activity.played,
                action_bar: self.action_bar.clone(),
                difficulty: self.difficulty,
            },
        );
        self.unsaved = false;
    }

    /// Saves can't be reached, so nothing about this client can be trusted to
    /// load: logs why, tells it, and answers `false` so that it is disconnected.
    /// What others know this character by: their character's id. Nil until there
    /// is a character, when nobody can see them anyway.
    pub(crate) fn entity_id(&self) -> Id {
        self.character_id.map_or(Id(uuid::Uuid::nil()), |id| id.0)
    }

    pub(crate) fn refuse_for_saves(&self, error: &str) -> bool {
        error!("Refusing a client: {error}");
        self.link
            .send(&ServerMessage::Refused(SAVES_TROUBLE.into()));
        false
    }

    /// Forgets everything about its character but the connection: for one being
    /// deleted, before another is made. Nothing is saved.
    pub(super) fn forget_character(&mut self) {
        // The link moves to the fresh client; a spare one stands in meanwhile.
        let (_, spare) = shared::protocol::local_pair();
        let link = std::mem::replace(&mut self.link, spare);
        let fresh = Client::new(link, None);
        let kept = (self.host, self.account, self.world, self.makes_characters);
        *self = Client {
            host: kept.0,
            account: kept.1,
            world: kept.2,
            makes_characters: kept.3,
            // The player's default for a new one, never Ironman.
            difficulty: self.difficulty.map(Difficulty::for_existing),
            ..fresh
        };
    }

    /// Disconnects it: the player leaves their level, and the character is
    /// saved.
    pub(super) fn disconnect(&mut self, world: &mut ServerWorld) {
        self.gone = true;
        sites::leave(self, world);
        self.save(world);
        world.save_sites();
    }
}
