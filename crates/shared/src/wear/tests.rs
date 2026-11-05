use super::*;
use crate::armour::hands::Fingers;
use crate::creature::{CreatureKind, Location, Size};
use crate::id::Id;
use crate::item::StackLimits;
use crate::item::{Favour, Inventory, Item, Made, Quality};
use crate::item::{Holder, ItemKind, MoveRefused, Rotation};
use crate::item_form::ItemForm;

const HUMAN: CreatureKind = CreatureKind::Human;

fn id(n: u128) -> Id {
    Id::for_test(n)
}

/// These kinds, made for this creature, ids from 1, in a big inventory.
fn carrying_for(kind: CreatureKind, kinds: &[ItemKind]) -> Inventory {
    let def = kind.def();
    let mut inventory = Inventory {
        creature: kind,
        size: (40, 40),
        ..Inventory::default()
    };
    for (index, &item) in kinds.iter().enumerate() {
        let made = Made::for_body(item, def.size, def.fingers(), false);
        let new = Item {
            made,
            ..Item::new(id(index as u128 + 1), item)
        };
        inventory.give(new, StackLimits::NONE).expect("room");
    }
    inventory
}

fn carrying(kinds: &[ItemKind]) -> Inventory {
    carrying_for(HUMAN, kinds)
}

/// Takes it off into a far corner of the inventory.
fn take_off(inventory: &mut Inventory, n: u128) -> Result<(), MoveRefused> {
    inventory.move_item(id(n), Holder::Inventory, 30, 30, Rotation::None)
}

#[test]
fn a_higher_layer_stops_what_is_under_it_going_on_or_coming_off() {
    use ItemKind::{Boots, Socks};
    let mut inventory = carrying(&[Socks, Boots]);
    assert_eq!(inventory.wear(id(2)), Ok(Fit::Exact));
    // Socks can't go on under boots already on.
    assert_eq!(
        inventory.check_wear(id(1)),
        Err(WearRefused::Covered(id(2)))
    );
    take_off(&mut inventory, 2).unwrap();
    inventory.wear(id(1)).unwrap();
    inventory.wear(id(2)).unwrap();
    // Nor come off under them.
    assert_eq!(inventory.covered_by(id(1)), Some(id(2)));
    assert_eq!(take_off(&mut inventory, 1), Err(MoveRefused::Covered));
    assert!(!inventory.to_bin(id(1)));
    take_off(&mut inventory, 2).unwrap();
    assert_eq!(take_off(&mut inventory, 1), Err(MoveRefused::NoRoom));
    assert_eq!(
        inventory.move_item(id(1), Holder::Inventory, 32, 30, Rotation::None),
        Ok(())
    );
}

#[test]
fn a_backpack_and_a_cloak_share_the_back_over_armour() {
    use ItemKind::{Backpack, Cloak, FullArmour};
    let mut inventory = carrying(&[Backpack, FullArmour, Cloak, Backpack]);
    inventory.wear(id(1)).unwrap();
    assert_eq!(inventory.wear(id(2)), Ok(Fit::Exact));
    assert_eq!(inventory.wear(id(3)), Ok(Fit::Exact));
    // The back has room for two.
    assert!(matches!(
        inventory.check_wear(id(4)),
        Err(WearRefused::Clash(_))
    ));
}

#[test]
fn padded_armour_takes_the_middle_layer_too() {
    use ItemKind::{FullArmour, Gambeson, TShirt, Trousers};
    let mut inventory = carrying(&[Gambeson, FullArmour, TShirt, Trousers]);
    inventory.wear(id(1)).unwrap();
    // A full suit has its own padding, so a gambeson is in its way...
    assert_eq!(inventory.check_wear(id(2)), Err(WearRefused::Clash(id(1))));
    take_off(&mut inventory, 1).unwrap();
    // ...but not trousers, where it has none.
    inventory.wear(id(4)).unwrap();
    inventory.wear(id(2)).unwrap();
    // Its padding covers what is under it as well.
    assert_eq!(
        inventory.check_wear(id(3)),
        Err(WearRefused::Covered(id(2)))
    );
    // And nothing Middle goes on under it.
    assert_eq!(inventory.check_wear(id(1)), Err(WearRefused::Clash(id(2))));
}

