use super::*;
use crate::creature::Size;
use crate::item::{Quality, Shape};
use crate::rng::Seed;

fn rolled(kind: TorsoKind, style: Option<Style>) -> impl Iterator<Item = TorsoForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("torso").rng();
        roll(&mut rng, kind, style, Quality::Rare)
    })
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for kind in [TorsoKind::Armour, TorsoKind::ArmingVest] {
        for form in rolled(kind, None) {
            assert!(form.allowed(), "{form:?}");
            assert_eq!(form.kind(), kind);
        }
    }
    for style in Style::ALL {
        for form in rolled(TorsoKind::Armour, Some(style)).take(300) {
            assert!(form.allowed(), "{form:?}");
            assert_eq!(form.style, style);
            assert!(style.allows(form.construction), "{form:?}");
        }
    }
}

#[test]
fn every_name_turns_up() {
    let all: Vec<_> = rolled(TorsoKind::Armour, None).collect();
    let mut names: Vec<_> = all.iter().map(|form| form.name()).collect();
    for style in Style::ALL {
        names.extend(rolled(TorsoKind::Armour, Some(style)).map(|form| form.name()));
    }
    for name in [
        "Cuirass",
        "Breastplate",
        "Leather Cuirass",
        "Mail Shirt",
        "Ring Mail",
        "Padded Jack",
        "Triple-disc Cuirass",
        "Kardiophylax",
        "Muscle Cuirass",
        "Bell Cuirass",
        "Tankō",
        "Kastenbrust",
        "Gothic Cuirass",
        "Milanese Cuirass",
        "Peascod Cuirass",
        "Munition Breastplate",
        "Elven Cuirass",
        "Dwarven Cuirass",
        "Maximilian Cuirass",
        "Studded Leather",
        "Buff Jerkin",
        "Leather Muscle Cuirass",
        "Mithril Shirt",
        "Kusari Katabira",
        "Lorica Hamata",
        "Byrnie",
        "Coir Armour",
        "Linothorax",
        "Myeonje Baegap",
        "Ichcahuipilli",
        "Hide Armour",
        "Scale Shirt",
        "Lorica Squamata",
        "Lorica Plumata",
        "Horn Scale Shirt",
        "Dragonscale Shirt",
        "Coin Armour",
        "Lamellar Cuirass",
        "Dō-maru",
        "Haramaki",
        "Ō-yoroi Dō",
        "Mountain Pattern Cuirass",
        "Klivanion",
        "Bone Lamellar",
        "Laminar Cuirass",
        "Lorica Segmentata",
        "Okegawa-dō",
        "Banded Mail",
        "Brigandine",
        "Coat of Plates",
        "Jack of Plate",
        "Leather Brigandine",
        "Dingjia",
        "Tatami Dō",
        "Plated Mail",
        "Yushman",
        "Bakhterets",
        "Zirah Bagtar",
        "Mirror Armour",
        "Char-aina",
        "Splint Cuirass",
        "Slat Armour",
        "Rattan Cuirass",
    ] {
        assert!(names.contains(&name), "{name}");
    }
    assert!(all.iter().any(|form| form.over_mail));
    assert!(all.iter().any(|form| form.lined));
    let vest = rolled(TorsoKind::ArmingVest, None).next().unwrap();
    assert_eq!(vest.name(), "Arming Vest");
    assert!(!vest.padded());
}

#[test]
fn padding_takes_the_middle_layer_but_the_vest_is_it() {
    for form in rolled(TorsoKind::Armour, None) {
        let padding = form.construction == Construction::Padded;
        assert_eq!(form.padded(), form.lined || padding, "{form:?}");
    }
}

#[test]
fn gilt_and_etched_only_when_rare() {
    for seed in 0..2000u64 {
        let mut rng = Seed::new(seed).rng();
        let form = roll(&mut rng, TorsoKind::Armour, None, Quality::Uncommon);
        assert!(form.finish.allowed(Quality::Uncommon), "{form:?}");
    }
}

#[test]
fn weights_match_the_worked_out_figures() {
    let plain = rolled(TorsoKind::Armour, Some(Style::White))
        .find(|form| {
            matches!(
                form.body,
                TorsoBody::Plate {
                    breast: Breast::Round,
                    plackart: Plackart::None,
                    lance_rest: false,
                    ..
                }
            ) && form.material == Material::Steel
                && form.back
                && !form.over_mail
                && !form.lined
                && form.trim.is_none()
        })
        .unwrap();
    assert!((5000..=5600).contains(&plain.grams()), "{}", plain.grams());
    let breastplate = TorsoForm {
        back: false,
        ..plain
    };
    assert!((3400..=3800).contains(&breastplate.grams()));
    assert_eq!(plain.shape(), Shape::rectangle(3, 3));
    assert_eq!(breastplate.shape(), Shape::rectangle(2, 2));
    let mithril = TorsoForm {
        material: Material::Mithril,
        ..plain
    };
    assert!(mithril.grams() < plain.grams() * 2 / 3);
    for form in rolled(TorsoKind::Armour, None) {
        assert!(
            (800..=13_000).contains(&form.grams()),
            "{form:?} {}",
            form.grams()
        );
    }
    assert_eq!(plain.grams_at(Size::Large), plain.grams() * 36 / 16);
}
