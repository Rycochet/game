use super::capture::Outcome;
use super::show::words;
use super::*;
use crate::common::Mods;

fn waiting() -> KeyWaiting {
    let mut waiting = KeyWaiting::default();
    waiting.start(Entity::PLACEHOLDER, Mods::NONE);
    waiting
}

fn chosen(key: KeyCode, mods: Mods) -> Option<Outcome> {
    Some(Outcome::Chosen(Some(Binding::new(key, mods))))
}

#[test]
fn a_key_is_taken_at_once() {
    assert_eq!(
        waiting().step(KeyCode::KeyG, true),
        chosen(KeyCode::KeyG, Mods::NONE)
    );
}

#[test]
fn a_key_with_a_modifier_held_is_that_combination() {
    let mut waiting = waiting();
    assert_eq!(waiting.step(KeyCode::ShiftRight, true), None);
    assert_eq!(
        waiting.step(KeyCode::KeyG, true),
        chosen(KeyCode::KeyG, Mods::SHIFT)
    );
}

#[test]
fn a_modifier_let_go_alone_is_itself() {
    let mut waiting = waiting();
    waiting.step(KeyCode::ControlLeft, true);
    waiting.step(KeyCode::ShiftLeft, true);
    assert_eq!(
        waiting.step(KeyCode::ShiftLeft, false),
        chosen(KeyCode::ShiftLeft, Mods::CTRL)
    );
}

#[test]
fn letting_go_of_an_earlier_modifier_binds_nothing() {
    let mut waiting = waiting();
    waiting.step(KeyCode::ControlLeft, true);
    waiting.step(KeyCode::ShiftLeft, true);
    assert_eq!(waiting.step(KeyCode::ControlLeft, false), None);
}

#[test]
fn escape_cancels_and_backspace_empties() {
    let mut held = waiting();
    held.step(KeyCode::ShiftLeft, true);
    assert_eq!(held.step(KeyCode::Escape, true), Some(Outcome::Cancelled));
    assert_eq!(
        waiting().step(KeyCode::Backspace, true),
        Some(Outcome::Chosen(None))
    );
}

#[test]
fn a_box_says_what_it_waits_for() {
    let key_box = KeyBox {
        binding: Some(Binding::new(KeyCode::KeyG, Mods::SHIFT)),
        warned: false,
    };
    assert_eq!(words(&key_box, None), "Shift + G");
    let mut waiting = waiting();
    assert_eq!(words(&key_box, Some(&waiting)), "Press key");
    waiting.step(KeyCode::ShiftLeft, true);
    assert_eq!(words(&key_box, Some(&waiting)), "Shift + …");
    assert_eq!(words(&KeyBox::default(), None), "");
}
