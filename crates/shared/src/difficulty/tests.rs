use super::*;
use crate::vitals::REFILL_TICKS;

#[test]
fn names_read_back_in_any_case() {
    for difficulty in Difficulty::ALL {
        assert_eq!(Difficulty::from_name(difficulty.name()), Some(difficulty));
        let shouted = format!("  {}  ", difficulty.name().to_uppercase());
        assert_eq!(Difficulty::from_name(&shouted), Some(difficulty));
    }
    assert_eq!(Difficulty::from_name("Nightmare"), None);
}

#[test]
fn harm_is_a_fifth_less_or_more() {
    let harm = Points(10_000);
    assert_eq!(Difficulty::Easy.harm_taken(harm), Points(8_000));
    assert_eq!(Difficulty::Normal.harm_taken(harm), harm);
    assert_eq!(Difficulty::Hard.harm_taken(harm), Points(12_000));
    assert_eq!(Difficulty::Ironman.harm_taken(harm), Points(12_000));
    // No overflow at the top.
    assert_eq!(
        Difficulty::Hard.harm_taken(Points(u32::MAX)),
        Points(u32::MAX)
    );
}

#[test]
fn easy_refills_twice_as_fast() {
    assert_eq!(Difficulty::Easy.refill_ticks() * 2, REFILL_TICKS);
    for difficulty in [Difficulty::Normal, Difficulty::Hard, Difficulty::Ironman] {
        assert_eq!(difficulty.refill_ticks(), REFILL_TICKS);
    }
}

#[test]
fn ironman_is_never_changed_to_or_from() {
    use Difficulty::*;
    for from in Difficulty::ALL {
        for to in Difficulty::ALL {
            let allowed = from != Ironman && to != Ironman;
            assert_eq!(from.may_change_to(to), allowed, "{from:?} to {to:?}");
        }
    }
    assert_eq!(Ironman.for_existing(), Hard);
    assert_eq!(Easy.for_existing(), Easy);
}

#[test]
fn parsing_lists() {
    use Difficulty::*;
    assert_eq!(Difficulties::parse(""), (Difficulties::ALL, vec![]));
    assert_eq!(Difficulties::parse(" , "), (Difficulties::ALL, vec![]));
    let (allowed, unknown) = Difficulties::parse("easy, HARD");
    assert_eq!(allowed, Difficulties::only(Easy).with(Hard));
    assert!(unknown.is_empty());
    let (allowed, unknown) = Difficulties::parse("normal,nightmare");
    assert_eq!(allowed, Difficulties::only(Normal));
    assert_eq!(unknown, vec!["nightmare".to_owned()]);
    let (allowed, unknown) = Difficulties::parse("nightmare, hell");
    assert_eq!(allowed, Difficulties::ALL);
    assert_eq!(unknown.len(), 2);
    assert_eq!(Difficulties::parse(&allowed.names()).0, allowed);
}

#[test]
fn the_last_is_never_taken_away() {
    use Difficulty::*;
    let one = Difficulties::only(Hard);
    assert_eq!(one.without(Hard), one);
    assert_eq!(one.with(Easy).without(Hard), Difficulties::only(Easy));
    assert_eq!(Difficulties::ALL.len(), 4);
}

#[test]
fn closest_prefers_the_easier_on_a_tie() {
    use Difficulty::*;
    let ends = Difficulties::only(Easy).with(Ironman);
    assert_eq!(ends.closest(Easy), Easy);
    assert_eq!(ends.closest(Normal), Easy);
    assert_eq!(ends.closest(Hard), Ironman);
    let middle = Difficulties::only(Normal).with(Hard);
    assert_eq!(middle.closest(Ironman), Hard);
    assert_eq!(middle.closest(Easy), Normal);
    let gap = Difficulties::only(Easy).with(Hard);
    assert_eq!(gap.closest(Normal), Easy);
    assert_eq!(Difficulties::only(Ironman).closest(Easy), Ironman);
}
