use super::tables::constructions;
use super::*;
use crate::creature::Size;
use crate::item::Quality;
use crate::rng::Seed;

fn rolled(
    kind: NeckKind,
    style: Option<Style>,
    quality: Quality,
) -> impl Iterator<Item = NeckForm> {
    (0..2000u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("neck").rng();
        roll(&mut rng, kind, style, quality)
    })
}

fn every(quality: Quality) -> impl Iterator<Item = NeckForm> {
    NeckKind::ALL
        .into_iter()
        .flat_map(move |kind| rolled(kind, None, quality))
}

#[test]
fn every_roll_keeps_to_its_grid() {
    for quality in [Quality::Common, Quality::Rare] {
        for form in every(quality) {
            assert!(form.allowed(), "{form:?}");
            assert!(form.finish.allowed(quality), "{form:?}");
        }
    }
    for kind in NeckKind::ALL {
        for style in Style::ALL {
            for form in rolled(kind, Some(style), Quality::Rare).take(200) {
                assert!(form.allowed(), "{form:?}");
                assert_eq!((form.kind, form.style), (kind, style));
            }
        }
    }
}

#[test]
fn every_style_makes_only_what_it_allows() {
    for kind in NeckKind::ALL {
        for style in Style::ALL {
            for &(construction, _) in constructions(kind, style) {
                assert!(
                    style.allows(construction),
                    "{kind:?} {style:?} {construction:?}"
                );
            }
        }
    }
}

#[test]
fn every_shape_and_name_turns_up() {
    let forms: Vec<_> = every(Quality::Rare).collect();
    for plates in Plates::ALL {
        assert!(forms.iter().any(|form| form.plates == plates), "{plates:?}");
    }
    for front in Front::ALL {
        assert!(forms.iter().any(|form| form.front == front), "{front:?}");
    }
    for name in [
        "Padded Collar",
        "Standard",
        "Ring Mail Collar",
        "Guruwa",
        "Plate Collar",
        "Gorget",
        "Colletin",
        "Leather Gorget",
        "Scale Gorget",
        "Lamellar Gorget",
        "Nodowa",
        "Bevor",
        "Leather Bevor",
        "Aventail",
        "Lamellar Aventail",
        "Scale Aventail",
        "Quilted Aventail",
        "Leather Aventail",
        "Plated Aventail",
        "Bishop's Mantle",
        "Scale Mantle",
    ] {
        assert!(
            forms.iter().any(|form| form.name().ends_with(name)),
            "{name}"
        );
    }
    assert!(forms.iter().any(|form| form.spikes));
    assert!(forms.iter().any(|form| form.over_breastplate()));
    assert!(forms.iter().any(|form| form.edging == Edging::Patterned));
    assert!(forms.iter().any(|form| form.hem == Hem::Vandyked));
    assert!(forms.iter().any(|form| form.hem == Hem::Scalloped));
}

#[test]
fn weights_are_near_the_museum_pieces() {
    let steel = |form: &NeckForm| {
        form.construction == Construction::Plate
            && form.material == crate::material::Material::Steel
            && form.trim.is_none()
            && !form.spikes
    };
    let gorget = rolled(NeckKind::Gorget, Some(Style::White), Quality::Common)
        .find(|form| steel(form) && form.plates == Plates::Plates && form.high)
        .unwrap();
    assert!((900..=1400).contains(&gorget.grams()), "{}", gorget.grams());
    let bevor = rolled(NeckKind::Bevor, Some(Style::White), Quality::Common)
        .find(|form| steel(form) && form.high && form.lames == 0)
        .unwrap();
    assert!((850..=1300).contains(&bevor.grams()), "{}", bevor.grams());
    let aventail = rolled(NeckKind::Aventail, Some(Style::White), Quality::Common)
        .find(|form| {
            form.construction == Construction::Mail
                && form.material == crate::material::Material::Iron
                && form.front == Front::Chin
                && !form.cape
        })
        .unwrap();
    assert!(
        (1000..=1600).contains(&aventail.grams()),
        "{}",
        aventail.grams()
    );
    for form in every(Quality::Common) {
        assert!((100..=3500).contains(&form.grams()), "{form:?}");
        assert!(form.grams_at(Size::Large) > form.grams(), "{form:?}");
    }
}

#[test]
fn collars_take_two_by_one_and_the_rest_two_by_two() {
    for form in every(Quality::Common) {
        let shape = form.shape();
        let collar = form.kind == NeckKind::Collar
            || (form.kind == NeckKind::Gorget && form.plates == Plates::Collar);
        assert_eq!(
            (shape.width, shape.height),
            (2, if collar { 1 } else { 2 }),
            "{form:?}"
        );
    }
}
