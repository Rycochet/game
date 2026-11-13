use super::*;
use crate::quests::tests::entry;
use shared::poi::ChunkPos;

#[test]
fn the_world_map_marks_the_steps_that_can_be_done_now() {
    let cave = ChunkPos { x: 2, y: 1 };
    let crypt = ChunkPos { x: -1, y: 3 };
    assert_eq!(world_chunks(&entry(QuestState::Taken, vec![0, 0])), [cave]);
    assert_eq!(world_chunks(&entry(QuestState::Taken, vec![1, 0])), [crypt]);
}

#[test]
fn handing_in_marks_the_givers_town() {
    let ready = entry(QuestState::Ready, vec![1, 1]);
    assert_eq!(world_chunks(&ready), [ready.id.town]);
    assert!(world_chunks(&entry(QuestState::Done, vec![1, 1])).is_empty());
}

#[test]
fn a_level_shows_what_the_server_sent_for_the_quest_followed() {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<GameView>()
        .init_resource::<QuestBook>()
        .init_resource::<LevelDots>()
        .init_resource::<TrackedDots>()
        .add_systems(Update, find_tracked_dots);
    let quest = entry(QuestState::Taken, vec![0, 0]);
    let id = quest.id;
    app.world_mut()
        .resource_mut::<QuestBook>()
        .entries
        .insert(id, quest);
    app.world_mut()
        .resource_mut::<LevelDots>()
        .0
        .insert(id, vec![(3, 4)]);
    app.world_mut()
        .resource_mut::<NextState<GameView>>()
        .set(GameView::Dungeon);
    app.update();
    app.update();
    let dots = app.world().resource::<TrackedDots>();
    assert_eq!(dots.places, [(3, 4)]);
    assert_eq!(dots.rgb, crate::common::QUEST_BLUE);
}
