use bevy::prelude::*;

use super::*;

fn key(key: KeyCode) -> Binding {
    Binding::new(key, Mods::NONE)
}

fn shift(key: KeyCode) -> Binding {
    Binding::new(key, Mods::SHIFT)
}

#[test]
fn no_two_actions_share_a_default() {
    let controls = Controls::default();
    let all: Vec<Binding> = controls.all().map(|(_, binding)| binding).collect();
    for (at, binding) in all.iter().enumerate() {
        assert!(!all[at + 1..].contains(binding), "{binding:?} twice");
        assert!(Binding::can_bind(binding.key), "{binding:?} can't be bound");
    }
}

#[test]
fn a_key_given_to_another_action_leaves_the_first() {
    let mut controls = Controls::default();
    let lost = controls.set(Action::StepUp, 1, Some(key(KeyCode::KeyM)));
    assert_eq!(lost, vec![(Action::FullMap, 0)]);
    assert_eq!(controls.keys(Action::FullMap), [None, None]);
    assert_eq!(controls.keys(Action::StepUp)[1], Some(key(KeyCode::KeyM)));
}

#[test]
fn a_key_moved_within_an_action_leaves_its_other_slot() {
    let mut controls = Controls::default();
    let lost = controls.set(Action::StepUp, 1, Some(key(KeyCode::KeyW)));
    assert_eq!(lost, vec![(Action::StepUp, 0)]);
    assert_eq!(
        controls.keys(Action::StepUp),
        [None, Some(key(KeyCode::KeyW))]
    );
}

#[test]
fn a_combination_and_its_key_are_different_bindings() {
    let mut controls = Controls::default();
    let lost = controls.set(Action::FullMap, 1, Some(shift(KeyCode::KeyM)));
    assert!(lost.is_empty());
}

#[test]
fn names_are_saved_and_read_back() {
    for binding in [
        key(KeyCode::KeyG),
        shift(KeyCode::KeyG),
        Binding::new(KeyCode::KeyG, Mods::CTRL.with(Mods::ALT).with(Mods::SUPER)),
        key(KeyCode::ShiftRight),
        Binding::new(KeyCode::ControlLeft, Mods::SHIFT),
        key(KeyCode::Backquote),
    ] {
        assert_eq!(Binding::from_name(&binding.to_name()), Some(binding));
    }
    assert_eq!(shift(KeyCode::KeyG).to_name(), "Shift+KeyG");
    assert_eq!(key(KeyCode::ShiftRight).to_name(), "Shift");
    assert_eq!(Binding::from_name("Shift+Nonsense"), None);
    assert_eq!(Binding::from_name("Escape"), None);
}

#[test]
fn labels_read_as_the_keys() {
    assert_eq!(shift(KeyCode::KeyG).label(), "Shift + G");
    assert_eq!(key(KeyCode::ArrowUp).label(), "Up Arrow");
    assert_eq!(key(KeyCode::ShiftRight).label(), "Shift");
}

#[test]
fn a_modifier_alone_never_needs_itself() {
    let binding = Binding::new(KeyCode::ShiftRight, Mods::SHIFT);
    assert_eq!(binding, key(KeyCode::ShiftLeft));
}

