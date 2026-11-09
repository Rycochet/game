use super::armour::Piece;
use super::*;
use crate::armour::{Construction, Dye, Finish, Style};
use crate::creature::{CreatureKind, Size};
use crate::id::Id;
use crate::item::{Item, ItemKind, Quality};
use crate::material::Material;
use crate::rng::Seed;

/// How many seeds a request is tried with before it counts as never made.
const SEEDS: u64 = 24;

fn asked(
    kind: ItemKind,
    seed: u64,
    quality: Quality,
    request: &RollRequest,
) -> Result<Item, Refused> {
    let mut item = Item::new(Id::for_test(1), kind);
    item.quality = Some(quality);
    let mut next = 1 << 64;
    let mut new_id = || {
        next += 1;
        Id::for_test(next)
    };
    give_form_to(&mut item, None, Seed::new(seed), request, &mut new_id).map(|()| item)
}

/// The first seed's item that keeps to the request, if any does.
fn made(kind: ItemKind, quality: Quality, request: &RollRequest) -> Option<Item> {
    (0..SEEDS).find_map(|seed| asked(kind, seed, quality, request).ok())
}

fn form(item: &Item) -> &ItemForm {
    item.form.as_ref().expect("a form")
}

fn material_of(form: &ItemForm) -> Option<Material> {
    Some(match form {
        ItemForm::Arms(form) => form.material,
        ItemForm::Hands(form) => form.material,
        ItemForm::Shoulders(form) => form.material,
        ItemForm::Torso(form) => form.material,
        ItemForm::Head(form) => form.material,
        ItemForm::Legs(form) => form.material,
        ItemForm::Waist(form) => form.material,
        ItemForm::Feet(form) => form.material,
        ItemForm::Neck(form) => form.material,
        ItemForm::Footwear(form) => form.upper,
        ItemForm::Under(form) => form.material,
        ItemForm::Set(form) => form.material(),
        ItemForm::Ring(ring) => ring.metal,
        ItemForm::Blade(blade) => blade.blade_metal,
        ItemForm::Gem(gem) => gem.material,
        ItemForm::Key(_) => return None,
    })
}

fn armour_kinds() -> impl Iterator<Item = ItemKind> {
    ItemKind::ALL
        .into_iter()
        .filter(|&kind| Piece::of(kind).is_some())
}

#[test]
fn asking_for_nothing_rolls_as_give_form() {
    for kind in ItemKind::ALL {
        for seed in 0..5 {
            let plain = asked(kind, seed, Quality::Rare, &RollRequest::NONE).unwrap();
            let mut given = Item::new(Id::for_test(1), kind);
            given.quality = Some(Quality::Rare);
            let mut next = 1 << 64;
            give_form_from(&mut given, None, Seed::new(seed), &mut || {
                next += 1;
                Id::for_test(next)
            });
            assert_eq!(plain, given, "{kind:?}");
        }
    }
}

#[test]
fn options_a_kind_doesnt_take_are_refused() {
    let style = RollRequest {
        style: Some(Style::Gothic),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Gem, 1, Quality::Common, &style),
        Err(Refused(Field::Style))
    );
    let material = RollRequest {
        material: Some(Material::Iron),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Apple, 1, Quality::Common, &material),
        Err(Refused(Field::Material))
    );
    let finish = RollRequest {
        finish: Some(Finish::Bright),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Gauntlets, 1, Quality::Common, &finish),
        Err(Refused(Field::Finish))
    );
    assert_eq!(
        choices(ItemKind::Apple, Quality::Common, &RollRequest::NONE),
        Choices::default()
    );
}

#[test]
fn gems_are_of_the_material_and_cut_asked_for() {
    let listed = choices(ItemKind::Gem, Quality::Common, &RollRequest::NONE);
    assert_eq!(listed.gem_materials.len(), Material::GEMS.len());
    assert_eq!(listed.gem_cuts.len(), Cut::ALL.len());
    for material in listed.gem_materials {
        for cut in &listed.gem_cuts {
            let request = RollRequest {
                gem_material: Some(material),
                gem_cut: Some(*cut),
                ..RollRequest::NONE
            };
            for seed in 0..3 {
                let ItemForm::Gem(gem) =
                    *form(&asked(ItemKind::Gem, seed, Quality::Common, &request).unwrap())
                else {
                    panic!("a gem");
                };
                assert_eq!((gem.material, gem.cut), (material, *cut));
            }
        }
    }
    let iron = RollRequest {
        gem_material: Some(Material::Iron),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Gem, 1, Quality::Common, &iron),
        Err(Refused(Field::GemMaterial))
    );
}

