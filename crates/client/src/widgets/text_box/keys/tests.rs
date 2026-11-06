use super::*;

fn key(key_code: KeyCode, logical_key: Key, state: KeyState) -> KeyboardInput {
    KeyboardInput {
        key_code,
        logical_key,
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    }
}

fn down(key_code: KeyCode, logical_key: Key) -> KeyboardInput {
    key(key_code, logical_key, KeyState::Pressed)
}

/// Presses these, following the modifiers among them, as `take_text` does.
fn type_into(edited: &mut Edited, keys: &[KeyboardInput]) {
    let mut held = Held::default();
    for key in keys {
        held.follow(key);
        if key.state == KeyState::Pressed {
            press(key, &held, edited, false);
        }
    }
}

#[test]
fn arrows_with_shift_select_and_typing_replaces() {
    let mut edited = Edited::at_end("hello", false);
    type_into(
        &mut edited,
        &[
            down(KeyCode::ShiftLeft, Key::Shift),
            down(KeyCode::ArrowLeft, Key::ArrowLeft),
            down(KeyCode::ArrowLeft, Key::ArrowLeft),
            key(KeyCode::ShiftLeft, Key::Shift, KeyState::Released),
            down(KeyCode::KeyP, Key::Character("p".into())),
        ],
    );
    assert_eq!(edited.text, "help");
    assert_eq!(edited.caret, 4);
}

#[test]
fn ctrl_moves_by_words_and_home_end_go_to_the_ends() {
    let mut edited = Edited::at_end("one two", false);
    type_into(
        &mut edited,
        &[
            down(KeyCode::ControlLeft, Key::Control),
            down(KeyCode::ArrowLeft, Key::ArrowLeft),
        ],
    );
    assert_eq!(edited.caret, 4);
    type_into(&mut edited, &[down(KeyCode::Home, Key::Home)]);
    assert_eq!(edited.caret, 0);
    type_into(&mut edited, &[down(KeyCode::End, Key::End)]);
    assert_eq!(edited.caret, 7);
}

#[test]
fn ctrl_a_selects_all_and_other_shortcuts_type_nothing() {
    let mut edited = Edited::at_end("abc", false);
    type_into(
        &mut edited,
        &[
            down(KeyCode::ControlLeft, Key::Control),
            down(KeyCode::KeyQ, Key::Character("q".into())),
            down(KeyCode::KeyA, Key::Character("a".into())),
        ],
    );
    assert_eq!(edited.text, "abc");
    assert_eq!(edited.selection(), Some((0, 3)));
}

#[test]
fn an_owner_changing_the_text_puts_the_caret_at_the_end() {
    let text_box = TextBox {
        text: "new words".into(),
        ..default()
    };
    let cursor = Cursor {
        caret: 1,
        anchor: 2,
        text: "old".into(),
        ..default()
    };
    assert_eq!(
        current(&text_box, &cursor),
        Edited::at_end("new words", false)
    );
}

#[test]
fn keeping_an_edit_says_whether_the_writing_changed() {
    let mut text_box = TextBox {
        text: "ab".into(),
        ..default()
    };
    let mut cursor = Cursor {
        caret: 2,
        anchor: 2,
        text: "ab".into(),
        ..default()
    };
    let mut edited = current(&text_box, &cursor);
    edited.move_caret(Motion::Left, false);
    assert!(!keep(edited.clone(), &mut text_box, &mut cursor, 5.0));
    assert_eq!((cursor.caret, cursor.moved_at), (1, 5.0));
    edited.insert("x");
    assert!(keep(edited, &mut text_box, &mut cursor, 6.0));
    assert_eq!(text_box.text, "axb");
}
