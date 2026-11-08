//! Towns set into the wilderness are known whole, as in their own visit: a
//! player who steps inside one is sent all of it the client doesn't have (the
//! window always takes in the whole of a town that lies in it).

use crate::connections::Client;
use crate::sites::{Place, Visit, add_masks, cells_in, send_cells};

/// Sends the town the player stands in, if any, as remembered: what the
/// client hasn't been sent on this visit, or has let go of.
pub(super) fn send_town(client: &mut Client) -> bool {
    let Some(mut visit) = client.visit.take() else {
        return true;
    };
    let mut sent = true;
    if let Some(bounds) = town_here(&visit) {
        let mut new = cells_in(bounds);
        for (chunk, mask) in &visit.sent {
            if let Some(found) = new.get_mut(chunk) {
                *found &= !mask;
            }
        }
        new.retain(|_, mask| *mask != 0);
        sent = send_cells(client, &visit, &new, true);
        add_masks(&mut visit.sent, new);
    }
    client.visit = Some(visit);
    sent
}

/// The cells of the town set in that the player stands in, if any.
fn town_here(visit: &Visit) -> Option<(i64, i64, i64, i64)> {
    let Place::Wilderness {
        cutouts,
        in_poi: Some(kind),
        ..
    } = &visit.place
    else {
        return None;
    };
    if !kind.is_town() {
        return None;
    }
    let (x, y) = visit.position;
    cutouts
        .iter()
        .map(|&(_, bounds)| bounds)
        .find(|&(min_x, min_y, width, height)| {
            (min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y)
        })
}
