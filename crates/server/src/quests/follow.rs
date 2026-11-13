//! Following each player for quests: their log when they come into a world,
//! steps moving on as they arrive at places and kill, and the markers of the
//! town they are in.

use shared::creature::CreatureKind;
use shared::poi::ChunkPos;
use shared::world::WorldId;

use super::clock::unix_now;
use super::planets::tell_planets;
use super::progress::{Event, advance};
use super::{log, send_markers, standing};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::QuestPlace;
use crate::storage::SiteId;

/// What a client was last told about, for quests (`Client::quest_seen`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Seen {
    /// The world whose log it was sent.
    world: Option<WorldId>,
    /// The poi and level the player was last at.
    place: Option<(ChunkPos, u32)>,
    /// The town they are in, if any: markers follow.
    town: Option<ChunkPos>,
    /// They are in the Hall, and have been told their planets.
    hall: bool,
}

impl Seen {
    pub(crate) fn town(&self) -> Option<ChunkPos> {
        self.town
    }
}

/// Every tick, each player in `world`: their log on coming into the world,
/// and steps and markers on coming to another place.
pub(crate) fn follow(clients: &mut [Client], world: &mut ServerWorld) {
    let now = unix_now();
    for client in clients.iter_mut() {
        if client.phase != crate::connections::Phase::Playing || client.is_gone() {
            continue;
        }
        follow_one(client, world, now);
    }
}

fn follow_one(client: &mut Client, world: &mut ServerWorld, now: i64) {
    if client.quest_seen.world != client.world {
        client.quest_seen = Seen {
            world: client.world,
            ..Seen::default()
        };
        let whole = log::whole_log(client, world);
        client
            .link
            .send(&shared::protocol::ServerMessage::Quest(whole));
    }
    let in_hall = client.character.hall.is_some();
    if in_hall && !client.quest_seen.hall {
        tell_planets(client);
    }
    client.quest_seen.hall = in_hall;
    let place: Option<QuestPlace> = client.visit.as_ref().and_then(|visit| visit.quest_place());
    let key = place.map(|place| (place.poi.chunk, place.level));
    if key == client.quest_seen.place {
        return;
    }
    client.quest_seen.place = key;
    client.quest_seen.town = place
        .filter(|place| place.poi.kind.is_town())
        .map(|place| place.poi.chunk);
    if let Some(place) = place {
        advance(client, world, Event::Arrived(place), now);
    }
    if let Some(town) = client.quest_seen.town {
        send_markers(client, world, town, now);
    }
}

/// The player shared in killing a creature of `kind` on a level of `site`:
/// steps counting it move on, and towns round about think better of them if
/// it was hostile.
pub(crate) fn killed(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: CreatureKind,
    site: SiteId,
) {
    let site = match site {
        SiteId::Poi(chunk) => Some(chunk),
        _ => None,
    };
    let at = client.world_position();
    if standing::killed_near(&mut client.character.quests, world, kind, at) {
        client.unsaved = true;
    }
    advance(client, world, Event::Killed { kind, site }, unix_now());
}
