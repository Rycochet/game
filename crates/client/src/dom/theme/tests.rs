use super::*;

#[test]
fn a_class_picks_a_look_and_falls_back_to_the_plain_one() {
    let theme = Theme::parchment();
    let plain = theme.look(Tag::P, None);
    assert_eq!(theme.look(Tag::P, Some("no such class")), plain);
    let warning = theme.look(Tag::P, Some("warning"));
    assert_eq!(warning.colour, Some(WARNING_TEXT));
    assert_eq!(warning.margin, plain.margin);
    assert!(theme.has(Tag::P, "warning"));
    assert!(!theme.has(Tag::Div, "warning"));
}

#[test]
fn layout_elements_only_space_what_is_in_them() {
    let theme = Theme::parchment();
    let spaced = Look {
        gap: STACK,
        ..default()
    };
    assert_eq!(theme.look(Tag::Div, None), spaced);
    assert_eq!(theme.look(Tag::Form, None), spaced);
    assert_eq!(theme.look(Tag::Div, Some("sections")).gap, 24.0);
    // Nothing has room of its own round it.
    let no_margin =
        |looks: &Vec<(&str, Look)>| looks.iter().all(|(_, look)| look.margin == [0.0; 4]);
    assert!(theme.looks.values().all(no_margin));
}

#[test]
fn spacing_is_in_steps_of_four() {
    let theme = Theme::parchment();
    for looks in theme.looks.values() {
        for (_, look) in looks {
            assert!(look.margin.iter().all(|side| side % 4.0 == 0.0));
            assert_eq!(look.gap % 4.0, 0.0);
        }
    }
}

#[test]
fn setting_a_look_again_replaces_it() {
    let mut theme = Theme::parchment();
    let look = Look {
        gap: 8.0,
        ..default()
    };
    theme.set(Tag::P, "", look);
    assert_eq!(theme.look(Tag::P, None), look);
}