#[test]
fn lined_torso_armour_takes_the_middle_layer_but_unlined_goes_over_a_vest() {
    use crate::armour::torso::{self, TorsoKind};
    use ItemKind::{ArmingVest, Gambeson, TorsoArmour};
    let mut inventory = carrying(&[ArmingVest, TorsoArmour, Gambeson]);
    inventory.wear(id(1)).unwrap();
    // Without a form it is unlined, so it goes over the vest.
    assert_eq!(inventory.wear(id(2)), Ok(Fit::Exact));
    // A vest and a gambeson clash on the torso.
    take_off(&mut inventory, 2).unwrap();
    assert_eq!(inventory.check_wear(id(3)), Err(WearRefused::Clash(id(1))));
    let mut rng = crate::rng::Seed::new(1).rng();
    let lined = (0..)
        .map(|_| torso::roll(&mut rng, TorsoKind::Armour, None, Quality::Common))
        .find(|form| form.lined)
        .unwrap();
    let mut armour = Item::new(id(9), TorsoArmour);
    armour.form = Some(ItemForm::Torso(lined));
    assert_eq!(armour.padded(), &[Location::Torso]);
}

#[test]
fn surcoats_and_caparisons_go_over_armour() {
    use ItemKind::{Barding, Caparison, FullArmour, Surcoat, Tabard};
    let mut inventory = carrying(&[FullArmour, Surcoat, Tabard]);
    inventory.wear(id(1)).unwrap();
    assert_eq!(inventory.wear(id(2)), Ok(Fit::Exact));
    assert_eq!(inventory.covered_by(id(1)), Some(id(2)));
    // Both are on the torso.
    assert_eq!(inventory.check_wear(id(3)), Err(WearRefused::Clash(id(2))));

    let mut horse = carrying_for(CreatureKind::Horse, &[Barding, Caparison]);
    horse.wear(id(1)).unwrap();
    assert_eq!(horse.wear(id(2)), Ok(Fit::Exact));
    assert_eq!(horse.covered_by(id(1)), Some(id(2)));
}

#[test]
fn gloves_lock_rings_and_fingerless_ones_dont() {
    use ItemKind::{FingerlessGloves, Gauntlets, Gloves, Ring};
    let mut inventory = carrying(&[Ring, Gloves, Ring, Gauntlets, FingerlessGloves]);
    inventory.wear(id(1)).unwrap();
    inventory.wear(id(2)).unwrap();
    // The ring on stays locked, and another can't go on.
    assert_eq!(inventory.covered_by(id(1)), Some(id(2)));
    assert_eq!(
        inventory.check_wear(id(3)),
        Err(WearRefused::Covered(id(2)))
    );
    // A gauntlet goes over a glove.
    assert_eq!(inventory.wear(id(4)), Ok(Fit::Exact));
    assert_eq!(inventory.covered_by(id(2)), Some(id(4)));

    let mut bare = carrying(&[Ring, FingerlessGloves, Ring]);
    bare.wear(id(2)).unwrap();
    assert_eq!(bare.wear(id(1)), Ok(Fit::Exact));
    assert_eq!(bare.covered_by(id(1)), None);
    assert_eq!(bare.wear(id(3)), Ok(Fit::Exact));
}

#[test]
fn each_finger_takes_one_ring() {
    let mut inventory = carrying(&[ItemKind::Ring; 11]);
    for n in 1..=10 {
        inventory.wear(id(n)).unwrap();
    }
    assert!(matches!(
        inventory.check_wear(id(11)),
        Err(WearRefused::Clash(_))
    ));
}

