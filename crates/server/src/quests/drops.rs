//! Quest drops (owner): a "Collect" step's part of a creature ("kobold
//! scales"), dropped only for a character on that step who shared in killing
//! one of its kind at its poi, until the count. It goes straight into what
//! they carry, a quest item like any other; nobody else's loot changes. With
//! no room for it, it is lost and doesn't count.

use shared::creature::CreatureKind;
use shared::item::{Item, ItemKind};
use shared::poi::ChunkPos;
use shared::quest::{Hook, QuestId, Target};
use shared::story::{BodyPart, Remains};

use super::{cache, done_flags, log};
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;

/// The character shared in killing a creature of `kind` at the poi in
/// `site`: each open "Collect" step for it gets one more. `false` if the
/// client has gone.
pub(super) fn drop_remains(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: CreatureKind,
    site: ChunkPos,
) -> bool {
    let wanted = Target::Poi {
        chunk: site,
        hook: Hook::Collect(kind),
    };
    let ids: Vec<QuestId> = client.character.quests.taken.keys().copied().collect();
    let mut sent = true;
    for id in ids {
        let Some(quest) = cache::quest(world, id) else {
            continue;
        };
        let Some(remains) = quest.story.facts.remains else {
            continue;
        };
        let Some(progress) = client.character.quests.taken.get(&id) else {
            continue;
        };
        let done = done_flags(&quest, &progress.counts);
        let step = (quest.open_steps(&done).into_iter())
            .find(|&i| quest.steps[i].target == wanted && !done[i]);
        let Some(step) = step else {
            continue;
        };
        let item = remains_item(world, id, remains);
        if client
            .inventory
            .give(item, world.stacks, &mut new_id)
            .is_err()
        {
            continue;
        }
        let Some(progress) = client.character.quests.taken.get_mut(&id) else {
            continue;
        };
        progress.counts.resize(quest.steps.len(), 0);
        progress.counts[step] += 1;
        progress.ready = quest.ready_to_turn_in(&done_flags(&quest, &progress.counts));
        client.unsaved = true;
        client.update_speed(world.action_clock.ticks());
        sent &= client.send_inventory() && log::tell_changed(client, world, id);
    }
    sent
}

/// One of a quest's remains, named for the creature and part ("Kobold
/// Scales"), in English as items' own names are for now.
fn remains_item(world: &ServerWorld, quest: QuestId, remains: Remains) -> Item {
    let name = format!("{} {}", remains.kind.def().name, part_name(remains.part));
    Item {
        quest: Some(quest),
        name: Some(name),
        made_at: Some(world.made_now()),
        ..Item::new(new_id(), ItemKind::Remains)
    }
}

fn part_name(part: BodyPart) -> &'static str {
    match part {
        BodyPart::Heart => "Heart",
        BodyPart::Liver => "Liver",
        BodyPart::Tooth => "Tooth",
        BodyPart::Fang => "Fang",
        BodyPart::Claw => "Claw",
        BodyPart::Fingernail => "Fingernail Clippings",
        BodyPart::Toenail => "Toenail Clippings",
        BodyPart::Tail => "Tail",
        BodyPart::Horn => "Horn",
        BodyPart::Tusk => "Tusk",
        BodyPart::Ear => "Ear",
        BodyPart::Feather => "Feather",
        BodyPart::Hoof => "Hoof",
        BodyPart::Scale => "Scale",
    }
}
