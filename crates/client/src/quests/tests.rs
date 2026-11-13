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
fn entry(state: QuestState, counts: Vec<u32>) -> LogEntry {
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
