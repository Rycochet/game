//! Who an Escort step sees home (owner: a relative or the household's pet,
//! `shared::quest` templates), as a mob: from the quest's story, the same
//! each time it is made, for one character.

use shared::creature::CreatureKind;
use shared::gender::Gender;
use shared::id::Id;
use shared::mob::{Spawn, spawn_at};
use shared::quest::{Hook, Quest, QuestId, Target};
use shared::rng::Seed;
use shared::story::Role;

use crate::ServerWorld;
use crate::connections::Client;
use crate::entities::{Escortee, escortee_mob};

/// Who is escorted in a quest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Who {
    pub(super) kind: CreatureKind,
    pub(super) gender: Gender,
    pub(super) name: String,
    pub(super) seed: Seed,
}

/// The step of `quest` that sees someone home, if it has one: its index and
/// the chunk of the poi they are found at.
pub(super) fn escort_step(quest: &Quest) -> Option<(usize, shared::poi::ChunkPos)> {
    quest
        .steps
        .iter()
        .enumerate()
        .find_map(|(i, step)| match step.target {
            Target::Poi {
                chunk,
                hook: Hook::Escortee,
            } => Some((i, chunk)),
            _ => None,
        })
}

/// Who `quest`'s escort is: its story's pet if it has one, else the giver's
/// relative.
pub(super) fn who(quest: &Quest) -> Option<Who> {
    let cast = &quest.story.cast;
    let seed = quest.story.text_seed.derive("escortee");
    if let Some(pet) = &cast.pet {
        return Some(Who {
            kind: pet.pet.kind.creature(),
            gender: pet.pet.gender,
            name: pet.pet.name.clone(),
            seed,
        });
    }
    let person = &cast.member(Role::Relative)?.person;
    let name = match &person.family {
        Some(family) => format!("{} {family}", person.given),
        None => person.given.clone(),
    };
    Some(Who {
        kind: person.kind,
        gender: person.gender,
        name,
        seed,
    })
}

/// The escortee's id, the same each time for one character and quest.
pub(super) fn escortee_id(client: &Client, quest: QuestId) -> Id {
    let who = client.entity_id();
    let name = format!("escortee {quest:?}");
    Id(uuid::Uuid::new_v5(&who.0, name.as_bytes()))
}

/// How the escortee is rolled, standing at `cell`: its look from its own
/// seed, its gender the story's, carrying no money.
pub(super) fn spawn(who: &Who, cell: (i64, i64)) -> Spawn {
    let mut spawn = spawn_at(who.seed, who.kind, 0, cell);
    if spawn.gender != who.gender {
        let (rolled, own) = (spawn.gender.bonus(), who.gender.bonus());
        for which in shared::sheet::Attribute::ALL {
            let now = spawn
                .attributes
                .get(which)
                .saturating_sub(rolled.get(which));
            spawn.attributes.set(which, now + own.get(which));
        }
        spawn.gender = who.gender;
        spawn.appearance = spawn.appearance.fitted(who.gender);
    }
    spawn.coins = 0;
    spawn
}

/// The escortee as a mob at `cell`, following its quester or not.
pub(super) fn made(
    world: &ServerWorld,
    client: &Client,
    quest: QuestId,
    who: &Who,
    (cell, following): ((i64, i64), bool),
) -> crate::entities::Mob {
    let spawn = spawn(who, cell);
    let escort = Escortee {
        quester: client.entity_id(),
        quest,
        name: who.name.clone(),
        following,
    };
    escortee_mob(world, escortee_id(client, quest), &spawn, escort)
}
