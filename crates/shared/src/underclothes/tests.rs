use std::collections::BTreeSet;

use super::*;
use crate::creature::{Location, Size};
use crate::item::Quality;
use crate::rng::Seed;

fn rolled(piece: UnderPiece, quality: Quality) -> impl Iterator<Item = UnderForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("underclothes").rng();
        roll(&mut rng, piece, quality)
    })
}

fn every(quality: Quality) -> Vec<UnderForm> {
    UnderPiece::ALL
        .into_iter()
        .flat_map(|piece| rolled(piece, quality))
        .collect()
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for quality in [Quality::Common, Quality::Rare] {
        for form in every(quality) {
            assert!(form.allowed(), "{form:?}");
        }
    }
}

#[test]
fn each_piece_rolls_its_own_garment() {
    for piece in UnderPiece::ALL {
        assert!(rolled(piece, Quality::Common).all(|form| form.piece() == piece));
        assert_eq!(UnderPiece::of(piece.kind()), Some(piece));
    }
}

#[test]
fn gold_thread_only_comes_at_rare_and_better() {
    let gold = |forms: Vec<UnderForm>| {
        forms
            .iter()
            .any(|form| form.decoration == Decoration::GoldThread)
    };
    assert!(!gold(every(Quality::Common)));
    assert!(gold(every(Quality::Rare)));
}

#[test]
fn every_name_from_the_research_turns_up() {
    let names: BTreeSet<_> = every(Quality::Rare)
        .iter()
        .map(UnderForm::specific_name)
        .collect();
    for wanted in [
        "Shirt",
        "Long Shirt",
        "Chemise",
        "Smock",
        "Undertunic",
        "Subucula",
        "Særk",
        "Hadajuban",
        "Nagajuban",
        "Zhongyi",
        "Nightshirt",
        "Night Chemise",
        "Hair Shirt",
        "Braies",
        "Short Braies",
        "Drawers",
        "Sokgot",
        "Fascia",
        "Strophium",
        "Sarashi",
        "Breast Wrap",
        "Breast Girdle",
        "Dudou",
        "Lengberg Band",
        "Loincloth",
        "Fundoshi",
        "Etchu",
        "Kaupinam",
        "Langota",
        "Cache-sexe",
        "Perizoma",
        "Subligaculum",
        "Tubban",
        "Hose",
        "Joined Hose",
        "Stockings",
        "Soled Hose",
        "Stirrup Hose",
        "Leggings",
        "Socks",
        "Udones",
        "Foot Wraps",
        "Tabi",
        "Coptic Socks",
        "Nålbound Socks",
        "Piloi",
        "Petticoat",
        "Underskirt",
        "Quilted Petticoat",
        "Koshimaki",
        "Sokchima",
        "Mamigun",
        "Coif",
        "Undercap",
        "Hairnet",
        "Caul",
        "Nightcap",
        "Wimple",
        "Barbe",
    ] {
        assert!(names.contains(wanted), "{wanted} never rolled");
    }
}

#[test]
fn stays_come_on_long_shirts_and_say_so() {
    let with_stays: Vec<_> = rolled(UnderPiece::Shirt, Quality::Common)
        .filter(|form| matches!(form.garment, Garment::Shirt(shirt) if shirt.stays != Stays::None))
        .collect();
    assert!(!with_stays.is_empty());
    for form in with_stays {
        let Garment::Shirt(shirt) = form.garment else {
            unreachable!()
        };
        assert!(shirt.length.long() && !shirt.night);
        assert!(form.name().contains(" and "), "{}", form.name());
    }
}

#[test]
fn a_coif_always_says_its_material() {
    for form in rolled(UnderPiece::Coif, Quality::Common) {
        if form.specific_name() == "Coif" {
            assert_ne!(form.name(), "Coif");
        }
    }
}

#[test]
fn sleeveless_shirts_and_footless_hose_leave_those_bare() {
    for form in every(Quality::Common) {
        let bare = form.garment.bare();
        match form.garment {
            Garment::Shirt(shirt) => {
                assert_eq!(
                    bare.contains(&Location::Arms),
                    shirt.sleeves == Sleeves::None
                )
            }
            Garment::Hose(hose) => {
                assert_eq!(
                    bare.contains(&Location::Feet),
                    hose.feet != HoseFeet::Footed
                )
            }
            _ => assert!(bare.is_empty()),
        }
    }
}

#[test]
fn weights_and_cells_are_sensible() {
    for form in every(Quality::Rare) {
        let grams = form.grams();
        assert!(
            (5..=2500).contains(&grams),
            "{} weighs {grams}",
            form.name()
        );
        assert!(form.grams_at(Size::Large) > grams);
        let shape = form.shape();
        assert!(shape.width <= 2 && shape.height <= 3);
    }
}
