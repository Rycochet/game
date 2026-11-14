//! Whether a character may take a quest: its requirements against what they
//! have done and are, and the hints a giver gives for what is missing (never
//! numbers). An alignment requirement they don't meet hides the quest
//! entirely (owner).

use shared::protocol::{Heard, Hint};
use shared::quest::{Need, Quest, QuestId, Slot};

use super::standing::standing;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::place_name_of;

/// What a character makes of a quest's requirements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// They may take it.
    Met,
    /// Not yet: what the giver says of it.
    Missing(Vec<Hint>),
    /// They are never shown it.
    Hidden,
}

/// Judges a quest's requirements for `client`'s character.
pub(crate) fn judge(quest: &Quest, client: &Client, world: &ServerWorld) -> Verdict {
    let book = &client.character.quests;
    let mut hints = Vec::new();
    for requirement in &quest.requirements {
        let hint = match requirement.need {
            Need::Alignment(band) => {
                let ours = client.sheet.alignment.map(|alignment| alignment.band());
                if ours != Some(band) {
                    return Verdict::Hidden;
                }
                continue;
            }
            Need::Standing { town, least } => {
                (standing(book, town) < least).then_some(Hint::Standing { least })
            }
            Need::Done(id) => (!book.done.contains_key(&id)).then(|| done_hint(quest, id, world)),
            Need::Skill { skill, least } => {
                (client.sheet.skills.value(skill) < least).then_some(Hint::Skill(skill))
            }
            Need::Moon { moon } => (!world.moon_up(moon)).then(|| Hint::Moon {
                colour: world.moon_colour(moon),
            }),
        };
        hints.extend(hint);
    }
    if hints.is_empty() {
        Verdict::Met
    } else {
        Verdict::Missing(hints)
    }
}

/// What the giver of `quest` says of `id`, a quest it needs done first.
fn done_hint(quest: &Quest, id: QuestId, world: &ServerWorld) -> Hint {
    if same_giver(quest.id, id) {
        return Hint::Earlier;
    }
    match heard_from(id, world) {
        Some(Heard::Neighbour { giver, kind }) => Hint::Neighbour { giver, kind },
        Some(Heard::Town { name }) => Hint::Elsewhere {
            town: id.town,
            name,
        },
        None => Hint::Elsewhere {
            town: id.town,
            name: None,
        },
    }
}

/// Who a character taking `quest` was given good words by: the giver of the
/// first quest it needed done that isn't the giver's own.
pub(crate) fn heard(quest: &Quest, world: &ServerWorld) -> Option<Heard> {
    (quest.requirements.iter()).find_map(|requirement| match requirement.need {
        Need::Done(id) if !same_giver(quest.id, id) => heard_from(id, world),
        _ => None,
    })
}

/// Whether two quests have the same giver: the same building of one town.
fn same_giver(a: QuestId, b: QuestId) -> bool {
    a.town == b.town
        && matches!((a.slot, b.slot), (Slot::Giver { building: x, .. }, Slot::Giver { building: y, .. }) if x == y)
}

/// Who gave quest `id`: another giver of the same town, or a town by name.
fn heard_from(id: QuestId, world: &ServerWorld) -> Option<Heard> {
    let Some(town) = world.quests.kept(id.town) else {
        let name = (world.pois.in_chunk(id.town)).and_then(|poi| place_name_of(world, &poi));
        return Some(Heard::Town { name });
    };
    if let Slot::Giver { building, .. } = id.slot {
        let quest = town.quests.iter().find(|quest| quest.id == id)?;
        let look = town.buildings.iter().find(|look| look.id == building)?;
        return Some(Heard::Neighbour {
            giver: quest.story.cast.giver().person.clone(),
            kind: look.kind,
        });
    }
    let name = place_name_of(world, &town.poi);
    Some(Heard::Town { name })
}
