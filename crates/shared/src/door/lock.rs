//! Locks on doors: information on the door for now; one day an item that can
//! be taken out and put in another door.

use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::item::Quality;

/// A door's lock. Its key opens it by its `id` (`ItemForm::Key`), which stays
/// the same wherever the lock goes. Only a closed door can be locked, and a
/// locked one can't be opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    pub id: Id,
    /// Nearly always Common.
    pub quality: Quality,
    pub locked: bool,
}
