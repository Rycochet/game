//! Doors as a player sees and remembers them.

use shared::door::DoorLook;
use shared::dungeon::Cell;

use super::VisitDoors;
use crate::connections::Client;
use crate::sites::Visit;
use crate::storage::{RememberedDoors, SiteId};

/// How the player sees the door at `(x, y)` of a level (site `id`, level
/// `index`, the cell kept as `kept`): as it is now, or as they remember it if
/// `remembered`. A secret door only once found (it is then remembered), while
/// it stands open, or with truesight; `None` for one they would take for wall,
/// or for a cell without a door.
pub(in crate::sites) fn door_looks(
    client: &Client,
    doors: &VisitDoors,
    (id, index): (SiteId, u32),
    cell: Cell,
    (x, y): (i64, i64),
    kept: (i64, i64),
    remembered: bool,
) -> Option<DoorLook> {
    let door = doors.get(&(x, y))?;
    let known = remembered_doors(client, id, index).and_then(|doors| doors.get(&kept).copied());
    let now = door.door.swing;
    let shown = client.truesight || known.is_some() || (!remembered && !now.is_shut());
    if cell == Cell::SecretDoor && !shown {
        return None;
    }
    let swing = if remembered {
        known.unwrap_or(now)
    } else {
        now
    };
    let mut look = door.door.look(x, y);
    look.swing = swing;
    Some(look)
}

/// The doors a character remembers on a level.
fn remembered_doors(client: &Client, id: SiteId, index: u32) -> Option<&RememberedDoors> {
    client.character.doors.get(&id)?.get(&index)
}

/// The player sees the door at `(x, y)` of their visit's level: it is
/// remembered as it is now (a secret door once they can tell it is one), on
/// every map the cell is kept on. Whether that changed what they remembered,
/// so the client is to be told.
pub(in crate::sites) fn remember_doors(
    client: &mut Client,
    visit: &Visit,
    (x, y): (i64, i64),
) -> bool {
    let Some(door) = visit.doors.get(&(x, y)) else {
        return false;
    };
    let swing = door.door.swing;
    let secret = visit.level.level.cell(x, y) == Cell::SecretDoor;
    let mut places = vec![(visit.id, visit.index, (visit.kept_at(x), y))];
    for mirror in &visit.mirrors {
        if let Some(at) = mirror.of(x, y) {
            places.push((mirror.id, mirror.index, at));
        }
    }
    let (id, index, kept) = places[0];
    let known = remembered_doors(client, id, index).is_some_and(|doors| doors.contains_key(&kept));
    if secret && swing.is_shut() && !known && !client.truesight {
        // It still looks like wall.
        return false;
    }
    let mut changed = false;
    for (id, index, at) in places {
        let doors = client.character.doors.entry(id).or_default();
        changed |= doors.entry(index).or_default().insert(at, swing) != Some(swing);
    }
    client.unsaved |= changed;
    changed
}