#[test]
fn rings_keep_their_metal_settings_and_stones() {
    for quality in [Quality::Common, Quality::Legendary] {
        let listed = choices(ItemKind::Ring, quality, &RollRequest::NONE);
        assert!(
            !listed.gem_cuts.contains(&Cut::Uncut),
            "never uncut in a ring"
        );
        for &metal in &listed.materials {
            let request = RollRequest {
                material: Some(metal),
                ..RollRequest::NONE
            };
            let ring = made(ItemKind::Ring, quality, &request).expect("made");
            assert_eq!(material_of(form(&ring)), Some(metal));
        }
        for &count in &listed.settings {
            let request = RollRequest {
                settings: Some(count),
                ..RollRequest::NONE
            };
            let ItemForm::Ring(ring) =
                form(&made(ItemKind::Ring, quality, &request).expect("made")).clone()
            else {
                panic!("a ring");
            };
            assert_eq!(ring.settings.len(), count as usize);
        }
        let request = RollRequest {
            gem_material: Some(Material::Ruby),
            gem_cut: Some(Cut::Pear),
            ..RollRequest::NONE
        };
        for seed in 0..SEEDS {
            let ring = asked(ItemKind::Ring, seed, quality, &request).unwrap();
            let ItemForm::Ring(ring_form) = form(&ring) else {
                panic!("a ring");
            };
            assert!(
                !ring_form.settings.is_empty(),
                "a stone asked for has a setting"
            );
            for stone in form(&ring).gems() {
                let Some(ItemForm::Gem(gem)) = stone.form else {
                    panic!("a gem");
                };
                assert_eq!((gem.material, gem.cut), (Material::Ruby, Cut::Pear));
            }
        }
    }
    let copper = RollRequest {
        material: Some(Material::Copper),
        ..RollRequest::NONE
    };
    let five = choices(ItemKind::Ring, Quality::Poor, &copper).settings;
    assert!(
        !five.contains(&5),
        "only the finest have five, never copper"
    );
    let refused = [
        (
            RollRequest {
                gem_cut: Some(Cut::Uncut),
                ..RollRequest::NONE
            },
            Field::GemCut,
        ),
        (
            RollRequest {
                material: Some(Material::Steel),
                ..RollRequest::NONE
            },
            Field::Material,
        ),
        (
            RollRequest {
                settings: Some(0),
                gem_material: Some(Material::Ruby),
                ..RollRequest::NONE
            },
            Field::Settings,
        ),
        (
            RollRequest {
                settings: Some(5),
                material: Some(Material::Copper),
                ..RollRequest::NONE
            },
            Field::Settings,
        ),
    ];
    for (request, field) in refused {
        assert_eq!(
            asked(ItemKind::Ring, 1, Quality::Poor, &request),
            Err(Refused(field))
        );
    }
}

#[test]
fn blades_keep_their_metal_and_limits() {
    let poor = choices(ItemKind::Sword, Quality::Poor, &RollRequest::NONE);
    assert!(poor.limits);
    assert!(
        !poor.materials.contains(&Material::Gold),
        "gold only with better work"
    );
    let rare = choices(ItemKind::Sword, Quality::Rare, &RollRequest::NONE);
    for metal in rare.materials {
        let request = RollRequest {
            material: Some(metal),
            ..RollRequest::NONE
        };
        for seed in 0..5 {
            let sword = asked(ItemKind::Sword, seed, Quality::Rare, &request).unwrap();
            assert_eq!(material_of(form(&sword)), Some(metal));
        }
    }
    let gold = RollRequest {
        material: Some(Material::Gold),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Sword, 1, Quality::Poor, &gold),
        Err(Refused(Field::Material))
    );
    let limits = Limits {
        fits: Some((1, 3)),
        ..Limits::NONE
    };
    let within = RollRequest {
        limits: Some(limits),
        ..RollRequest::NONE
    };
    for seed in 0..10 {
        let blade = asked(ItemKind::Sword, seed, Quality::Common, &within).unwrap();
        assert!(
            limits.allows(blade.shape(), blade.total_weight()),
            "{:?}",
            blade.kind
        );
    }
    let nothing = RollRequest {
        limits: Some(Limits {
            most_grams: Some(1),
            ..Limits::NONE
        }),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Sword, 1, Quality::Common, &nothing),
        Err(Refused(Field::Limits))
    );
}

