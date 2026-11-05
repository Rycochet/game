use super::table::ROWS;
use super::*;
use crate::body::Frame;
use crate::creature::{CreatureKind, Size};
use crate::item::{Quality, Shape};
use crate::rng::Seed;

const KINDS: [HeadKind; 4] = [
    HeadKind::Cap,
    HeadKind::Coif,
    HeadKind::Helmet,
    HeadKind::BevoredHelm,
];

fn rolled(kind: HeadKind, style: Option<Style>) -> impl Iterator<Item = HeadForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("head").rng();
        roll(&mut rng, kind, style, Quality::Rare)
    })
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for kind in KINDS {
        for form in rolled(kind, None) {
            assert!(form.allowed(), "{form:?}");
            assert_eq!(form.kind, kind);
        }
    }
    for style in Style::ALL {
        for form in rolled(HeadKind::Helmet, Some(style)).take(300) {
            assert!(form.allowed(), "{form:?}");
            assert_eq!(form.style, style);
            assert!(style.allows(form.construction), "{form:?}");
        }
    }
}

#[test]
fn every_name_turns_up() {
    let mut names = Vec::new();
    for kind in KINDS {
        names.extend(rolled(kind, None).map(|form| form.name()));
    }
    for style in [Style::Classical, Style::IndoPersian, Style::Japanese] {
        names.extend(rolled(HeadKind::Helmet, Some(style)).map(|form| form.name()));
    }
    let named = ROWS.iter().filter_map(|row| row.name);
    let generic = [
        "Arming Cap",
        "Coif",
        "Mail Coif",
        "Ring Mail Coif",
        "Leather Helmet",
        "Leather Cap",
        "Rattan Cap",
        "Rattan War Hat",
        "Skull Helm",
    ];
    for name in named.chain(generic) {
        assert!(names.iter().any(|have| have == name), "{name}");
    }
    assert!(names.iter().any(|name| name.starts_with("Horned ")));
    assert!(names.iter().any(|name| name.starts_with("Winged ")));
}

#[test]
fn closed_faces_are_helms_and_brims_war_hats() {
    for form in rolled(HeadKind::Helmet, None).filter(|form| form.specific_name().is_none()) {
        let name = form.generic_name();
        assert_eq!(form.brim.round(), name == "War Hat", "{form:?}");
    }
    let cap = rolled(HeadKind::Cap, None).next().unwrap();
    assert_eq!(cap.shape(), Shape::rectangle(2, 1));
}

#[test]
fn gilt_and_etched_only_when_rare() {
    for seed in 0..2000u64 {
        let mut rng = Seed::new(seed).rng();
        let form = roll(&mut rng, HeadKind::Helmet, None, Quality::Uncommon);
        assert!(form.finish.allowed(Quality::Uncommon), "{form:?}");
    }
}

#[test]
fn weights_match_the_worked_out_figures() {
    let find = |name: &str| {
        rolled(HeadKind::Helmet, None)
            .chain(rolled(HeadKind::BevoredHelm, None))
            .find(|form| {
                form.name() == name && form.material == Material::Steel && form.top == Top::None
            })
            .unwrap()
    };
    let grams = |name| find(name).grams();
    assert!(
        (2500..=3500).contains(&grams("Great Helm")),
        "{}",
        grams("Great Helm")
    );
    assert!(
        (1700..=2500).contains(&grams("Bascinet")),
        "{}",
        grams("Bascinet")
    );
    assert!((7000..=10_000).contains(&grams("Frog-mouth Helm")));
    let coif = rolled(HeadKind::Coif, None)
        .find(|form| form.construction == Construction::Mail && form.material == Material::Steel)
        .unwrap();
    assert!((1900..=2300).contains(&coif.grams()));
    for kind in KINDS {
        for form in rolled(kind, None) {
            assert!((150..=10_000).contains(&form.grams()), "{form:?}");
        }
    }
    let helm = find("Great Helm");
    assert_eq!(helm.grams_at(Size::Large), helm.grams() * 36 / 16);
}

#[test]
fn the_rim_runs_round_the_head_below_the_crown() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for kind in KINDS {
        for form in rolled(kind, None).take(300) {
            let rim = form.rim(&frame);
            assert_eq!(rim.len(), 16);
            let edge = form.edge(&frame);
            assert_eq!(rim[0][1], edge.front);
            assert_eq!(rim[4][1], edge.side);
            assert_eq!(rim[8][1], edge.back);
            assert!(rim.iter().all(|point| point[1] < frame.height), "{form:?}");
            assert!(rim[4][0] > frame.head.0 && rim[12][0] < -frame.head.0);
        }
    }
}
