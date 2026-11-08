use super::*;
use shared::difficulty::Difficulties;

fn playing(current: Difficulty, allowed: Difficulties) -> PlayDifficulty {
    PlayDifficulty {
        current: Some(current),
        allowed,
    }
}

#[test]
fn before_playing_every_difficulty_is_the_players_own() {
    let shown = choices(PlayDifficulty::default(), Difficulty::Hard);
    assert_eq!(shown.options, Difficulty::ALL.to_vec());
    assert_eq!(shown.selected, Difficulty::Hard);
    assert_eq!(shown.why_not, None);
}

#[test]
fn while_playing_only_allowed_ones_and_never_ironman() {
    let shown = choices(
        playing(Difficulty::Normal, Difficulties::ALL),
        Difficulty::Easy,
    );
    assert_eq!(
        shown.options,
        vec![Difficulty::Easy, Difficulty::Normal, Difficulty::Hard]
    );
    assert_eq!(shown.selected, Difficulty::Normal);
    let some = Difficulties::only(Difficulty::Normal).with(Difficulty::Hard);
    let shown = choices(playing(Difficulty::Hard, some), Difficulty::Easy);
    assert_eq!(shown.options, vec![Difficulty::Normal, Difficulty::Hard]);
}

#[test]
fn nothing_to_change_to_is_disabled() {
    let one = Difficulties::only(Difficulty::Normal);
    let shown = choices(playing(Difficulty::Normal, one), Difficulty::Easy);
    assert_eq!(shown.options, vec![Difficulty::Normal]);
    assert!(shown.why_not.is_some());
    // Ironman is allowed, but not to change to.
    let with_ironman = one.with(Difficulty::Ironman);
    let shown = choices(playing(Difficulty::Normal, with_ironman), Difficulty::Easy);
    assert!(shown.why_not.is_some());
    let shown = choices(
        playing(Difficulty::Ironman, Difficulties::ALL),
        Difficulty::Easy,
    );
    assert_eq!(shown.options, vec![Difficulty::Ironman]);
    assert!(shown.why_not.is_some());
}
