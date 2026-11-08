//! When a poi stops being secret in the wilderness: on its way in being seen
//! (its entrance, or the way down into one entered from above), or the
//! player stepping inside its cutout (owner). The client is told ("Found")
//! for anything but a town, which everyone knows of, its border is sent now
//! that it may be shown, and the world map sends its chunk again.

use shared::protocol::ServerMessage;

use crate::connections::Client;
use crate::sites::{Place, SetIn, Visit};
use crate::world_map::knows_of;

/// The pois set in whose way in is among the `cells` just seen.
pub(in crate::sites) fn found_by_sight(
    client: &mut Client,
    visit: &Visit,
    cells: &[(i64, i64)],
) -> bool {
    let Place::Wilderness { cutouts, .. } = &visit.place else {
        return true;
    };
    let seen = |set_in: &&SetIn| set_in.entrance.is_some_and(|at| cells.contains(&at));
    let found: Vec<&SetIn> = cutouts.iter().filter(seen).collect();
    learn(client, cutouts, &found)
}

/// The poi whose cutout the player stands in, after a step.
pub(in crate::sites) fn found_by_standing(client: &mut Client) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    let mut sent = true;
    if let Place::Wilderness {
        cutouts,
        in_poi: Some(_),
        ..
    } = &visit.place
    {
        let here = cutouts.iter().find(|set_in| set_in.takes(visit.position));
        sent = learn(client, cutouts, &Vec::from_iter(here));
    }
    client.visit = Some(visit);
    sent
}

/// Marks each of `found` known, if the character didn't know of it, and
/// tells the client.
fn learn(client: &mut Client, cutouts: &[SetIn], found: &[&SetIn]) -> bool {
    let (mut learnt, mut sent) = (false, true);
    for set_in in found {
        let known = client.character.pois.entry(set_in.chunk).or_default();
        if known.known || known.visited {
            continue;
        }
        known.known = true;
        client.unsaved = true;
        // The world map's chunk shows it from now on.
        client.world_chunks.remove(&set_in.chunk);
        learnt = true;
        if !set_in.kind.is_town() {
            sent &= client
                .link
                .send(&ServerMessage::PoiFound { kind: set_in.kind });
        }
    }
    sent && (!learnt || send_borders(client, cutouts))
}

/// The cells round the pois set in that the player knows of (a ring round a
/// secret one would give it away).
pub(in crate::sites) fn send_borders(client: &Client, cutouts: &[SetIn]) -> bool {
    let known = cutouts
        .iter()
        .filter(|set_in| knows_of(client, set_in.chunk, set_in.kind));
    let cells = known.flat_map(|set_in| set_in.border.iter().copied());
    client.link.send(&ServerMessage::Borders(cells.collect()))
}
