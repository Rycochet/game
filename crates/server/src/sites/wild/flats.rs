//! Telling the client which ground is kept level (`FlatGround`): a cutout's,
//! along with the first of its cells it is sent in a window, so it draws them
//! and costs steps on them as the server does, and learns of a secret poi's
//! no sooner than its cells.

use shared::protocol::ServerMessage;

use super::super::cells::cells_in;
use crate::connections::Client;
use crate::sites::{Place, Visit};
use crate::storage::CellMasks;

/// Sends the flats of the cutouts that `masks` (cells about to be sent)
/// reaches into and the visit hasn't sent any cells of yet. `false` if the
/// client has gone.
pub(in crate::sites) fn send_flats(client: &Client, visit: &Visit, masks: &CellMasks) -> bool {
    let Place::Wilderness { cutouts, .. } = &visit.place else {
        return true;
    };
    let touches = |cells: &CellMasks, other: &CellMasks| {
        cells
            .iter()
            .any(|(chunk, mask)| other.get(chunk).is_some_and(|m| m & mask != 0))
    };
    let new: Vec<_> = cutouts
        .iter()
        .filter(|set_in| {
            let cells = cells_in(set_in.bounds);
            touches(&cells, masks) && !touches(&cells, &visit.sent)
        })
        .map(|set_in| set_in.flat)
        .collect();
    new.is_empty() || client.link.send(&ServerMessage::FlatGround(new))
}
