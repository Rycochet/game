use super::*;

fn rolled(kind: Option<ShouldersKind>) -> impl Iterator<Item = ShouldersForm> {
    (0..3000u64).map(move |seed| {
        let mut rng = crate::rng::Seed::new(seed).derive("shoulders").rng();
        roll(&mut rng, kind, None)
    })
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for form in rolled(None) {
        assert!(form.allowed(), "{form:?}");
    }
    for kind in ShouldersKind::ALL {
        for form in rolled(Some(kind)).take(500) {
            assert_eq!(form.kind(), kind);
            assert!(form.allowed(), "{form:?}");
        }
    }
}

#[test]
fn every_kind_and_name_turns_up() {
    let all: Vec<_> = rolled(None).collect();
    for kind in ShouldersKind::ALL {
        assert!(all.iter().any(|form| form.kind() == kind), "{kind:?}");
    }
    for name in [
        "Pauldrons",
        "Great Pauldrons",
        "Spiked Pauldrons",
        "Skull Pauldrons",
        "Spaulders",
        "Dendra Shoulder Guards",
        "Lorica Shoulder Guards",
        "Ō-sode",
        "Ko-sode",
        "Lamellar Shoulder Guards",
        "Mail Doubling",
        "Linothorax Yoke",
        "Ring Mail Cape",
        "Naplechniki",
        "Ailettes",
        "Chukchi Wing",
    ] {
        assert!(all.iter().any(|form| form.name() == name), "{name}");
    }
    assert!(all.iter().any(|form| form.finish == Finish::Studded));
    assert!(
        all.iter()
            .any(|form| form.adornment == Adornment::BeastHead)
    );
    assert!(all.iter().any(|form| form.trim.is_some()));
}

#[test]
fn heraldry_puts_metal_on_colour() {
    for seed in 0..500 {
        let heraldry = Heraldry::roll(&mut crate::rng::Seed::new(seed).rng());
        assert!(heraldry.allowed(), "{heraldry:?}");
    }
}

#[test]
fn a_style_leans_the_kind() {
    let count = |style, kind| {
        (0..1000u64)
            .filter(|&seed| {
                roll(&mut crate::rng::Seed::new(seed).rng(), None, Some(style)).kind() == kind
            })
            .count()
    };
    assert!(
        count(Style::Gothic, ShouldersKind::Spaulders)
            > count(Style::White, ShouldersKind::Spaulders)
    );
    let milanese = (0..300u64).map(|seed| {
        roll(
            &mut crate::rng::Seed::new(seed).rng(),
            Some(ShouldersKind::Pauldrons),
            Some(Style::Milanese),
        )
    });
    for form in milanese {
        if form.construction == Construction::Plate
            && let ShouldersPiece::Pauldrons { wings, .. } = form.piece
        {
            assert_eq!(wings, Wings::Large);
        }
    }
}

#[test]
fn weights_match_the_worked_out_figures() {
    let plate = ShouldersForm {
        piece: ShouldersPiece::Pauldrons {
            cut: Cut::Matched,
            wings: Wings::Small,
            lames: 4,
            upper_lames: 0,
            haute_piece: HautePiece::None,
            gardbrace: false,
            besagews: false,
        },
        construction: Construction::Plate,
        adornment: Adornment::None,
        finish: Finish::Plain,
        material: Material::Steel,
        cloth: None,
        trim: None,
        style: Style::White,
    };
    assert!((2500..=2800).contains(&plate.grams()), "{}", plate.grams());
    let mail = ShouldersForm {
        piece: ShouldersPiece::Cape {
            cut: CapeCut::Round,
            length: Length::Long,
        },
        construction: Construction::Mail,
        ..plate
    };
    assert!((2000..=2200).contains(&mail.grams()), "{}", mail.grams());
    let mithril = ShouldersForm {
        material: Material::Mithril,
        ..plate
    };
    assert!(mithril.grams() < plate.grams() * 2 / 3);
    for form in rolled(None) {
        assert!(
            (100..=6000).contains(&form.grams()),
            "{form:?} {}",
            form.grams()
        );
    }
    assert!(plate.grams_at(Size::Large) > plate.grams());
}
