//! Whether a character may take a quest: its requirements against what they
//! have done and are, and the hints a giver gives for what is missing (never
//! numbers). An alignment requirement they don't meet hides the quest
//! entirely (owner).

use shared::protocol::Hint;
use shared::quest::{Need, Quest, Slot};

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
            Need::Done(id) => (!book.done.contains_key(&id)).then(|| {
                let own_chain = quest.id.earlier() == Some(id)
                    || matches!((quest.id.slot, id.slot), (Slot::Giver { building: a, .. }, Slot::Giver { building: b, .. }) if a == b && quest.id.town == id.town);
                if own_chain {
                    Hint::Earlier
                } else {
                    let name = (world.pois.in_chunk(id.town)).and_then(|poi| place_name_of(world, &poi));
                    Hint::Elsewhere {
                        town: id.town,
                        name,
                    }
                }
            }),
            Need::Skill { skill, least } => {
                (client.sheet.skills.value(skill) < least).then_some(Hint::Skill(skill))
            }
        };
        hints.extend(hint);
    }
    if hints.is_empty() {
        Verdict::Met
    } else {
        Verdict::Missing(hints)
    }
}