#[test]
fn gloves_need_the_right_number_of_fingers_unless_magical() {
    let kobold = CreatureKind::Kobold;
    // Made for ten fingers, of any size that fits a kobold.
    let gloves = |magical| Item {
        made: Made {
            size: Size::Small,
            smallest: Size::Small,
            largest: Size::Small,
            fingers: Some(10),
            magical,
        },
        quality: Some(Quality::Uncommon),
        ..Item::new(id(1), ItemKind::Gloves)
    };
    let mut inventory = Inventory {
        creature: kobold,
        ..Inventory::default()
    };
    inventory.give(gloves(false), StackLimits::NONE).unwrap();
    assert_eq!(inventory.check_wear(id(1)), Err(WearRefused::WrongFingers));
    let mut magic = Inventory {
        creature: kobold,
        ..Inventory::default()
    };
    magic.give(gloves(true), StackLimits::NONE).unwrap();
    assert_eq!(magic.check_wear(id(1)), Ok(Fit::Exact));
    // Fingerless ones don't care.
    let mut fingerless = carrying_for(kobold, &[ItemKind::FingerlessGloves]);
    assert_eq!(fingerless.wear(id(1)), Ok(Fit::Exact));
}

#[test]
fn mitten_gauntlets_fit_any_fingers_and_demi_gauntlets_leave_rings_free() {
    use crate::armour::hands::{self, HandsPiece};
    let kobold = CreatureKind::Kobold;
    // Gauntlets made for ten fingers, as rolled, with mittens or fingers.
    let gauntlets = |fingers| {
        let form = (0..)
            .map(|seed| {
                hands::roll(
                    &mut crate::rng::Seed::new(seed).rng(),
                    HandsPiece::Gauntlets,
                    None,
                )
            })
            .find(|form| form.fingers == fingers)
            .unwrap();
        Item {
            made: Made {
                fingers: Some(10),
                ..Made::for_body(ItemKind::Gauntlets, Size::Small, Some(10), false)
            },
            form: Some(ItemForm::Hands(form)),
            ..Item::new(id(1), ItemKind::Gauntlets)
        }
    };
    let wearing = |item| {
        let mut inventory = Inventory {
            creature: kobold,
            ..Inventory::default()
        };
        inventory.give(item, StackLimits::NONE).unwrap();
        inventory.check_wear(id(1))
    };
    assert_eq!(
        wearing(gauntlets(Fingers::Fingered)),
        Err(WearRefused::WrongFingers)
    );
    assert!(wearing(gauntlets(Fingers::Mitten)).is_ok());
    // A ring goes on and off under demi-gauntlets.
    let mut inventory = carrying(&[ItemKind::DemiGauntlets, ItemKind::Ring]);
    inventory.wear(id(1)).unwrap();
    inventory.wear(id(2)).unwrap();
    assert_eq!(inventory.covered_by(id(2)), None);
}

#[test]
fn bodies_decide_what_fits() {
    use ItemKind::{Barding, TShirt, Trousers};
    let chicken = CreatureKind::Chicken;
    let mut inventory = carrying_for(chicken, &[Trousers, TShirt]);
    assert_eq!(inventory.wear(id(1)), Ok(Fit::Exact));
    // A chicken has wings, not arms.
    assert_eq!(
        inventory.check_wear(id(2)),
        Err(WearRefused::WrongBody(Location::Arms))
    );

    let horse = CreatureKind::Horse;
    let inventory = carrying_for(horse, &[Trousers, Barding]);
    assert_eq!(
        inventory.check_wear(id(1)),
        Err(WearRefused::WrongBody(Location::Legs))
    );
    assert_eq!(inventory.check_wear(id(2)), Ok(Fit::Exact));

    let centaur = CreatureKind::Centaur;
    let inventory = carrying_for(centaur, &[Trousers, TShirt]);
    assert!(inventory.check_wear(id(1)).is_err());
    assert_eq!(inventory.check_wear(id(2)), Ok(Fit::Exact));
}

