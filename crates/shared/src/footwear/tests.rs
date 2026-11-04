use super::*;
use crate::creature::Size;
use crate::item::Quality;
use crate::rng::Seed;

fn rolled(piece: FootwearPiece, quality: Quality) -> impl Iterator<Item = FootwearForm> {
    (0..4000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("footwear").rng();
        roll(&mut rng, piece, quality)
    })
}

fn every(quality: Quality) -> Vec<FootwearForm> {
    FootwearPiece::ALL
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
fn jewels_only_come_at_rare_and_better() {
    let jewelled = |forms: Vec<FootwearForm>| {
        forms
            .iter()
            .any(|form| form.decoration == Decoration::Jewels)
    };
    assert!(!jewelled(every(Quality::Common)));
    assert!(jewelled(every(Quality::Rare)));
}

#[test]
fn every_choice_turns_up() {
    let forms = every(Quality::Rare);
    let seen = |test: &dyn Fn(&FootwearForm) -> bool| forms.iter().any(test);
    for build in Build::ALL {
        assert!(seen(&|form| form.build == build), "{build:?}");
    }
    for height in Height::ALL {
        assert!(seen(&|form| form.height == height), "{height:?}");
    }
    for toe in Toe::ALL {
        assert!(seen(&|form| form.toe == toe), "{toe:?}");
    }
    for heel in Heel::ALL {
        assert!(seen(&|form| form.heel == heel), "{heel:?}");
    }
    for fastening in Fastening::ALL {
        assert!(seen(&|form| form.fastening == fastening), "{fastening:?}");
    }
    for top in Top::ALL {
        assert!(seen(&|form| form.top == top), "{top:?}");
    }
    for decoration in Decoration::ALL {
        assert!(
            seen(&|form| form.decoration == decoration),
            "{decoration:?}"
        );
    }
    for under in Under::ALL {
        assert!(seen(&|form| form.under == under), "{under:?}");
    }
}

#[test]
fn every_name_turns_up() {
    let forms = every(Quality::Rare);
    for name in [
        "Oxfords",
        "Derbies",
        "Brogues",
        "Monk Shoes",
        "Loafers",
        "Court Shoes",
        "Mary Janes",
        "Saddle Shoes",
        "Latchet Shoes",
        "Buckled Shoes",
        "Turnshoes",
        "Poulaines",
        "Duckbill Shoes",
        "Moccasins",
        "Pampooties",
        "Opanaks",
        "Carbatinas",
        "Juttis",
        "Tsarouchia",
        "Espadrilles",
        "Bast Shoes",
        "Cloth Shoes",
        "Ghillies",
        "Chelsea Boots",
        "Chukka Boots",
        "Jodhpur Boots",
        "Hobnail Boots",
        "Button Boots",
        "Brogans",
        "Cowboy Boots",
        "Engineer Boots",
        "Mukluks",
        "Valenki",
        "Gutals",
        "Calcei",
        "Buskins",
        "Hwa",
        "Riding Boots",
        "Top Boots",
        "Hessian Boots",
        "Jackboots",
        "Russian Boots",
        "Cavalier Boots",
        "Caligae",
        "Crepidae",
        "Huaraches",
        "Kolhapuris",
        "Waraji",
        "Zori",
        "Reed Sandals",
        "Sabots",
        "Geta",
        "Padukas",
        "Nalins",
        "Bakya",
        "Mules",
        "Babouches",
        "Socci",
        // Generic names.
        "Shoes",
        "Ankle Boots",
        "Calf Boots",
        "Knee Boots",
        "Sandals",
        "Clogs",
        "Slippers",
        "Wooden Sandals",
        "Clog Boots",
    ] {
        assert!(
            forms.iter().any(|form| form.name().ends_with(name)),
            "{name}"
        );
    }
}

#[test]
fn only_the_cavalier_boot_goes_over_the_knee() {
    for form in every(Quality::Rare) {
        if form.height == Height::Thigh {
            assert_eq!(form.name(), "Cavalier Boots");
            assert_eq!(form.top, Top::Bucket);
        }
    }
}

#[test]
fn boots_take_more_cells_the_higher_they_come() {
    for form in every(Quality::Common) {
        let shape = form.shape();
        let long = match form.height {
            Height::Low => 1,
            Height::Ankle | Height::Calf => 2,
            Height::Knee | Height::Thigh => 3,
        };
        assert_eq!((shape.width, shape.height), (2, long), "{form:?}");
    }
}

#[test]
fn weights_are_a_pair_of_footwear() {
    for form in every(Quality::Rare) {
        let grams = form.grams();
        assert!((150..=3000).contains(&grams), "{grams} {form:?}");
        assert_eq!(form.grams_at(Size::Medium), grams);
        assert!(form.grams_at(Size::Large) > grams);
    }
}
