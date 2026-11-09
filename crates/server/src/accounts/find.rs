//! Finding a character's account: by the character's name (the name index),
//! or by the character, looking through every account if it isn't loaded.

use shared::name;
use shared::protocol::CharacterId;

use super::{AccountId, Accounts};

impl Accounts {
    /// The character with this name (whatever its capitals) and its account.
    /// An error if saves can't be reached.
    pub(crate) fn find_character(
        &mut self,
        name: &str,
    ) -> Result<Option<(CharacterId, AccountId)>, String> {
        let Some(&character) = self.names.names.get(&name::key(name.trim())) else {
            return Ok(None);
        };
        Ok(self
            .owner_of(&character)?
            .map(|account| (character, account)))
    }

    /// The account a character is on. An error if saves can't be reached.
    pub(crate) fn owner_of(
        &mut self,
        character: &CharacterId,
    ) -> Result<Option<AccountId>, String> {
        let holds = |save: &super::AccountSave| save.characters.iter().any(|c| c.id == *character);
        let loaded = self.loaded.iter().find(|(_, save)| holds(save));
        if let Some((id, _)) = loaded {
            return Ok(Some(*id));
        }
        let Some(queue) = self.queue.clone() else {
            return Ok(None);
        };
        for id in queue.accounts()? {
            if !self.loaded.contains_key(&id) && holds(self.account(&id)?) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }
}
