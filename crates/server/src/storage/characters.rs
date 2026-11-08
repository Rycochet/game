//! Loading and saving characters.

use bevy::prelude::*;
use shared::ability::ActionBar;
use shared::difficulty::Difficulty;
use shared::item::Inventory;
use shared::protocol::CharacterId;
use shared::sheet::Sheet;
use shared::world::WorldId;
use std::collections::{BTreeMap, BTreeSet};

use super::{CharacterSave, CharacterWorldSave, Storage};
use crate::saves::{Save, SaveKey};

/// What goes with a character from world to world, saved with what they know
/// of each.
#[derive(Clone, Debug, Default)]
pub struct Everywhere {
    pub inventory: Inventory,
    pub sheet: Sheet,
    /// `CharacterSave::gates`.
    pub gates: BTreeSet<u32>,
    /// `CharacterSave::playtime`.
    pub playtime: u64,
    /// `CharacterSave::action_bar`.
    pub action_bar: ActionBar,
    /// `CharacterSave::difficulty`.
    pub difficulty: Option<Difficulty>,
}

impl Storage {
    /// A character's whole save, if they have one. A broken one is put aside (and
    /// logged), so they start again. An error if it couldn't be reached: it may
    /// be fine, and starting again would save a new character over it.
    fn character(&self, id: &CharacterId) -> Result<Option<CharacterSave>, String> {
        let Some(queue) = &self.queue else {
            return Ok(None);
        };
        Ok(match queue.load(&SaveKey::Character(*id))? {
            Some(Save::Character(save)) => Some(save),
            _ => None,
        })
    }

    /// The world a character is in, by their save: `None` if they are new.
    pub fn character_world(&self, id: &CharacterId) -> Result<Option<WorldId>, String> {
        Ok(self.character(id)?.map(|save| save.world))
    }

    /// What a character knows of `world` (nothing if they haven't been there),
    /// and of the other worlds they have been to: the world they are in is
    /// `character_world`, or for someone new, wherever the server puts them.
    /// Knowledge from before `forget_before` is ignored, and saved over.
    pub fn load_character(
        &self,
        id: &CharacterId,
        world: WorldId,
    ) -> Result<(CharacterWorldSave, BTreeMap<WorldId, CharacterWorldSave>), String> {
        let save = self.character(id)?;
        let mut worlds = save.map_or_else(BTreeMap::new, |save| save.worlds);
        let here = worlds
            .remove(&world)
            .filter(|here| self.not_forgotten(here.forgotten_to, &id.0.to_string()))
            .unwrap_or_default();
        Ok((here, worlds))
    }

    /// What a character carries, which goes with them from world to world. Nothing
    /// if they are new.
    pub fn load_inventory(&self, id: &CharacterId) -> Result<Inventory, String> {
        let save = self.character(id)?;
        Ok(save.map_or_else(Inventory::default, |save| save.inventory))
    }

    /// What a character is like: `None` if they are new.
    pub fn load_sheet(&self, id: &CharacterId) -> Result<Option<Sheet>, String> {
        Ok(self.character(id)?.map(|save| save.sheet))
    }

    /// The World Gates a character has gone through (`CharacterSave::gates`).
    pub fn load_gates(&self, id: &CharacterId) -> Result<BTreeSet<u32>, String> {
        Ok(self
            .character(id)?
            .map_or_else(BTreeSet::new, |save| save.gates))
    }

    /// How long a character has been played, in action ticks (0 if new).
    pub fn load_playtime(&self, id: &CharacterId) -> Result<u64, String> {
        Ok(self.character(id)?.map_or(0, |save| save.playtime))
    }

    /// A character's action bar (a new character's if they are new).
    pub fn load_action_bar(&self, id: &CharacterId) -> Result<ActionBar, String> {
        Ok(self
            .character(id)?
            .map_or_else(ActionBar::default, |mut save| {
                save.action_bar.catch_up();
                save.action_bar
            }))
    }

    /// A character's difficulty, if they have one saved.
    pub fn load_difficulty(&self, id: &CharacterId) -> Result<Option<Difficulty>, String> {
        Ok(self.character(id)?.and_then(|save| save.difficulty))
    }

    /// Deletes a character's save (dev tools), if they have one.
    pub fn remove_character(&self, id: &CharacterId) {
        if let Some(queue) = &self.queue {
            queue.remove(SaveKey::Character(*id));
        }
    }

    /// Saves a character who is in `world`, knowing `here` of it.
    pub fn save_character(
        &self,
        id: &CharacterId,
        world: WorldId,
        here: &CharacterWorldSave,
        others: &BTreeMap<WorldId, CharacterWorldSave>,
        everywhere: Everywhere,
    ) {
        if let Some(queue) = &self.queue {
            let mut worlds = others.clone();
            let forgotten_to = self.forget_before;
            worlds.insert(
                world,
                CharacterWorldSave {
                    forgotten_to,
                    ..here.clone()
                },
            );
            queue.save(
                SaveKey::Character(*id),
                Save::Character(CharacterSave {
                    world,
                    worlds,
                    inventory: everywhere.inventory,
                    sheet: everywhere.sheet,
                    gates: everywhere.gates,
                    playtime: everywhere.playtime,
                    action_bar: everywhere.action_bar,
                    difficulty: everywhere.difficulty,
                }),
            );
        }
    }
}
