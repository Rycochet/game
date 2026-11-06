use shared::id::Id;
use shared::sheet::Feeling;

use super::choose::{Seen, cycle, enemy_beside, in_order, nearest_enemy};

fn seen(number: u128, at: (i64, i64), feeling: Feeling) -> Seen {
    Seen {
        id: Id::for_test(number),
        at,
        feeling,
    }
}

#[test]
fn tab_goes_through_enemies_nearest_first() {
    let all = [
        seen(1, (5, 0), Feeling::Unfriendly),
        seen(2, (1, 0), Feeling::Friendly),
        seen(3, (2, 0), Feeling::Unfriendly),
        seen(4, (2, 0), Feeling::Unfriendly),
    ];
    let ids = |numbers: &[u128]| numbers.iter().map(|&n| Id::for_test(n)).collect::<Vec<_>>();
    assert_eq!(in_order((0, 0), &all), ids(&[3, 4, 1]));
}

#[test]
fn with_no_enemies_tab_goes_through_everyone() {
    let all = [
        seen(1, (3, 3), Feeling::Neutral),
        seen(2, (1, 0), Feeling::None),
    ];
    assert_eq!(
        in_order((0, 0), &all),
        vec![Id::for_test(2), Id::for_test(1)]
    );
}

#[test]
fn cycling_wraps_both_ways() {
    let order: Vec<Id> = (1..=3).map(Id::for_test).collect();
    let id = |n| Some(Id::for_test(n));
    assert_eq!(cycle(&order, None, true), id(1));
    assert_eq!(cycle(&order, None, false), id(3));
    assert_eq!(cycle(&order, id(1), true), id(2));
    assert_eq!(cycle(&order, id(3), true), id(1));
    assert_eq!(cycle(&order, id(1), false), id(3));
    assert_eq!(cycle(&order, id(2), false), id(1));
    // A target out of sight starts again from the nearest.
    assert_eq!(cycle(&order, id(9), true), id(1));
    assert_eq!(cycle(&[], id(1), true), None);
}

#[test]
fn after_a_death_the_nearest_other_enemy_is_chosen() {
    let all = [
        seen(1, (1, 0), Feeling::Unfriendly),
        seen(2, (4, 0), Feeling::Unfriendly),
        seen(3, (2, 0), Feeling::Friendly),
    ];
    assert_eq!(
        nearest_enemy((0, 0), &all, Id::for_test(1)),
        Some(Id::for_test(2))
    );
    assert_eq!(nearest_enemy((0, 0), &all[..1], Id::for_test(1)), None);
}

#[test]
fn only_an_enemy_right_beside_counts() {
    let far = seen(1, (2, 0), Feeling::Unfriendly);
    let friend = seen(2, (1, 1), Feeling::Friendly);
    let enemy = seen(3, (-1, 1), Feeling::Unfriendly);
    assert_eq!(enemy_beside((0, 0), &[far, friend]), None);
    assert_eq!(
        enemy_beside((0, 0), &[far, friend, enemy]),
        Some(Id::for_test(3))
    );
}
