use shared::poi::ChunkPos;
use shared::quest::{QuestId, Slot};

use super::*;

fn id() -> QuestId {
    QuestId {
        town: ChunkPos { x: 3, y: -2 },
        slot: Slot::Giver {
            building: 7,
            slot: 1,
            chain: 0,
        },
    }
}

#[test]
fn a_book_comes_back_from_ron_as_it_was() {
    let mut book = QuestBook::default();
    book.taken.insert(
        id(),
        Progress {
            at: 100,
            counts: vec![2, 0],
        },
    );
    book.done.insert(
        QuestId {
            slot: Slot::Feeder,
            ..id()
        },
        50,
    );
    book.ended.insert(
        id(),
        Ended {
            at: 10,
            failed: true,
        },
    );
    book.dailies
        .insert((id().town, 7), Dailies { count: 3, last: 90 });
    book.standing.insert(id().town, 25);
    let text = ron::to_string(&book).unwrap();
    assert_eq!(ron::from_str::<QuestBook>(&text).unwrap(), book);
    assert_eq!(
        ron::from_str::<QuestBook>("()").unwrap(),
        QuestBook::default()
    );
}

#[test]
fn a_quest_ended_can_be_taken_again_an_hour_later() {
    let mut book = QuestBook::default();
    assert!(book.may_retake(&id(), 0));
    book.ended.insert(
        id(),
        Ended {
            at: 1000,
            failed: false,
        },
    );
    assert!(!book.may_retake(&id(), 1000 + RETAKE_AFTER - 1));
    assert!(book.may_retake(&id(), 1000 + RETAKE_AFTER));
}
