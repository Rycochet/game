//! Following each player for quests: their log when they come into a world,
//! steps moving on as they arrive at places and kill, and the markers of the
//! town they are in.

use shared::creature::CreatureKind;
use shared::hall::GateCode;
use shared::poi::ChunkPos;
use shared::quest::QuestId;
use shared::world::WorldId;

use super::clock::unix_now;
use super::dots::{DotsSeen, tell_dots};
use super::planets::tell_planets;
use super::progress::{Event, advance};
use super::{log, send_markers, standing};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::QuestPlace;
use crate::storage::SiteId;

/// What a client was last told about, for quests (`Client::quest_seen`).
#[derive(Clone, Debug, Default)]
pub(crate) struct Seen {
    /// The world whose log it was sent.
    world: Option<WorldId>,
    /// The poi and level the player was last at.
    place: Option<(ChunkPos, u32, Option<QuestId>)>,
    /// The town they are in, if any: markers follow.
    town: Option<ChunkPos>,
    /// They are in the Hall, and have been told their planets.
    hall: bool,
    /// A planet whose log they asked for this tick (`planet_log`).
    planet_log: Option<WorldId>,
    /// The maps' quest dots, as last sent.
    pub(super) dots: DotsSeen,
    /// The signs over quest bosses in sight, as last sent.
    pub(super) bosses: Vec<shared::protocol::BossMark>,
    /// The sign over the player's quest item in sight, as last sent.
    pub(super) finds: Vec<shared::protocol::FindMark>,
    /// The gait of someone escorted with them, as last sent (`None`: not yet).
    pub(super) escort_gait: Option<Option<shared::movement::Gait>>,
    /// Where they were on the world map, for someone escorted a step behind.
    pub(super) trail: super::escort_look::Trail,
}

impl Seen {
    pub(crate) fn town(&self) -> Option<ChunkPos> {
        self.town
    }

    pub(crate) fn ask_planet_log(&mut self, world: WorldId) {
        self.planet_log = Some(world);
    }

    pub(crate) fn take_planet_log(&mut self) -> Option<WorldId> {
        self.planet_log.take()
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
        // Learned here, so the Quest Log can name the planet from anywhere.
        client.character.quests.code = GateCode::of_planet(world.seed);
        let whole = log::whole_log(client, world);
        client
            .link
            .send(&shared::protocol::ServerMessage::Quest(whole));
        super::ahead::tell_ahead(client, world);
    }
    let in_hall = client.character.hall.is_some();
    if in_hall && !client.quest_seen.hall {
        tell_planets(client);
    }
    client.quest_seen.hall = in_hall;
    let place: Option<QuestPlace> = client.visit.as_ref().and_then(|visit| visit.quest_place());
    let key = place.map(|place| (place.poi.chunk, place.level, place.dungeon));
    let arrived = key != client.quest_seen.place;
    if arrived {
        arrive(client, world, place, now);
    }
    super::escort::keep_escorts(client, world, arrived);
    tell_dots(client, world);
    super::boss_signs::tell_bosses(client, world);
    super::finds::tell_finds(client, world);
    super::dungeon_reset::reset_due(client, world, now);
}

/// On coming to another place: steps moving on, and the town's markers.
fn arrive(client: &mut Client, world: &mut ServerWorld, place: Option<QuestPlace>, now: i64) {
    client.quest_seen.place = place.map(|place| (place.poi.chunk, place.level, place.dungeon));
    client.quest_seen.town = place
        .filter(|place| place.poi.kind.is_town())
        .map(|place| place.poi.chunk);
    if let Some(place) = place {
        advance(client, world, Event::Arrived(place), now);
    }
    super::arrivals::quest_mobs(client, world);
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
    boss: bool,
) {
    let dungeon = match site {
        SiteId::Quest { quest, .. } => Some(quest),
        _ => None,
    };
    let site = match site {
        SiteId::Poi(chunk) => Some(chunk),
        _ => None,
    };
    let at = client.world_position();
    if standing::killed_near(&mut client.character.quests, world, kind, at) {
        client.unsaved = true;
    }
    let now = unix_now();
    if boss && let Some(chunk) = site {
        super::boss_kill::slew_boss(client, world, chunk, now);
    }
    if let Some(chunk) = site {
        super::drops::drop_remains(client, world, kind, chunk);
    }
    advance(
        client,
        world,
        Event::Killed {
            kind,
            site,
            dungeon,
        },
        now,
    );
}
