use super::tables::constructions;
use super::*;
use crate::creature::Size;
use crate::item::Quality;
use crate::rng::Seed;

const KINDS: [WaistKind; 2] = [WaistKind::Armour, WaistKind::Cuirassier];

fn rolled(
    kind: WaistKind,
    style: Option<Style>,
    quality: Quality,
) -> impl Iterator<Item = WaistForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("waist").rng();
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
    }
    for style in Style::ALL {
        for form in rolled(WaistKind::Armour, Some(style), Quality::Rare).take(300) {
            assert!(form.allowed(), "{form:?}");
            assert_eq!(form.style, style);
        }
    }
}

#[test]
fn every_style_makes_only_what_it_allows() {
    for style in Style::ALL {
        for &(construction, _) in constructions(WaistKind::Armour, style) {
            assert!(style.allows(construction), "{style:?} {construction:?}");
            assert_ne!(construction, Construction::Mirror);
        }
    }
}

#[test]
fn every_shape_of_piece_turns_up() {
    let forms: Vec<_> = rolled(WaistKind::Armour, None, Quality::Common).collect();
    for piece in WaistPiece::ALL {
        assert!(forms.iter().any(|form| form.piece == piece), "{piece:?}");
    }
}

#[test]
fn cuirassier_tassets_are_knee_long_plate() {
    for form in rolled(WaistKind::Cuirassier, None, Quality::Common) {
        assert_eq!(
            (form.construction, form.piece, form.length),
            (Construction::Plate, WaistPiece::Tassets, Length::Knee)
        );
        assert_eq!(form.specific_name(), Some("Cuirassier Tassets"));
    }
}

#[test]
fn every_name_turns_up() {
    let mut names: Vec<String> = rolled(WaistKind::Armour, None, Quality::Common)
        .map(|form| form.name())
        .collect();
    for style in [
        Style::Japanese,
        Style::Classical,
        Style::Byzantine,
        Style::Maximilian,
        Style::Kastenbrust,
    ] {
        names.extend(
            rolled(WaistKind::Armour, Some(style), Quality::Common).map(|form| form.name()),
        );
    }
    for name in [
        "Tonlet",
        "Steel Bases",
        "Dendra Hip Plates",
        "Almain Tassets",
        "Kusazuri",
        "Cingulum Apron",
        "Pteruges",
        "War Kilt",
        "Kremasmata",
        "Mitra",
        "Italic Belt",
        "Fauld",
        "Tassets",
        "Mail Skirt",
        "Mail Breeches",
        "Lamellar Hip Panels",
        "Brigandine Skirt",
    ] {
        assert!(names.iter().any(|have| have == name), "{name}");
    }
}

#[test]
fn weights_are_near_what_survives() {
    let grams = |test: &dyn Fn(&WaistForm) -> bool| {
        let form = rolled(WaistKind::Armour, None, Quality::Common)
            .find(|form| test(form))
            .expect("rolled");
        form.grams()
    };
    let steel = |form: &WaistForm| form.material == Material::Steel;
    let mail_skirt = grams(&|form| {
        form.piece == WaistPiece::Skirt
            && form.construction == Construction::Mail
            && form.length == Length::Thigh
            && steel(form)
    });
    assert!((2500..=3600).contains(&mail_skirt), "{mail_skirt}");
    let tonlet = grams(&|form| {
        form.piece == WaistPiece::Skirt && form.construction == Construction::Plate && steel(form)
    });
    assert!((4500..=7500).contains(&tonlet), "{tonlet}");
    let pteruges = grams(&|form| {
        form.piece == WaistPiece::Pteruges && form.material == Material::Leather && !form.front
    });
    assert!((500..=2500).contains(&pteruges), "{pteruges}");
}

#[test]
fn weight_grows_with_the_square_of_size() {
    let form = rolled(WaistKind::Armour, None, Quality::Common)
        .next()
        .expect("rolled");
    assert_eq!(form.grams_at(Size::Medium), form.grams());
    let quarters = Size::Large.quarters();
    assert_eq!(
        form.grams_at(Size::Large),
        form.grams() * quarters * quarters / 16
    );
}
