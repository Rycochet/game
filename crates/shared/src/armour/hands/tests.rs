use super::*;
use crate::armour::{Construction, Style};
use crate::body::{Frame, Side};
use crate::creature::CreatureKind;
use crate::creature::Size;
use crate::material::Material;

const PIECES: [HandsPiece; 3] = [HandsPiece::Gauntlets, HandsPiece::Demi, HandsPiece::Padded];

fn rolled(piece: HandsPiece) -> impl Iterator<Item = HandsForm> {
    (0..2000u64).map(move |seed| {
        let mut rng = crate::rng::Seed::new(seed).derive("hands").rng();
        roll(&mut rng, piece, None)
    })
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for piece in PIECES {
        for form in rolled(piece) {
            assert!(form.allowed(piece), "{piece:?} {form:?}");
        }
    }
}

#[test]
fn each_kind_rolls_only_what_it_covers() {
    assert!(
        rolled(HandsPiece::Gauntlets)
            .all(|form| form.fingers != Fingers::Open && form.construction != Construction::Padded)
    );
    assert!(rolled(HandsPiece::Demi).all(|form| form.fingers == Fingers::Open));
    assert!(rolled(HandsPiece::Padded).all(|form| form.construction == Construction::Padded));
}

#[test]
fn every_option_turns_up() {
    let all: Vec<_> = PIECES.into_iter().flat_map(rolled).collect();
    for cuff in [
        Cuff::Wrist,
        Cuff::Hourglass,
        Cuff::Bell,
        Cuff::Pointed,
        Cuff::Long,
    ] {
        assert!(all.iter().any(|form| form.cuff == cuff), "{cuff:?}");
    }
    for knuckles in [
        Knuckles::Plain,
        Knuckles::Ridged,
        Knuckles::Spiked,
        Knuckles::Rondel,
    ] {
        assert!(
            all.iter().any(|form| form.knuckles == knuckles),
            "{knuckles:?}"
        );
    }
    assert!(all.iter().any(|form| form.claws));
    assert!(all.iter().any(|form| form.locking));
    assert!(all.iter().any(|form| form.trim.is_some()));
    for name in [
        "Hourglass Gauntlets",
        "Tekkō",
        "Ring Mail Gloves",
        "Ring Mail Mittens",
        "Mail Mittens",
        "Padded Gloves",
    ] {
        assert!(all.iter().any(|form| form.name() == name), "{name}");
    }
}

#[test]
fn a_style_narrows_plate() {
    let rolled = |style| {
        (0..400u64).map(move |seed| {
            let mut rng = crate::rng::Seed::new(seed).rng();
            roll(&mut rng, HandsPiece::Gauntlets, Some(style))
        })
    };
    let plate = |form: &HandsForm| form.construction == Construction::Plate;
    assert!(
        rolled(Style::Kastenbrust)
            .filter(plate)
            .all(|form| form.fingers == Fingers::Mitten
                && form.cuff == Cuff::Long
                && form.style == Style::Kastenbrust)
    );
    assert!(
        rolled(Style::Gothic)
            .filter(plate)
            .all(|form| form.cuff == Cuff::Pointed)
    );
    assert!(
        rolled(Style::Maximilian)
            .filter(plate)
            .all(|form| form.fingers == Fingers::Fingered)
    );
}

#[test]
fn weights_match_the_worked_out_figures() {
    let plate = HandsForm {
        construction: Construction::Plate,
        fingers: Fingers::Fingered,
        cuff: Cuff::Hourglass,
        knuckles: Knuckles::Plain,
        claws: false,
        locking: false,
        material: Material::Steel,
        cloth: None,
        trim: None,
        style: Style::White,
    };
    assert_eq!(plate.grams(), 860);
    let long = HandsForm {
        cuff: Cuff::Long,
        ..plate
    };
    assert_eq!(long.grams(), 1150);
    let mithril = HandsForm {
        material: Material::Mithril,
        ..plate
    };
    assert!(mithril.grams() < plate.grams() * 2 / 3);
    let mail = HandsForm {
        construction: Construction::Mail,
        fingers: Fingers::Mitten,
        cuff: Cuff::Wrist,
        ..plate
    };
    assert!((600..=700).contains(&mail.grams()), "{}", mail.grams());
    for piece in PIECES {
        for form in rolled(piece) {
            assert!((100..=1500).contains(&form.grams()), "{form:?}");
        }
    }
    assert!(plate.grams_at(Size::Large) > plate.grams());
}

#[test]
fn hands_hang_from_the_wrist_and_fit_their_frame() {
    for kind in CreatureKind::ALL {
        let Some(frame) = Frame::of(kind) else {
            continue;
        };
        for side in Side::BOTH {
            let hand = Hand::of(&frame, side, Hand::digits(kind.def().fingers()));
            assert_eq!(hand.wrist[1] - hand.tip[1], hand.length);
            assert!(hand.palm < hand.length && hand.breadth > hand.thickness);
            assert!(hand.fingers >= 1);
        }
    }
    let frame = Frame::of(CreatureKind::Kobold).unwrap();
    assert_eq!(
        Hand::of(&frame, Side::Left, Hand::digits(Some(8))).fingers,
        3
    );
}