#[test]
fn a_layer_a_place_doesnt_take_is_refused() {
    // A chicken's legs take skin and middle, not outer.
    let chicken = CreatureKind::Chicken;
    let inventory = carrying_for(chicken, &[ItemKind::FullArmour]);
    assert!(matches!(
        inventory.check_wear(id(1)),
        Err(WearRefused::WrongBody(_))
    ));
}

/// Made for a medium creature, with this kind's range.
fn medium(kind: ItemKind) -> Made {
    Made::for_body(kind, Size::Medium, Some(10), false)
}

#[test]
fn shoes_fit_one_size_smaller_loosely_and_never_larger() {
    let shoes = medium(ItemKind::Shoes);
    assert_eq!(shoes.fit(false, Size::Medium, None), Some(Fit::Exact));
    assert_eq!(shoes.fit(false, Size::Small, None), Some(Fit::Loose(1)));
    assert_eq!(shoes.fit(false, Size::Tiny, None), None);
    assert_eq!(shoes.fit(false, Size::Large, None), None);
}

#[test]
fn a_belt_adjusts() {
    let belt = medium(ItemKind::Belt);
    let adjustable = ItemKind::Belt.def().wear.unwrap().adjustable;
    assert!(adjustable);
    assert_eq!(belt.fit(adjustable, Size::Small, None), Some(Fit::Exact));
    assert_eq!(belt.fit(adjustable, Size::Huge, None), Some(Fit::Exact));
    assert_eq!(belt.fit(adjustable, Size::Giant, None), None);
}

#[test]
fn magic_reaches_further_the_better_it_is() {
    let magic = Made {
        magical: true,
        ..medium(ItemKind::Shoes)
    };
    let fit = |size, quality| magic.fit(false, size, Some(quality));
    // Its normal range is what is left past the reach.
    assert_eq!(fit(Size::Large, Quality::Uncommon), Some(Fit::Exact));
    assert_eq!(fit(Size::Huge, Quality::Uncommon), None);
    assert_eq!(fit(Size::Small, Quality::Uncommon), Some(Fit::Exact));
    assert_eq!(fit(Size::Huge, Quality::Rare), Some(Fit::Exact));
    assert_eq!(fit(Size::Giant, Quality::Rare), None);
    assert_eq!(fit(Size::Giant, Quality::Epic), Some(Fit::Exact));
    assert_eq!(
        Made {
            size: Size::Tiny,
            ..magic
        }
        .fit(false, Size::Giant, Some(Quality::Artifact)),
        Some(Fit::Exact)
    );
    // Common is too plain for magic: shoes as usual.
    assert_eq!(fit(Size::Small, Quality::Common), Some(Fit::Loose(1)));
}

/// Worn by a medium human.
fn wearing(item: Item) -> Inventory {
    let mut inventory = Inventory::default();
    inventory.worn.push(item);
    inventory
}

fn with_favour(favour: Option<Favour>) -> Item {
    Item {
        favour,
        ..Item::new(id(1), ItemKind::Hat)
    }
}

#[test]
fn blessed_stops_others_and_cursed_stops_everyone() {
    let may = |favour, by| wearing(with_favour(favour)).may_remove(id(1), by);
    assert!(may(None, Remover::Wearer));
    assert!(may(None, Remover::Other));
    assert!(may(Some(Favour::Blessed), Remover::Wearer));
    assert!(!may(Some(Favour::Blessed), Remover::Other));
    assert!(!may(Some(Favour::Cursed), Remover::Wearer));
    assert!(!may(Some(Favour::Cursed), Remover::Other));
}

#[test]
fn a_curse_only_holds_what_is_worn() {
    let mut inventory = Inventory::default();
    inventory
        .give(with_favour(Some(Favour::Cursed)), StackLimits::NONE)
        .unwrap();
    assert!(inventory.may_remove(id(1), Remover::Wearer));
}

