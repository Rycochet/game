use shared::creature::CreatureKind;

use super::*;

fn person() -> Layout {
    Layout::of(CreatureKind::Human).expect("a human is drawn")
}

fn all_hair() -> impl Iterator<Item = Hair> {
    use HairStyle::*;
    let styles = [Bald, Short, Long, TiedBack, Braids, Topknot, Crest, Curly];
    let beards = [
        Beard::None,
        Beard::Short,
        Beard::Full,
        Beard::Long,
        Beard::Braided,
        Beard::Forked,
    ];
    styles
        .into_iter()
        .flat_map(move |style| beards.map(|beard| Hair { style, beard }))
}

#[test]
fn every_style_but_none_draws_something() {
    let layout = person();
    for styled in all_hair() {
        let drawn = hair(&layout, styled, Cover::default());
        assert_eq!(drawn.is_empty(), styled == Hair::NONE, "{styled:?}");
        assert!(drawn.iter().flatten().all(|corner| corner.is_finite()));
    }
}

#[test]
fn nothing_shows_above_a_rim() {
    let layout = person();
    let skull = Skull::of(&layout);
    // A helmet down to the middle of the head all round.
    let rim = skull.middle.y;
    let cover = Cover {
        rim: Some([rim; 16]),
        ..default()
    };
    for styled in all_hair() {
        for triangle in hair(&layout, styled, cover) {
            let lowest = triangle
                .iter()
                .map(|corner| corner.y)
                .fold(f32::MAX, f32::min);
            assert!(lowest < rim, "{styled:?}");
        }
    }
}

#[test]
fn a_knot_or_crest_goes_under_a_helmet() {
    let layout = person();
    let skull = Skull::of(&layout);
    let cover = Cover {
        rim: Some([skull.middle.y; 16]),
        ..default()
    };
    for style in [HairStyle::Topknot, HairStyle::Crest] {
        let styled = Hair {
            style,
            beard: Beard::None,
        };
        let top = |cover| {
            hair(&layout, styled, cover)
                .iter()
                .flatten()
                .map(|corner| corner.y)
                .fold(f32::MIN, f32::max)
        };
        assert!(
            top(Cover::default()) > skull.middle.y + skull.up,
            "{style:?}"
        );
        assert!(top(cover) < skull.middle.y + skull.up * 0.5, "{style:?}");
    }
}

#[test]
fn an_aventail_hides_it_and_a_bevor_the_beard() {
    let layout = person();
    let styled = Hair {
        style: HairStyle::Long,
        beard: Beard::Long,
    };
    let hidden = Cover {
        hidden: true,
        ..default()
    };
    assert!(hair(&layout, styled, hidden).is_empty());
    let bevor = Cover {
        chin_covered: true,
        ..default()
    };
    let no_beard = Hair {
        beard: Beard::None,
        ..styled
    };
    assert_eq!(
        hair(&layout, styled, bevor).len(),
        hair(&layout, no_beard, Cover::default()).len()
    );
}

#[test]
fn rim_heights_go_round_from_the_front_to_the_left() {
    let mut rim = [0.0; 16];
    rim[4] = 100.0;
    let cover = Cover {
        rim: Some(rim),
        ..default()
    };
    // A quarter turn round is the left side.
    assert_eq!(cover.rim_at(std::f32::consts::FRAC_PI_2), 100.0);
    assert_eq!(cover.rim_at(0.0), 0.0);
    assert_eq!(Cover::default().rim_at(1.0), f32::MAX);
}
