//! The items a quest pays besides coins: the artifact line's artifact or the
//! planet's set item (one per planet per character, for good: a character
//! who has it gets an artifact instead), made to fit the character.

use shared::item::{Item, ItemKind, MadeAt, Quality};
use shared::item_form::{ItemOrder, make_item};
use shared::quest::{Quest, artifact_kind, set_item_of};
use shared::story::RewardPart;

use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;
use crate::storage::QuestBook;

/// `quest` with its reward as `book`'s character would be paid it: the set
/// item becomes an artifact once found by another quest. Both the reward and
/// the story's own words of it change.
pub(super) fn as_seen(book: &QuestBook, mut quest: Quest) -> Quest {
    if book.set_item.is_some_and(|paid| paid != quest.id) {
        for part in quest
            .reward
            .iter_mut()
            .chain(quest.story.facts.reward.iter_mut())
        {
            if *part == RewardPart::SetItem {
                *part = RewardPart::Artifact;
            }
        }
    }
    quest
}

/// The items `quest` pays `client`'s character other than coins, and
/// whether one is the planet's set item. Items, training and favours of
/// ordinary quests aren't given yet.
pub(super) fn prize_items(
    client: &Client,
    world: &ServerWorld,
    quest: &Quest,
) -> (Vec<Item>, bool) {
    let quest = as_seen(&client.character.quests, quest.clone());
    let made_at = world.made_now();
    let mut items = Vec::new();
    let mut set_item = false;
    for part in &quest.reward {
        match part {
            RewardPart::Artifact => {
                let id = new_id();
                let kind = artifact_kind(id.seed());
                items.push(made(client, kind, Quality::Artifact, false, id, made_at));
            }
            RewardPart::SetItem => {
                let (kind, quality) = set_item_of(world.seed.value());
                items.push(made(client, kind, quality, true, new_id(), made_at));
                set_item = true;
            }
            _ => {}
        }
    }
    (items, set_item)
}

/// One item made for the character's body, from its own id's seed.
fn made(
    client: &Client,
    kind: ItemKind,
    quality: Quality,
    set: bool,
    id: shared::id::Id,
    made_at: MadeAt,
) -> Item {
    let order = ItemOrder {
        set,
        ..ItemOrder::plain(kind, quality)
    };
    make_item(
        &order,
        client.inventory.creature,
        id,
        id.seed(),
        Some(made_at),
        &mut new_id,
    )
}
