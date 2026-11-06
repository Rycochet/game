use super::*;
use crate::rng::Seed;

fn turned(turn: u16) -> Build {
    Build {
        turn,
        ..Build::CENTRED
    }
}

#[test]
fn the_cosine_table_is_whole_round_the_circle() {
    assert_eq!(cosine(0), 1000);
    assert_eq!(cosine(60), 500);
    assert_eq!(cosine(90), 0);
    assert_eq!(cosine(120), -500);
    assert_eq!(cosine(180), -1000);
    assert_eq!(cosine(240), -500);
    assert_eq!(cosine(300), 500);
    assert_eq!(cosine(360), 1000);
}

#[test]
fn the_top_row_gets_the_extra_point() {
    // Body at the top: 2, 1, 1.
    assert_eq!(turned(0).points(), [2000, 1000, 1000]);
    // Mind and Spirit each a third of a turn on.
    assert_eq!(turned(240).points(), [1000, 2000, 1000]);
    assert_eq!(turned(120).points(), [1000, 1000, 2000]);
}

#[test]
fn two_rows_equally_high_share_it() {
    // Body and Spirit either side of the top, Mind at the bottom.
    assert_eq!(turned(60).points(), [1500, 1000, 1500]);
}

#[test]
fn every_turn_shares_out_four_points_and_never_more_than_two_rows_gain() {
    for turn in 0..TURN_STEPS {
        let points = turned(turn).points();
        assert_eq!(points.iter().sum::<u32>(), POINTS_TOTAL, "{turn}");
        assert!(
            points.iter().all(|&row| (1000..=2000).contains(&row)),
            "{turn}"
        );
        assert!(
            points.iter().filter(|&&row| row > 1000).count() <= 2,
            "{turn}"
        );
    }
}

#[test]
fn only_dots_inside_and_a_turn_within_one_turn_are_valid() {
    assert!(Build::CENTRED.is_valid());
    assert!(Dot([1000, 0, 0]).is_valid());
    assert!(!Dot([500, 500, 1]).is_valid());
    assert!(!Dot([0, 0, 0]).is_valid());
    assert!(!turned(TURN_STEPS).is_valid());
    let outside = Build {
        dots: [Dot::CENTRE, Dot([1000, 1000, 0]), Dot::CENTRE],
        turn: 0,
    };
    assert!(!outside.is_valid());
}

#[test]
fn rolled_builds_are_valid_near_the_centre_and_the_same_for_a_seed() {
    for n in 0..200 {
        let roll = || Build::rolled(&mut Seed::new(n).rng());
        let build = roll();
        assert!(build.is_valid(), "{build:?}");
        assert_eq!(build, roll());
        for dot in build.dots {
            assert!(
                dot.0.iter().all(|&weight| (200..=470).contains(&weight)),
                "{dot:?}"
            );
        }
    }
}
