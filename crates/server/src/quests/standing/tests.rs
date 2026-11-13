use super::*;

#[test]
fn points_make_tiers() {
    assert_eq!(tier(0), Standing::Stranger);
    assert_eq!(tier(FOR_QUEST), Standing::Known);
    assert_eq!(tier(2 * FOR_QUEST), Standing::Trusted);
    assert_eq!(tier(74), Standing::Trusted);
    assert_eq!(tier(1000), Standing::Champion);
}

#[test]
fn standing_adds_up_by_town() {
    let mut book = QuestBook::default();
    let town = ChunkPos { x: 1, y: 2 };
    assert_eq!(standing(&book, town), Standing::Stranger);
    add(&mut book, town, FOR_QUEST);
    assert_eq!(standing(&book, town), Standing::Known);
    assert_eq!(standing(&book, ChunkPos { x: 2, y: 2 }), Standing::Stranger);
}
