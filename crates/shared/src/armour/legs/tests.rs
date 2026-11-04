use super::tables::constructions;
use super::*;
use crate::armour::arms::Sides;
use crate::creature::Size;
use crate::item::Quality;
use crate::rng::Seed;

const KINDS: [LegsKind; 3] = [LegsKind::Armour, LegsKind::Chausses, LegsKind::Padded];

fn rolled(
    kind: LegsKind,
    style: Option<Style>,
    quality: Quality,
) -> impl Iterator<Item = LegsForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("legs").rng();
        roll(&mut rng, kind, style, quality)
    })
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for kind in KINDS {
        for quality in [Quality::Common, Quality::Rare] {
            for form in rolled(kind, None, quality) {
                assert!(form.allowed(), "{form:?}");
                assert!(form.finish.allowed(quality), "{form:?}");
                assert_eq!(form.kind, kind);
            }
        }
        for style in Style::ALL {
            for form in rolled(kind, Some(style), Quality::Rare).take(300) {
                assert!(form.allowed(), "{form:?}");
                assert_eq!(form.style, style);
            }
        }
    }
}

#[test]
fn every_style_makes_only_what_it_allows() {
    for style in Style::ALL {
        for &(construction, _) in constructions(LegsKind::Armour, style) {
            assert!(style.allows(construction), "{style:?} {construction:?}");
        }
    }
}

#[test]
fn chausses_and_padded_chausses_cover_the_whole_leg() {
    for form in rolled(LegsKind::Chausses, None, Quality::Common) {
        assert_eq!(
            (form.construction, form.cover),
            (Construction::Mail, Cover::Whole)
        );
        assert!(form.footed());
    }
    for form in rolled(LegsKind::Padded, None, Quality::Common) {
        assert_eq!(
            (form.construction, form.cover),
            (Construction::Padded, Cover::Whole)
        );
        assert_eq!(form.knee, Knee::None);
    }
}

#[test]
fn every_name_turns_up() {
    let mut names: Vec<String> = rolled(LegsKind::Armour, None, Quality::Common)
        .map(|form| form.name())
        .collect();
    for style in [
        Style::Japanese,
        Style::Classical,
        Style::IndoPersian,
        Style::Steppe,
    ] {
        names
            .extend(rolled(LegsKind::Armour, Some(style), Quality::Common).map(|form| form.name()));
    }
    names.extend(rolled(LegsKind::Chausses, None, Quality::Common).map(|form| form.name()));
    names.extend(rolled(LegsKind::Padded, None, Quality::Common).map(|form| form.name()));
    for name in [
        "Leg Harness",
        "Cuisses",
        "Poleyns",
        "Greaves",
        "Schynbalds",
        "Leather Greaves",
        "Leather Knee Guards",
        "Splinted Greaves",
        "Splinted Legs",
        "Brigandine Cuisses",
        "Gamboised Cuisses",
        "Chausses",
        "Footless Chausses",
        "Demi-chausses",
        "Reinforced Chausses",
        "Ring Mail Leggings",
        "Padded Chausses",
        "Suneate",
        "Tsutsu-suneate",
        "Haidate",
        "Knemides",
        "Ocreae",
        "Ring Greaves",
        "Perimeridia",
        "Episphyria",
        "Leg Manicae",
        "Dizçek",
        "Hooped Greaves",
        "Lamellar Thigh Guards",
    ] {
        assert!(names.iter().any(|have| have == name), "{name}");
    }
    for prefix in ["Spiked ", "Bladed ", "Studded "] {
        assert!(
            names.iter().any(|name| name.starts_with(prefix)),
            "{prefix}"
        );
    }
}

#[test]
fn some_classical_greaves_are_for_one_leg() {
    let single = rolled(LegsKind::Armour, Some(Style::Classical), Quality::Common)
        .find(|form| form.sides != Sides::Both)
        .expect("a single greave");
    assert_eq!(single.sides.count(), 1);
    assert!(
        single.name().ends_with("Knemis") || single.name().ends_with("Ocrea") || single.coiled()
    );
}

#[test]
fn weights_are_near_the_worked_out_ones() {
    let first = |kind, wanted: &dyn Fn(&LegsForm) -> bool| {
        rolled(kind, None, Quality::Common)
            .find(|form| wanted(form))
            .expect("rolled")
    };
    let harness = first(LegsKind::Armour, &|form| {
        form.construction == Construction::Plate
            && form.cover == Cover::Whole
            && form.closed
            && form.knee == Knee::Cop
            && form.material == Material::Steel
    });
    assert!((3500..4800).contains(&harness.grams()), "{harness:?}");
    let poleyns = first(LegsKind::Armour, &|form| {
        form.construction == Construction::Plate && form.cover == Cover::Knee
    });
    assert!((450..900).contains(&poleyns.grams()), "{poleyns:?}");
    let chausses = first(LegsKind::Chausses, &|form| {
        form.reinforce == Reinforce::None && form.closed && form.material == Material::Iron
    });
    assert!((3500..5000).contains(&chausses.grams()), "{chausses:?}");
    let padded = first(LegsKind::Padded, &|_| true);
    assert!((700..1700).contains(&padded.grams()), "{padded:?}");
    // By the square of size.
    assert!(harness.grams_at(Size::Large) > harness.grams_at(Size::Small));
}

#[test]
fn spans_run_down_the_leg_from_the_crotch_to_the_ankle() {
    let frame = crate::body::Frame::of(crate::creature::CreatureKind::Human).expect("frame");
    for cover in Cover::ALL {
        let (top, bottom) = cover.span(&frame);
        assert!(top > bottom, "{cover:?}");
        assert!(
            top <= frame.hip - frame.thigh && bottom >= frame.ankle - 40,
            "{cover:?}"
        );
    }
}
