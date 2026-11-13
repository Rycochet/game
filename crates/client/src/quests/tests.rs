use super::poi_marks::marks_of;
use super::*;
use shared::creature::CreatureKind;
use shared::poi::ChunkPos;
use shared::protocol::{LogEntry, Marker, QuestState};
use shared::quest::{Hook, QuestId, QuestKind, Slot, Step, Target};
use shared::rng::Seed;
use shared::story::{
    Direction, DistanceBand, Goal, Household, Place, PlaceDescription, PlaceKind, Request, tell,
};

const TOWN: ChunkPos = ChunkPos { x: 0, y: 0 };
const CAVE: ChunkPos = ChunkPos { x: 2, y: 1 };
const CRYPT: ChunkPos = ChunkPos { x: -1, y: 3 };

/// A quest of two steps in series: exploring the cave, then the crypt.
pub(crate) fn entry(state: QuestState, counts: Vec<u32>) -> LogEntry {
    let household = Household::new(CreatureKind::Human, Seed::new(7));
    let mut request = Request::new(Seed::new(3), household, 0, "Ashford", Goal::Visit);
    request.place = Some(Place {
        description: PlaceDescription::new(PlaceKind::Cave, false),
        name: None,
        direction: Direction::East,
        distance: DistanceBand::Short,
    });
    let step = |chunk, after| Step {
        goal: Goal::Explore,
        target: Target::Poi {
            chunk,
            hook: Hook::Explore,
        },
        count: None,
        after,
        alternatives: Vec::new(),
    };
    LogEntry {
        id: QuestId {
            town: TOWN,
            slot: Slot::Feeder,
        },
        kind: QuestKind::Daily,
        building: 4,
        state,
        story: Box::new(tell(&request).story),
        steps: vec![step(CAVE, vec![]), step(CRYPT, vec![0])],
        counts,
        reward: Vec::new(),
        at: 0,
    }
}

fn book_of(entry: LogEntry) -> QuestBook {
    let mut book = QuestBook::default();
    book.entries.insert(entry.id, entry);
    book
}

#[test]
fn only_steps_that_can_be_done_now_mark_their_pois() {
    let marks = marks_of(&book_of(entry(QuestState::Taken, vec![0, 0])));
    let daily = MarkLook::new(QuestKind::Daily, false);
    assert_eq!(marks.get(&CAVE), Some(&daily));
    assert_eq!(marks.get(&CRYPT), None, "it waits for the cave");

    let marks = marks_of(&book_of(entry(QuestState::Taken, vec![1, 0])));
    assert_eq!(marks.get(&CAVE), None, "done");
    assert_eq!(marks.get(&CRYPT), Some(&daily));
}

#[test]
fn a_quest_to_hand_in_marks_its_town_with_a_question_mark() {
    let marks = marks_of(&book_of(entry(QuestState::Ready, vec![1, 1])));
    assert_eq!(marks.len(), 1);
    assert!(marks[&TOWN].ready);
    assert!(marks_of(&book_of(entry(QuestState::Done, vec![1, 1]))).is_empty());
}

#[test]
fn markers_are_coloured_by_kind_and_grey_when_locked() {
    let normal = MarkLook::of(Marker::Available(QuestKind::Normal));
    let daily = MarkLook::of(Marker::Ready(QuestKind::Daily));
    let artifact = MarkLook::of(Marker::Available(QuestKind::Artifact));
    let locked = MarkLook::of(Marker::Locked);
    assert_eq!(
        (normal.glyph(), daily.glyph(), locked.glyph()),
        ("!", "?", "!")
    );
    let colours = [normal.rgb, daily.rgb, artifact.rgb, locked.rgb];
    for (i, a) in colours.iter().enumerate() {
        for b in &colours[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    fn starts<M>(system: impl IntoSystem<(), (), M>) {
        IntoSystem::into_system(system).initialize(&mut World::new());
    }
    starts(read::read_server);
    starts(poi_marks::find_quest_pois);
    starts(over_view::show_marks);
}

#[test]
fn a_quest_under_way_shows_a_grey_question_mark() {
    let taken = MarkLook::of(Marker::Taken);
    assert!(taken.ready);
    assert_eq!(taken.rgb, MarkLook::of(Marker::Locked).rgb);
    assert_eq!(
        MarkLook::of_state(QuestKind::Daily, QuestState::Taken),
        taken
    );
    let ready = MarkLook::of_state(QuestKind::Daily, QuestState::Ready);
    assert_eq!(ready, MarkLook::of(Marker::Ready(QuestKind::Daily)));
    assert_eq!(ready.class(), "mark-blue");
    assert_eq!(taken.class(), "mark-grey");
}

#[test]
fn the_quest_followed_is_the_one_chosen_or_the_newest() {
    let mut older = entry(QuestState::Taken, vec![0, 0]);
    older.at = 10;
    let mut newer = entry(QuestState::Ready, vec![1, 1]);
    newer.id.slot = Slot::Giver {
        building: 4,
        slot: 1,
        chain: 0,
    };
    newer.at = 20;
    let mut book = book_of(older.clone());
    book.entries.insert(newer.id, newer.clone());
    assert_eq!(book.followed(), Some(newer.id));
    book.tracked = Some(older.id);
    assert_eq!(book.followed(), Some(older.id));
    book.entries.get_mut(&older.id).unwrap().state = QuestState::Done;
    assert_eq!(
        book.followed(),
        Some(newer.id),
        "only one under way is followed"
    );
}

#[test]
fn a_boss_dealt_with_marks_its_givers_town_until_the_quest_is_taken() {
    let id = QuestId {
        town: CRYPT,
        slot: Slot::Feeder,
    };
    let book = QuestBook {
        slain: vec![shared::protocol::SlainBoss {
            id,
            kind: QuestKind::Normal,
        }],
        ..QuestBook::default()
    };
    let marks = marks_of(&book);
    assert_eq!(
        marks.get(&CRYPT),
        Some(&MarkLook::new(QuestKind::Normal, false))
    );
    // A stranger's boss is red; one's own quest's is in its colour.
    let red = MarkLook::of_boss(shared::protocol::BossSign::Unknown);
    let own = MarkLook::of_boss(shared::protocol::BossSign::Quest(QuestKind::Daily));
    assert_eq!(red.glyph(), "!");
    assert_ne!(red.rgb, MarkLook::new(QuestKind::Normal, false).rgb);
    assert_eq!(own, MarkLook::new(QuestKind::Daily, false));
}