/// Every value `choices` lists for armour and footwear can be made, with no other
/// option asked for, and is what the item turns out to have.
#[test]
fn every_armour_choice_can_be_made() {
    let quality = Quality::Rare;
    for kind in armour_kinds() {
        let listed = choices(kind, quality, &RollRequest::NONE);
        let takes = fields(kind);
        assert_eq!(
            listed.styles.is_empty(),
            !takes.contains(&Field::Style),
            "{kind:?}"
        );
        assert_eq!(
            listed.materials.is_empty(),
            !takes.contains(&Field::Material),
            "{kind:?}"
        );
        for style in listed.styles {
            let request = RollRequest {
                style: Some(style),
                ..RollRequest::NONE
            };
            let item =
                made(kind, quality, &request).unwrap_or_else(|| panic!("{kind:?} {style:?}"));
            assert_eq!(form(&item).armour_style(), Some(style));
        }
        for construction in listed.constructions {
            let request = RollRequest {
                construction: Some(construction),
                ..RollRequest::NONE
            };
            let item = made(kind, quality, &request)
                .unwrap_or_else(|| panic!("{kind:?} {construction:?}"));
            assert_eq!(form(&item).armour_construction(), Some(construction));
        }
        for material in listed.materials {
            let request = RollRequest {
                material: Some(material),
                ..RollRequest::NONE
            };
            let item =
                made(kind, quality, &request).unwrap_or_else(|| panic!("{kind:?} {material:?}"));
            assert_eq!(material_of(form(&item)), Some(material));
        }
        for finish in listed.finishes {
            let request = RollRequest {
                finish: Some(finish),
                ..RollRequest::NONE
            };
            assert!(
                made(kind, quality, &request).is_some(),
                "{kind:?} {finish:?}"
            );
        }
        for dye in listed.dyes {
            let request = RollRequest {
                dye: Some(dye),
                ..RollRequest::NONE
            };
            assert!(made(kind, quality, &request).is_some(), "{kind:?} {dye:?}");
        }
    }
}

#[test]
fn a_set_is_made_as_asked_throughout() {
    let request = RollRequest {
        style: Some(Style::Gothic),
        construction: Some(Construction::Plate),
        material: Some(Material::Steel),
        finish: Some(Finish::Blackened),
        ..RollRequest::NONE
    };
    for seed in 0..5 {
        let item = asked(ItemKind::FullArmour, seed, Quality::Common, &request).unwrap();
        let ItemForm::Set(set) = form(&item) else {
            panic!("a set");
        };
        assert_eq!(set.style(), Style::Gothic);
        assert_eq!(set.construction(), Construction::Plate);
        assert_eq!(set.material(), Material::Steel);
        assert_eq!(set.torso.finish, Finish::Blackened);
        assert!(set.allowed());
    }
}

#[test]
fn what_cant_be_made_is_refused() {
    let mirror = RollRequest {
        style: Some(Style::Gothic),
        construction: Some(Construction::Mirror),
        ..RollRequest::NONE
    };
    let refused = asked(ItemKind::TorsoArmour, 1, Quality::Rare, &mirror);
    assert_eq!(refused, Err(Refused(Field::Construction)));
    let gilt = RollRequest {
        finish: Some(Finish::Gilt),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Helmet, 1, Quality::Common, &gilt),
        Err(Refused(Field::Finish))
    );
    assert!(
        !choices(ItemKind::Helmet, Quality::Common, &RollRequest::NONE)
            .finishes
            .contains(&Finish::Gilt)
    );
    let gem = RollRequest {
        material: Some(Material::Ruby),
        ..RollRequest::NONE
    };
    assert_eq!(
        asked(ItemKind::Helmet, 1, Quality::Common, &gem),
        Err(Refused(Field::Material))
    );
}

#[test]
fn a_style_narrows_the_constructions() {
    let any = choices(ItemKind::TorsoArmour, Quality::Rare, &RollRequest::NONE);
    let japanese = RollRequest {
        style: Some(Style::Japanese),
        ..RollRequest::NONE
    };
    let narrowed = choices(ItemKind::TorsoArmour, Quality::Rare, &japanese);
    assert!(narrowed.constructions.len() < any.constructions.len());
    assert!(
        narrowed
            .constructions
            .iter()
            .all(|&c| Style::Japanese.allows(c))
    );
    assert_eq!(
        narrowed.styles, any.styles,
        "an option's own value doesn't narrow it"
    );
}

#[test]
fn the_same_seed_and_request_make_the_same_item() {
    let request = RollRequest {
        construction: Some(Construction::Mail),
        dye: Some(Dye::Blue),
        ..RollRequest::NONE
    };
    let once = asked(ItemKind::LegArmour, 7, Quality::Rare, &request);
    assert_eq!(once, asked(ItemKind::LegArmour, 7, Quality::Rare, &request));
}

#[test]
fn make_item_rolls_as_usual_when_a_request_cant_be_met() {
    let order = ItemOrder {
        size: Some(Size::Medium),
        request: RollRequest {
            gem_cut: Some(Cut::Uncut),
            ..RollRequest::NONE
        },
        ..ItemOrder::plain(ItemKind::Ring, Quality::Common)
    };
    let human = CreatureKind::Human;
    let mut new_id = || Id::for_test(9);
    let refused = try_make_item(
        &order,
        human,
        Id::for_test(1),
        Seed::new(1),
        None,
        &mut new_id,
    );
    assert_eq!(refused, Err(Refused(Field::GemCut)));
    let plain = ItemOrder {
        request: RollRequest::NONE,
        ..order
    };
    let made = make_item(
        &order,
        human,
        Id::for_test(1),
        Seed::new(1),
        None,
        &mut new_id,
    );
    let usual = make_item(
        &plain,
        human,
        Id::for_test(1),
        Seed::new(1),
        None,
        &mut new_id,
    );
    assert_eq!(made, usual);
}
