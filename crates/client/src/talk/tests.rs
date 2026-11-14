use super::*;
use crate::common::MapPosition;

#[test]
fn beside_a_door_includes_diagonals_and_nothing_further() {
    let at = |x, y| MapPosition { x, y };
    assert!(beside(at(1, 1), (0, 0)));
    assert!(beside(at(0, 0), (0, 0)));
    assert!(!beside(at(2, 0), (0, 0)));
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    fn starts<M>(system: impl IntoSystem<(), (), M>) {
        IntoSystem::into_system(system).initialize(&mut World::new());
    }
    starts(change_of_mind);
    starts(follow_clicked_door);
    starts(open_on_arrival);
    starts(note_gifts);
    starts(read_server);
    starts(draw_grids);
    starts(show_gift_aim);
    starts(close_on_escape);
    starts(close_when_left);
    starts(news::read_quest_news);
    starts(input::read_events);
    starts(quests_page::show_pages);
}

#[test]
fn the_panel_is_good_markup_with_the_grids_columns() {
    markup::TALK.assert_good();
    let columns = format!("columns=\"{COLUMNS}\"");
    assert!(markup::TALK.source.contains(&columns));
}

#[test]
fn new_hints_and_good_words_are_worded_in_full() {
    use shared::celestial::ColourName;
    use shared::creature::CreatureKind;
    use shared::gender::Gender;
    use shared::lang::{Lang, Language};
    use shared::protocol::{Heard, Hint};
    use shared::site::BuildingKind;
    use shared::story::Person;

    let lang = Lang::new(Language::English);
    let giver = Person {
        kind: CreatureKind::Human,
        gender: Gender::Female,
        given: "Mara".to_owned(),
        family: Some("Tull".to_owned()),
    };
    let said = [
        words::hint_text(
            &lang,
            &Hint::Neighbour {
                giver: giver.clone(),
                kind: BuildingKind::Blacksmith,
            },
        ),
        words::hint_text(
            &lang,
            &Hint::Neighbour {
                giver: giver.clone(),
                kind: BuildingKind::House,
            },
        ),
        words::hint_text(
            &lang,
            &Hint::Moon {
                colour: Some(ColourName::Blue),
            },
        ),
        words::hint_text(&lang, &Hint::Moon { colour: None }),
        words::heard_text(
            &lang,
            &Heard::Neighbour {
                giver,
                kind: BuildingKind::Blacksmith,
            },
        ),
        words::heard_text(
            &lang,
            &Heard::Town {
                name: Some("Little Fen".to_owned()),
            },
        ),
        words::heard_text(&lang, &Heard::Town { name: None }),
    ];
    assert!(said[0].contains("Mara Tull, the blacksmith"), "{}", said[0]);
    assert!(said[1].contains("Mara Tull first"), "{}", said[1]);
    assert!(said[2].contains("blue moon"), "{}", said[2]);
    assert!(said[4].contains("helped Mara Tull"), "{}", said[4]);
    assert!(said[5].contains("Little Fen"), "{}", said[5]);
    for words in said {
        assert!(!words.contains('{') && !words.is_empty(), "{words}");
    }
}
