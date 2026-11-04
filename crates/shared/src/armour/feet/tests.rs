use super::tables::constructions;
use super::*;
use crate::creature::Size;
use crate::item::Quality;
use crate::rng::Seed;

fn rolled(style: Option<Style>, quality: Quality) -> impl Iterator<Item = FeetForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("feet").rng();
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
fn every_piece_toe_and_name_turns_up() {
    let forms: Vec<_> = rolled(None, Quality::Rare).collect();
    for piece in FeetPiece::ALL {
        assert!(forms.iter().any(|form| form.piece == piece), "{piece:?}");
    }
    for toe in Toe::ALL {
        assert!(forms.iter().any(|form| form.toe == toe), "{toe:?}");
    }
    for name in [
        "Gothic Sabatons",
        "Duck-bill Sabatons",
        "Bear-paw Sabatons",
        "Cow-mouth Sabatons",
        "Clawed Sabatons",
        "Kusari Tabi",
        "Kogake",
        "Armoured Boots",
        "Mail Shoes",
        "Ring Mail Shoes",
        "Foot Guards",
        "Sabatons",
    ] {
        assert!(
            forms.iter().any(|form| form.name().ends_with(name)),
            "{name}"
        );
    }
    assert!(forms.iter().any(|form| form.detachable));
    assert!(forms.iter().any(|form| form.bare_toes()));
}

#[test]
fn points_stay_within_the_owner_s_lengths() {
    for form in rolled(None, Quality::Common) {
        let most = if form.detachable { 250 } else { 130 };
        assert!(form.point <= most, "{form:?}");
        assert_eq!(form.point > 0, form.toe == Toe::Pointed, "{form:?}");
    }
}

#[test]
fn weights_are_near_the_surviving_sabaton() {
    let plain = rolled(Some(Style::White), Quality::Common)
        .find(|form| {
            form.piece == FeetPiece::Sabatons
                && form.construction == Construction::Plate
                && form.material == crate::material::Material::Steel
                && form.toe == Toe::Round
                && !form.heel
                && !form.high
                && form.trim.is_none()
        })
        .unwrap();
    // About 200 g a foot (Higgins Armory), so 400 to 500 g a pair.
    assert!((400..=500).contains(&plain.grams()), "{}", plain.grams());
    for form in rolled(None, Quality::Common) {
        assert!((100..=1600).contains(&form.grams()), "{form:?}");
        let large = form.grams_at(Size::Large);
        assert!(large > form.grams(), "{form:?}");
    }
}

#[test]
fn sabatons_take_two_by_two_and_the_rest_two_by_one() {
    for form in rolled(None, Quality::Common) {
        let shape = form.shape();
        let long = if form.piece == FeetPiece::Sabatons {
            2
        } else {
            1
        };
        assert_eq!((shape.width, shape.height), (2, long), "{form:?}");
    }
}
