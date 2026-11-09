//! Characters' names, unique on a server whatever their capitals
//! (`shared::name`): an index of every name taken, saved beside the accounts
//! (`SaveKey::Names`). A server without one makes it from every account,
//! renaming characters whose names are taken or can't be names any more.

use std::collections::BTreeMap;

use bevy::log::info;
use serde::{Deserialize, Serialize};
use shared::name::{self, PLAIN_NAME};
use shared::protocol::CharacterId;

use super::Accounts;
use crate::saves::{Save, SaveKey};

/// Every name taken, by its `name::key`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct NameIndex {
    pub names: BTreeMap<String, CharacterId>,
}

impl Accounts {
    /// Loads the index, or makes it. An error if saves can't be reached: names
    /// could then be given twice.
    pub(super) fn index_names(&mut self) -> Result<(), String> {
        let Some(queue) = self.queue.clone() else {
            return Ok(());
        };
        if let Some(Save::Names(index)) = queue.load(&SaveKey::Names)? {
            self.names = index;
            return Ok(());
        }
        for id in queue.accounts()? {
            self.account(&id)?;
            let account = self.loaded.get_mut(&id).expect("just loaded");
            let mut renamed = false;
            for character in &mut account.characters {
                let fitting = name::checked(&character.name).unwrap_or(PLAIN_NAME.into());
                let unique = unique(&self.names, &fitting);
                self.names.names.insert(name::key(&unique), character.id);
                if unique != character.name {
                    info!(
                        "Character {} renamed: its name is taken, or can't be one now",
                        character.id.0
                    );
                    character.name = unique;
                    renamed = true;
                }
            }
            if renamed {
                self.save(&id);
            }
        }
        self.save_names();
        Ok(())
    }

    /// Takes the name asked for (checked as `name::checked` does): what it will
    /// be, or why not.
    pub(super) fn take_name(
        &mut self,
        asked: &str,
        character: CharacterId,
    ) -> Result<String, String> {
        let name = name::checked(asked)?;
        if self.names.names.contains_key(&name::key(&name)) {
            return Err(format!("{name} is taken: choose another name"));
        }
        self.names.names.insert(name::key(&name), character);
        self.save_names();
        Ok(name)
    }

    /// Takes `base`, or it with the first number free after it.
    pub(super) fn take_numbered(&mut self, base: &str, character: CharacterId) -> String {
        let name = unique(&self.names, base);
        self.names.names.insert(name::key(&name), character);
        self.save_names();
        name
    }

    /// A deleted character's name is free again.
    pub(super) fn free_name(&mut self, character: CharacterId) {
        self.names.names.retain(|_, holder| *holder != character);
        self.save_names();
    }

    fn save_names(&self) {
        if let Some(queue) = &self.queue {
            queue.save(SaveKey::Names, Save::Names(self.names.clone()));
        }
    }
}

/// `base`, or it with the first number after it no one has.
fn unique(index: &NameIndex, base: &str) -> String {
    let taken = |name: &str| index.names.contains_key(&name::key(name));
    if !taken(base) {
        return base.to_owned();
    }
    (2..)
        .map(|number| name::numbered(base, number))
        .find(|name| !taken(name))
        .expect("numbers run on")
}