#[test]
fn only_changes_are_saved_and_defaults_fill_the_rest() {
    let mut controls = Controls::default();
    controls.set(Action::StepUp, 1, Some(shift(KeyCode::KeyM)));
    let text = ron::to_string(&controls).expect("saved");
    assert_eq!(text, r#"{"step_up":(Some("KeyW"),Some("Shift+KeyM"))}"#);
    let back: Controls = ron::from_str(&text).expect("read");
    assert_eq!(back, controls);
    assert_eq!(ron::to_string(&Controls::default()).expect("saved"), "{}");
}

#[test]
fn a_default_the_player_gave_away_stays_away() {
    // As if Full Map were new: its M was already given to Step Up.
    let back: Controls = ron::from_str(r#"{"step_up":(Some("KeyM"),None)}"#).expect("read");
    assert_eq!(back.keys(Action::FullMap), [None, None]);
}

#[test]
fn unknown_names_are_left_out() {
    let back: Controls =
        ron::from_str(r#"{"gone":(Some("KeyM"),None),"step_up":(Some("Nonsense"),Some("KeyQ"))}"#)
            .expect("read");
    assert_eq!(back.keys(Action::StepUp), [None, Some(key(KeyCode::KeyQ))]);
}

fn input(down: &[KeyCode]) -> ButtonInput<KeyCode> {
    let mut keys = ButtonInput::default();
    for &each in down {
        keys.press(each);
    }
    keys
}

#[test]
fn the_most_particular_binding_wins() {
    let mut controls = Controls::default();
    controls.set(Action::TurnItem, 1, Some(shift(KeyCode::KeyM)));
    let shifted = input(&[KeyCode::ShiftRight, KeyCode::KeyM]);
    assert!(controls.pressed(&shifted, Action::TurnItem));
    assert!(!controls.pressed(&shifted, Action::FullMap));
    let plain = input(&[KeyCode::KeyM]);
    assert!(controls.pressed(&plain, Action::FullMap));
    assert!(!controls.pressed(&plain, Action::TurnItem));
}

#[test]
fn plain_keys_work_whatever_else_is_held() {
    let controls = Controls::default();
    let keys = input(&[KeyCode::ShiftLeft, KeyCode::KeyW]);
    assert!(controls.pressed(&keys, Action::StepUp));
}

#[test]
fn a_modifier_alone_counts_while_down() {
    let mut controls = Controls::default();
    controls.set(Action::TurnItem, 0, Some(key(KeyCode::ShiftLeft)));
    assert!(controls.just_pressed(&input(&[KeyCode::ShiftRight]), Action::TurnItem));
}

#[test]
fn events_match_with_what_is_held() {
    let controls = Controls::default();
    assert!(controls.is_key_of(Action::FullScreen, KeyCode::Enter, Mods::ALT));
    assert!(!controls.is_key_of(Action::FullScreen, KeyCode::Enter, Mods::NONE));
}

#[test]
fn number_pad_defaults_are_in_the_second_column() {
    let numpad = |binding: &Option<Binding>| {
        binding.is_some_and(|binding| format!("{:?}", binding.key).starts_with("Numpad"))
    };
    for action in Action::ALL {
        let [first, _] = action.defaults();
        assert!(!numpad(&first), "{action:?} has the number pad first");
    }
    for (action, key) in [
        (Action::StepUp, KeyCode::Numpad8),
        (Action::StepDown, KeyCode::Numpad2),
        (Action::StepLeft, KeyCode::Numpad4),
        (Action::StepRight, KeyCode::Numpad6),
        (Action::TurnLeft, KeyCode::Numpad7),
        (Action::TurnRight, KeyCode::Numpad9),
        (Action::ResetView, KeyCode::Numpad5),
        (Action::ZoomIn, KeyCode::NumpadAdd),
        (Action::ZoomOut, KeyCode::NumpadSubtract),
        (Action::Explore, KeyCode::Numpad0),
        (Action::Leave, KeyCode::NumpadDecimal),
    ] {
        assert_eq!(action.defaults()[1], Some(Binding::new(key, Mods::NONE)));
    }
}

#[test]
fn releasing_lets_go_of_held_keys() {
    let controls = Controls::default();
    let mut keys = input(&[KeyCode::Numpad7]);
    keys.clear();
    assert!(controls.pressed(&keys, Action::TurnLeft));
    controls.release(&mut keys, Action::TurnLeft);
    assert!(!controls.active(&keys, Action::TurnLeft));
}

#[test]
fn clearing_takes_the_press() {
    let controls = Controls::default();
    let mut keys = input(&[KeyCode::KeyC]);
    assert!(controls.clear_just_pressed(&mut keys, Action::Character));
    assert!(!controls.just_pressed(&keys, Action::Character));
}

#[test]
fn short_labels_fit_a_slot_corner() {
    let ctrl = Binding::new(KeyCode::Digit1, Mods::CTRL);
    let shift_alt = Binding::new(KeyCode::F12, Mods::SHIFT.with(Mods::ALT));
    assert_eq!(key(KeyCode::Digit1).short_label(), "1");
    assert_eq!(key(KeyCode::F12).short_label(), "F12");
    assert_eq!(shift(KeyCode::Digit1).short_label(), "S1");
    assert_eq!(ctrl.short_label(), "^1");
    assert_eq!(key(KeyCode::Numpad1).short_label(), "N1");
    assert_eq!(key(KeyCode::ArrowUp).short_label(), "UA");
    assert_eq!(shift_alt.short_label(), "SAF");
    for action in Action::ALL {
        for binding in action.defaults().into_iter().flatten() {
            assert!(binding.short_label().chars().count() <= 3);
        }
    }
}

#[test]
fn every_action_has_its_own_place_and_the_bar_is_one_to_zero() {
    for (index, action) in Action::ALL.into_iter().enumerate() {
        assert_eq!(action.index(), index);
        assert_eq!(Action::from_id(action.id()), Some(action));
    }
    assert_eq!(Action::UseSlot(0).defaults()[0], Some(key(KeyCode::Digit1)));
    assert_eq!(Action::UseSlot(9).defaults()[0], Some(key(KeyCode::Digit0)));
}