#[test]
fn magic_is_found_out_only_when_it_was_needed() {
    let body = Inventory::default().creature.def().size;
    let shoes = |size: Size, quality| Item {
        made: Made {
            magical: true,
            ..Made::for_body(ItemKind::Shoes, size, None, false)
        },
        quality: Some(quality),
        ..Item::new(id(1), ItemKind::Shoes)
    };
    // Its own size: nothing to give it away.
    let mut own = wearing(shoes(body, Quality::Rare));
    assert!(!own.magic_found_out(id(1)));
    assert!(!own.worn[0].known.magic);
    // Too small for the wearer: only magic could do that.
    let mut small = wearing(shoes(body.stepped(-1), Quality::Rare));
    assert!(small.magic_found_out(id(1)));
    assert!(small.worn[0].known.magic);
    // Loose anyway, out of reach of poor magic: nothing changes.
    let mut plain = wearing(shoes(body.stepped(1), Quality::Common));
    assert!(!plain.magic_found_out(id(1)));
}

#[test]
fn hidden_secrets_arent_shown_until_found_out() {
    let secret = Item {
        made: Made {
            magical: true,
            ..Made::default()
        },
        favour: Some(Favour::Cursed),
        ..Item::new(id(1), ItemKind::Hat)
    };
    let mut hidden = wearing(secret.clone());
    hidden.hide_secrets();
    assert!(!hidden.worn[0].made.magical);
    assert_eq!(hidden.worn[0].favour, None);
    // One found out says nothing of the other.
    let mut known = wearing(secret);
    known.favour_found_out(id(1));
    known.hide_secrets();
    assert!(!known.worn[0].made.magical);
    assert_eq!(known.worn[0].favour, Some(Favour::Cursed));
}

#[test]
fn only_wearables_better_than_common_roll_magic() {
    let rolled = |kind, quality| {
        (0..2000).any(|n| {
            let mut item = Item {
                quality,
                ..Item::new(id(n), kind)
            };
            item.roll_secrets();
            item.made.magical
        })
    };
    assert!(rolled(ItemKind::Hat, Some(Quality::Uncommon)));
    assert!(!rolled(ItemKind::Hat, Some(Quality::Common)));
    assert!(!rolled(ItemKind::Stone, Some(Quality::Rare)));
}

#[test]
fn most_things_are_neither_blessed_nor_cursed_and_none_both() {
    let favours: Vec<_> = (0..2000)
        .map(|n| {
            let mut item = Item::new(id(n), ItemKind::Hat);
            item.roll_secrets();
            item.favour
        })
        .collect();
    let count = |one| favours.iter().filter(|&&f| f == one).count();
    assert!(count(None) > 1600);
    assert!(count(Some(Favour::Blessed)) > 0);
    assert!(count(Some(Favour::Cursed)) > 0);
}

#[test]
fn magic_doesnt_change_the_body() {
    let chicken = CreatureKind::Chicken;
    let mut inventory = Inventory {
        creature: chicken,
        ..Inventory::default()
    };
    let shirt = Item {
        made: Made {
            magical: true,
            ..Made::default()
        },
        ..Item::new(id(1), ItemKind::TShirt)
    };
    inventory.give(shirt, StackLimits::NONE).unwrap();
    assert_eq!(
        inventory.check_wear(id(1)),
        Err(WearRefused::WrongBody(Location::Arms))
    );
}

#[test]
fn a_worn_bag_keeps_its_grid_and_its_contents() {
    use ItemKind::{Backpack, Ring};
    let mut inventory = carrying(&[Backpack, Ring]);
    inventory
        .move_item(id(2), Holder::Inside(id(1)), 0, 0, Rotation::None)
        .unwrap();
    inventory.wear(id(1)).unwrap();
    assert!(inventory.is_worn(id(1)));
    assert!(inventory.find(id(2)).is_some());
    assert_eq!(inventory.grid_size(Holder::Inside(id(1))), Some((10, 12)));
    // Things go in and out of it while it is worn.
    assert_eq!(
        inventory.move_item(id(2), Holder::Inventory, 0, 0, Rotation::None),
        Ok(())
    );
    assert_eq!(
        inventory.move_item(id(2), Holder::Inside(id(1)), 1, 1, Rotation::None),
        Ok(())
    );
    // It can't go inside what it holds, worn or not, and weighs the same.
    assert_eq!(inventory.total_weight(), 1200 + 10);
}

