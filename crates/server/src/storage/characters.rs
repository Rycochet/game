//! Loading and saving characters.

use bevy::prelude::*;
use shared::item::Inventory;
use shared::protocol::CharacterId;
use shared::sheet::Sheet;
use shared::world::WorldId;
use std::collections::BTreeMap;

use super::{CharacterSave, CharacterWorldSave, Storage};
use crate::saves::{Save, SaveKey};

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

    /// What a character knows of `world` (nothing if they haven't been there),
    /// and of the other worlds they have been to. Being given `world` is how
    /// they come to be in it, wherever their save says they were: a server only
    /// runs one world so far. Knowledge from before `forget_before` is ignored,
    /// and saved over.
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

    /// Saves a character who is in `world`, knowing `here` of it.
    pub fn save_character(
        &self,
        id: &CharacterId,
        world: WorldId,
        here: &CharacterWorldSave,
        others: &BTreeMap<WorldId, CharacterWorldSave>,
        inventory: &Inventory,
        sheet: &Sheet,
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
                    inventory: inventory.clone(),
                    sheet: sheet.clone(),
                }),
            );
        }
    }
}
