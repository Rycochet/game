use super::tables::constructions;
use super::*;
use crate::creature::Size;
use crate::item::Quality;
use crate::rng::Seed;

fn rolled(style: Option<Style>, quality: Quality) -> impl Iterator<Item = ArmsForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("arms").rng();
        roll(&mut rng, style, quality)
    })
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for quality in [Quality::Common, Quality::Rare] {
        for form in rolled(None, quality) {
            assert!(form.allowed(), "{form:?}");
            assert!(form.finish.allowed(quality), "{form:?}");
        }
    }
    for style in Style::ALL {
        for form in rolled(Some(style), Quality::Rare).take(300) {
            assert!(form.allowed(), "{form:?}");
            assert_eq!(form.style, style);
        }
    }
}

#[test]
fn every_style_makes_only_what_it_allows() {
    for style in Style::ALL {
        for &(construction, _) in constructions(style) {
            assert!(style.allows(construction), "{style:?} {construction:?}");
        }
    }
}

#[test]
fn every_name_turns_up() {
    let mut names: Vec<String> = rolled(None, Quality::Common)
        .map(|form| form.name())
        .collect();
    for style in [Style::Japanese, Style::Classical, Style::IndoPersian] {
        names.extend(rolled(Some(style), Quality::Common).map(|form| form.name()));
    }
    for name in [
        "Arm Harness",
        "Vambraces",
        "Couters",
        "Rerebraces",
        "Bracers",
        "Leather Bracers",
        "Studded Leather Vambraces",
        "Splinted Arms",
        "Splinted Vambraces",
        "Mail Sleeves",
        "Ring Mail Sleeves",
        "Scale Sleeves",
        "Lamellar Sleeves",
        "Padded Sleeves",
        "Brigandine Sleeves",
        "Kote",
        "Han Kote",
        "Kusari Gote",
        "Manica",
        "Manicae",
        "Bazubands",
        "Archer's Bracer",
        "Bronze Vambraces",
    ] {
        assert!(names.iter().any(|have| have == name), "{name}");
    }
    assert!(names.iter().any(|name| name.starts_with("Spiked ")));
}

#[test]
fn weights_are_near_the_worked_out_ones() {
    let first = |wanted: &dyn Fn(&ArmsForm) -> bool| {
        rolled(None, Quality::Common)
            .find(|form| wanted(form))
            .expect("rolled")
    };
    let harness = first(&|form| {
        form.construction == Construction::Plate
            && form.reach == Reach::Whole
            && form.material == Material::Steel
    });
    assert!((3000..4000).contains(&harness.grams()), "{harness:?}");
    let bracers = first(&|form| {
        form.construction == Construction::Leather
            && form.reach == Reach::Wrist
            && form.sides == Sides::Both
    });
    assert!((150..450).contains(&bracers.grams()), "{bracers:?}");
    let mail =
        first(&|form| form.construction == Construction::Mail && form.material == Material::Iron);
    assert!((2400..3400).contains(&mail.grams()), "{mail:?}");
    // By the square of size.
    let small = harness.grams_at(Size::Small) as u64;
    let large = harness.grams_at(Size::Large) as u64;
    assert!(large > small);
}

#[test]
fn only_whole_kote_take_the_middle_layer() {
    for form in rolled(Some(Style::Japanese), Quality::Common) {
        assert_eq!(form.padded(), form.kote() && form.reach == Reach::Whole);
    }
    assert!(rolled(None, Quality::Common).any(|form| form.padded()));
}

#[test]
fn spans_run_down_the_arm_and_stop_at_the_wrist() {
    let frame = crate::body::Frame::of(crate::creature::CreatureKind::Human).expect("frame");
    for reach in Reach::ALL {
        let (top, bottom) = reach.span(&frame);
        assert!(top < bottom, "{reach:?}");
        assert!(top >= 100 && bottom < frame.elbow_length + frame.wrist_length);
    }
}
