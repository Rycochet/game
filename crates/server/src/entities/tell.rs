//! Telling a client how what it sees differs from what it knew: who came into
//! sight, who went, and everyone's new stretches of movement in one message.

use shared::energy::site_move_cost;
use shared::id::Id;
use shared::mob::EntityLook;
use shared::movement::step_cost;
use shared::protocol::{ServerMessage, Stretch};
use shared::spot::Spot;
use std::collections::BTreeSet;

use crate::connections::Client;

/// The longest a stretch is shown taking: two seconds (a jump across the
/// world map's seam, say, is shown as a jump).
const MOST_TICKS: u32 = 40;

/// Someone seen, and how fast they act (energy a tick).
pub(super) type Seen = (EntityLook, u32);

/// Tells a client how what it can see differs from what it knew (`on_level`:
/// positions in units, otherwise world map points). Everyone who moved is in
/// one `Moves`. A client that has gone is noticed when its messages are next
/// read.
pub(super) fn tell(client: &mut Client, seen: Vec<Seen>, died: &BTreeSet<Id>, on_level: bool) {
    let now_seen: BTreeSet<Id> = seen.iter().map(|(look, _)| look.id).collect();
    let feelings_changed: BTreeSet<Id> = seen
        .iter()
        .filter(|(look, _)| {
            let feeling = client.seen_feelings.get(&look.id);
            let worn = client.seen_worn.get(&look.id);
            let mark = client.seen_marks.get(&look.id);
            feeling.is_some_and(|known| *known != look.feeling)
                || worn.is_some_and(|known| *known != look.worn)
                || mark.is_some_and(|known| *known != look.mark)
        })
        .map(|(look, _)| look.id)
        .collect();
    client.seen_feelings = seen.iter().map(|(l, _)| (l.id, l.feeling)).collect();
    client.seen_worn = seen.iter().map(|(l, _)| (l.id, l.worn.clone())).collect();
    client.seen_marks = seen.iter().map(|(l, _)| (l.id, l.mark.clone())).collect();
    client.seen_entities.retain(|id, _| {
        let stays = now_seen.contains(id);
        if !stays {
            let died = died.contains(id);
            client
                .link
                .send(&ServerMessage::EntityGone { id: *id, died });
        }
        stays
    });
    let mut moves = Vec::new();
    for (look, speed) in seen {
        let at = (look.x, look.y);
        let known = client.seen_entities.insert(look.id, at);
        // Told afresh, the token takes the new colour and the new picture.
        let known = known.filter(|_| !feelings_changed.contains(&look.id));
        match known {
            None => {
                client.link.send(&ServerMessage::EntitySeen(look));
            }
            Some(from) if from != at => moves.push(Stretch {
                id: look.id,
                from,
                to: at,
                ticks: ticks_for(from, at, speed, on_level),
            }),
            Some(_) => {}
        }
    }
    if !moves.is_empty() {
        client.link.send(&ServerMessage::Moves(moves));
    }
}

/// How many ticks going from `from` to `to` takes someone of `speed`, walking
/// on a level (`on_level`, in units) or the world map (points): what the
/// client shows the stretch taking.
fn ticks_for(from: (i64, i64), to: (i64, i64), speed: u32, on_level: bool) -> u32 {
    let cost = if on_level {
        site_move_cost(Spot::new(from.0, from.1), Spot::new(to.0, to.1))
    } else {
        step_cost(from, to)
    };
    cost.div_ceil(speed.max(1)).min(MOST_TICKS)
}