#[test]
fn a_bag_comes_off_with_what_is_in_it() {
    use ItemKind::{Backpack, Ring};
    let mut inventory = carrying(&[Backpack, Ring]);
    inventory
        .move_item(id(2), Holder::Inside(id(1)), 0, 0, Rotation::None)
        .unwrap();
    inventory.wear(id(1)).unwrap();
    take_off(&mut inventory, 1).unwrap();
    assert!(!inventory.is_worn(id(1)));
    assert_eq!(inventory.find(id(1)).unwrap().contents.len(), 1);
}

#[test]
fn changing_body_takes_everything_off_and_gives_back_what_doesnt_fit() {
    use ItemKind::{FullArmour, Ring, Stone};
    let mut inventory = carrying(&[FullArmour, Ring, Stone]);
    inventory.wear(id(1)).unwrap();
    // A chicken's 2 × 2 inventory: the armour is too big and the ring goes first.
    let lost = inventory.change_body(CreatureKind::Chicken, StackLimits::NONE);
    assert!(inventory.worn.is_empty());
    assert_eq!(inventory.size, (2, 2));
    assert_eq!(lost.len() + inventory.items.len(), 3);
    assert!(lost.iter().any(|item| item.kind == FullArmour));
}

#[test]
fn a_picture_shows_what_is_drawn_and_not_what_drawn_things_cover() {
    use crate::item_form::give_form;
    use ItemKind::{FullArmour, Gambeson, Surcoat};
    let mut inventory = carrying(&[Gambeson, FullArmour, Surcoat]);
    for placed in inventory.items.iter_mut() {
        give_form(&mut placed.item, None, &mut || id(99));
    }
    // Nothing drawn is worn yet.
    inventory.wear(id(1)).unwrap();
    assert!(inventory.shown().is_empty());
    take_off(&mut inventory, 1).unwrap();
    inventory.wear(id(2)).unwrap();
    inventory.wear(id(3)).unwrap();
    // The harness shows, under a surcoat that isn't drawn yet.
    let shown = inventory.shown();
    assert_eq!(shown.len(), 1);
    assert!(matches!(shown[0].form, ItemForm::Set(_)));
}

/// A helmet for a creature of this kind, its face closed or open.
fn helmet_for(kind: CreatureKind, closed: bool) -> Inventory {
    use crate::armour::head::{self, HeadKind};
    let form = (0..)
        .map(|seed| {
            let mut rng = crate::rng::Seed::new(seed).rng();
            head::roll(&mut rng, HeadKind::Helmet, None, Quality::Common)
        })
        .find(|form| form.face.closed() == closed)
        .expect("both kinds of face are rolled");
    let mut inventory = carrying_for(kind, &[]);
    let def = kind.def();
    let helmet = Item {
        made: Made::for_body(ItemKind::Helmet, def.size, def.fingers(), false),
        form: Some(ItemForm::Head(form)),
        ..Item::new(id(1), ItemKind::Helmet)
    };
    inventory.give(helmet, StackLimits::NONE).expect("room");
    inventory
}

#[test]
fn closed_helmets_dont_go_over_snouts_or_horns() {
    let wears = |kind, closed| helmet_for(kind, closed).check_wear(id(1));
    assert!(wears(HUMAN, true).is_ok());
    assert!(wears(CreatureKind::Kobold, false).is_ok());
    for kind in [
        CreatureKind::Kobold,
        CreatureKind::Lizardfolk,
        CreatureKind::Minotaur,
        CreatureKind::Birdfolk,
    ] {
        assert_eq!(
            wears(kind, true),
            Err(WearRefused::WrongBody(Location::Head)),
            "{kind:?}"
        );
    }
}
