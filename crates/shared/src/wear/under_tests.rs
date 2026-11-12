//! Wearing underclothes together: one Skin layer, braies, loincloths and petticoats on
//! the Waist alone, and forms that leave arms or feet bare (owner, 2026-11-09).

use super::*;
use crate::armour::Dye;
use crate::creature::{CreatureKind, Location};
use crate::id::Id;
use crate::item::{Inventory, Item, ItemKind, Made, StackLimits};
use crate::item_form::ItemForm;
use crate::material::Material;
use crate::underclothes::{
    Braies, BraiesLength, Cloth, Colours, Decoration, Ends, Fit as BraiesFit, Garment, Held, Hose,
    HoseFeet, HoseHeight, HoseMake, Length, Neck, Petticoat, Shirt, SkirtFullness, SkirtLength,
    SkirtMake, SkirtWaist, Sleeves, SockHeight, SockMake, SockTop, Socks, Stays, Toe, UnderForm,
};

fn id(n: u128) -> Id {
    Id::for_test(n)
}

fn form(garment: Garment) -> UnderForm {
    UnderForm {
        garment,
        material: Material::Linen,
        cloth: Cloth::Plain,
        decoration: Decoration::Plain,
        dye: Dye::Natural,
        contrast: None,
    }
}

fn shirt(sleeves: Sleeves) -> (ItemKind, UnderForm) {
    let shirt = Shirt {
        length: Length::Thigh,
        sleeves,
        neck: Neck::Slit,
        cut: crate::underclothes::Cut::Gussets,
        night: false,
        stays: Stays::None,
    };
    (ItemKind::Shirt, form(Garment::Shirt(shirt)))
}

fn braies() -> (ItemKind, UnderForm) {
    let braies = Braies {
        length: BraiesLength::Knee,
        fit: BraiesFit::Loose,
        waist: crate::underclothes::Waistband::Belt,
        ends: Ends::Tucked,
    };
    (ItemKind::Braies, form(Garment::Braies(braies)))
}

fn hose(feet: HoseFeet) -> (ItemKind, UnderForm) {
    let hose = Hose {
        feet,
        height: HoseHeight::Thigh,
        make: HoseMake::BiasCut,
        sole: false,
        held: Held::Ties,
        colours: Colours::Plain,
        codpiece: false,
    };
    (ItemKind::Hose, form(Garment::Hose(hose)))
}

fn socks() -> (ItemKind, UnderForm) {
    let socks = Socks {
        make: SockMake::Sewn,
        toe: Toe::Whole,
        height: SockHeight::Calf,
        top: SockTop::Plain,
    };
    (ItemKind::Socks, form(Garment::Socks(socks)))
}

fn petticoat() -> (ItemKind, UnderForm) {
    let skirt = Petticoat {
        length: SkirtLength::Ankle,
        fullness: SkirtFullness::Full,
        make: SkirtMake::Plain,
        waist: SkirtWaist::Drawstring,
    };
    (ItemKind::Petticoat, form(Garment::Petticoat(skirt)))
}

/// These, made for a human, ids from 1.
fn carrying(pieces: &[(ItemKind, UnderForm)]) -> Inventory {
    let kind = CreatureKind::Human;
    let def = kind.def();
    let mut inventory = Inventory {
        creature: kind,
        size: (40, 40),
        ..Inventory::default()
    };
    for (index, &(item, under)) in pieces.iter().enumerate() {
        let new = Item {
            made: Made::for_body(item, def.size, def.fingers(), false),
            form: Some(ItemForm::Under(under)),
            ..Item::new(id(index as u128 + 1), item)
        };
        inventory
            .give(new, StackLimits::NONE, &mut crate::id::stand_ins())
            .expect("room");
    }
    inventory
}

#[test]
fn the_medieval_kit_goes_on_together() {
    let mut inventory = carrying(&[shirt(Sleeves::Wrist), braies(), hose(HoseFeet::Footed)]);
    for n in 1..=3 {
        assert_eq!(inventory.wear(id(n)), Ok(Fit::Exact), "piece {n}");
    }
}

#[test]
fn footed_hose_take_the_feet_so_no_socks_go_with_them() {
    let mut inventory = carrying(&[socks(), hose(HoseFeet::Footed)]);
    inventory.wear(id(1)).unwrap();
    assert!(matches!(
        inventory.check_wear(id(2)),
        Err(WearRefused::Clash(_))
    ));
}

#[test]
fn footless_and_stirrup_hose_leave_the_feet_for_socks() {
    for feet in [HoseFeet::Footless, HoseFeet::Stirrup] {
        let mut inventory = carrying(&[socks(), hose(feet)]);
        inventory.wear(id(1)).unwrap();
        assert_eq!(inventory.wear(id(2)), Ok(Fit::Exact), "{feet:?}");
    }
}

#[test]
fn a_petticoat_goes_over_stockings_but_not_with_braies() {
    let mut inventory = carrying(&[hose(HoseFeet::Footed), petticoat(), braies()]);
    inventory.wear(id(1)).unwrap();
    assert_eq!(inventory.wear(id(2)), Ok(Fit::Exact));
    assert!(matches!(
        inventory.check_wear(id(3)),
        Err(WearRefused::Clash(_))
    ));
}

#[test]
fn a_sleeveless_shirt_is_worn_on_the_torso_alone() {
    let inventory = carrying(&[shirt(Sleeves::None), shirt(Sleeves::Short)]);
    let worn = |n| inventory.find(id(n)).unwrap().worn_locations();
    assert_eq!(worn(1), vec![Location::Torso]);
    assert_eq!(worn(2), vec![Location::Torso, Location::Arms]);
}
