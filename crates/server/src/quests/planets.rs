//! The planets where a character has quest history: for the "!" and "?" over
//! the Hall's gates that lead to them (owner), and the Quest Log's list of
//! planets.

use shared::protocol::{PlanetQuests, QuestNews, ServerMessage};

use crate::connections::Client;
use crate::storage::QuestBook;

/// Each planet with quest history, and whether quests are taken there and one
/// waits to be handed in: the world the character is in (or came from) and
/// every other they have been to.
pub(crate) fn planets(client: &Client) -> Vec<PlanetQuests> {
    let here = client.world.map(|world| (world, &client.character.quests));
    let others = (client.other_worlds().iter()).map(|(&world, save)| (world, &save.quests));
    here.into_iter()
        .chain(others)
        .filter(|(_, book)| book.has_history())
        .map(|(world, book)| PlanetQuests {
            world,
            code: book.code,
            here: Some(world) == client.world,
            taken: !book.taken.is_empty(),
            ready: any_ready(book),
        })
        .collect()
}

fn any_ready(book: &QuestBook) -> bool {
    book.taken.values().any(|progress| progress.ready)
}

/// Tells the client its planets. `false` if it has gone.
pub(crate) fn tell_planets(client: &mut Client) -> bool {
    let news = QuestNews::Planets(planets(client));
    client.link.send(&ServerMessage::Quest(news))
}
