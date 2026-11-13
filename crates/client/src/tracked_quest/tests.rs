use super::lines::{objectives, return_line};
use crate::quests::tests::entry;
use bevy::prelude::*;
use shared::lang::Lang;
use shared::protocol::QuestState;

fn english() -> Lang {
    crate::common::Wording::default().0
}

#[test]
fn only_steps_that_can_be_done_now_are_listed() {
    let lang = english();
    let first = objectives(&entry(QuestState::Taken, vec![0, 0]), &lang);
    assert_eq!(first.len(), 1);
    let second = objectives(&entry(QuestState::Taken, vec![1, 0]), &lang);
    assert_eq!(second.len(), 1);
    assert_ne!(first, second);
    assert!(return_line(&entry(QuestState::Taken, vec![1, 0]), &lang).is_none());
}

#[test]
fn once_every_step_is_done_only_returning_is_left() {
    let lang = english();
    let ready = entry(QuestState::Ready, vec![1, 1]);
    assert!(objectives(&ready, &lang).is_empty());
    let words = return_line(&ready, &lang).unwrap();
    assert!(!words.contains('{'), "{words}");
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    fn starts<M>(system: impl IntoSystem<(), (), M>) {
        IntoSystem::into_system(system).initialize(&mut World::new());
    }
    starts(super::show::show_tracked);
    starts(super::markup::bind_box);
    starts(super::markup::spawn_box);
}
