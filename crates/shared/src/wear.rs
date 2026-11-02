//! Wearing things: the rules both sides check clothes by. See
//! `docs/context/shared/creature.md`.
//!
//! An item has one layer and the locations it covers. It fits a body if every one
//! of them exists there and takes that layer. It can only go on or come off if
//! nothing on a higher layer is at any of its locations.

use crate::creature::{CreatureKind, Layer, Location, Size};
use crate::id::Id;
use crate::item::{Inventory, Item, ItemDef, Made};

/// How an item fits its wearer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Exact,
    /// This many sizes off, with a penalty when penalties exist.
    Loose(u8),
}

/// Why something can't be put on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WearRefused {
    /// It isn't carried.
    NoSuchItem,
    AlreadyWorn,
    NotClothing,
    /// This body has no such place, or that place doesn't take this layer.
    WrongBody(Location),
    /// Made for a much bigger or smaller creature.
    WrongSize,
    /// Made for another number of fingers.
    WrongFingers,
    /// Something on the same layer is in the way.
    Clash(Id),
    /// Something on a higher layer covers one of its locations.
    Covered(Id),
}

impl Made {
    /// How this fits a creature of this size: `None` if it doesn't.
    pub fn fit(&self, adjustable: bool, size: Size) -> Option<Fit> {
        if self.magical {
            return Some(Fit::Exact);
        }
        if size < self.smallest || size > self.largest {
            return None;
        }
        let off = self.size.apart(size);
        Some(if adjustable || off == 0 {
            Fit::Exact
        } else {
            Fit::Loose(off)
        })
    }
}

/// The locations of a worn item, and its layer.
fn coverage(def: &ItemDef) -> (&'static [Location], Option<Layer>) {
    match def.wear {
        Some(wear) => (wear.locations, Some(wear.layer)),
        None => (&[], None),
    }
}

/// Whether two things cover a location in common.
fn share(a: &[Location], b: &[Location]) -> bool {
    a.iter().any(|location| b.contains(location))
}

impl Inventory {
    pub fn is_worn(&self, id: Id) -> bool {
        self.worn.iter().any(|item| item.id == id)
    }

    /// What, on a higher layer than `layer`, is at any of these locations.
    fn covering(&self, layer: Layer, locations: &[Location], not: Id) -> Option<Id> {
        self.worn
            .iter()
            .filter(|other| other.id != not)
            .find(|other| {
                let (theirs, their_layer) = coverage(&other.kind.def());
                their_layer > Some(layer) && share(theirs, locations)
            })
            .map(|other| other.id)
    }

    /// What is on top of this worn item, stopping it coming off.
    pub fn covered_by(&self, id: Id) -> Option<Id> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        let (locations, layer) = coverage(&item.kind.def());
        self.covering(layer?, locations, id)
    }

    /// Whether this carried item can be put on, and how it would fit.
    pub fn check_wear(&self, id: Id) -> Result<Fit, WearRefused> {
        if self.is_worn(id) {
            return Err(WearRefused::AlreadyWorn);
        }
        let item = self.find(id).ok_or(WearRefused::NoSuchItem)?;
        let wear = item.kind.def().wear.ok_or(WearRefused::NotClothing)?;
        let body = self.creature.def();
        for &location in wear.locations {
            let part = body
                .part(location)
                .ok_or(WearRefused::WrongBody(location))?;
            if !part.layers.contains(&wear.layer) {
                return Err(WearRefused::WrongBody(location));
            }
        }
        let fit = item
            .made
            .fit(wear.adjustable, body.size)
            .ok_or(WearRefused::WrongSize)?;
        let whole_fingers = !wear.one_place && wear.locations.contains(&Location::Fingers);
        if whole_fingers && !item.made.magical && item.made.fingers != body.fingers() {
            return Err(WearRefused::WrongFingers);
        }
        if let Some(other) = self.clash(item) {
            return Err(WearRefused::Clash(other));
        }
        if let Some(other) = self.covering(wear.layer, wear.locations, id) {
            return Err(WearRefused::Covered(other));
        }
        Ok(fit)
    }

    /// What on the same layer is in its way. Things taking one place only get in
    /// each other's way once every place is taken.
    fn clash(&self, item: &Item) -> Option<Id> {
        let wear = item.kind.def().wear?;
        let same_layer = || {
            self.worn.iter().filter(move |other| {
                other
                    .kind
                    .def()
                    .wear
                    .is_some_and(|theirs| theirs.layer == wear.layer)
            })
        };
        for &location in wear.locations {
            let here = || {
                same_layer().filter(move |other| {
                    other
                        .kind
                        .def()
                        .wear
                        .is_some_and(|theirs| theirs.locations.contains(&location))
                })
            };
            let whole = |other: &&Item| {
                other
                    .kind
                    .def()
                    .wear
                    .is_some_and(|theirs| !theirs.one_place)
            };
            if !wear.one_place {
                if let Some(other) = here().next() {
                    return Some(other.id);
                }
            } else if let Some(other) = here().find(whole) {
                return Some(other.id);
            } else {
                let places = self
                    .creature
                    .def()
                    .part(location)
                    .map_or(0, |part| part.places);
                if here().count() >= places as usize {
                    return here().next().map(|other| other.id);
                }
            }
        }
        None
    }

    /// Puts this carried item on, if it can go.
    pub fn wear(&mut self, id: Id) -> Result<Fit, WearRefused> {
        let fit = self.check_wear(id)?;
        let item = self.take(id).ok_or(WearRefused::NoSuchItem)?;
        self.worn.push(item);
        Ok(fit)
    }

    /// How a worn item fits its wearer.
    pub fn worn_fit(&self, id: Id) -> Option<Fit> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        let wear = item.kind.def().wear?;
        item.made.fit(wear.adjustable, self.creature.def().size)
    }

    /// For a character becoming another kind of creature (dev tools): everything
    /// comes off and goes in the new pockets, as far as it fits. Gives back what
    /// didn't.
    pub fn change_body(&mut self, kind: CreatureKind) -> Vec<Item> {
        let old = std::mem::take(self);
        self.creature = kind;
        self.outfitted = old.outfitted;
        self.pockets_size = kind.def().pockets;
        let everything = old
            .pockets
            .into_iter()
            .map(|placed| placed.item)
            .chain(old.worn);
        everything
            .filter_map(|item| self.give(item).err())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::Size;
    use crate::item::{Holder, ItemKind, MoveRefused, Rotation};

    const HUMAN: CreatureKind = CreatureKind::Human;

    fn id(n: u128) -> Id {
        Id::for_test(n)
    }

    /// These kinds, made for this creature, ids from 1, in big pockets.
    fn carrying_for(kind: CreatureKind, kinds: &[ItemKind]) -> Inventory {
        let def = kind.def();
        let mut inventory = Inventory {
            creature: kind,
            pockets_size: (40, 40),
            ..Inventory::default()
        };
        for (index, &item) in kinds.iter().enumerate() {
            let made = Made::for_body(item, def.size, def.fingers(), false);
            let new = Item {
                made,
                ..Item::new(id(index as u128 + 1), item)
            };
            inventory.give(new).expect("room");
        }
        inventory
    }

    fn carrying(kinds: &[ItemKind]) -> Inventory {
        carrying_for(HUMAN, kinds)
    }

    /// Takes it off into a far corner of the pockets.
    fn take_off(inventory: &mut Inventory, n: u128) -> Result<(), MoveRefused> {
        inventory.move_item(id(n), Holder::Pockets, 30, 30, Rotation::None)
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
        assert!(!inventory.destroy(id(1)));
        take_off(&mut inventory, 2).unwrap();
        assert_eq!(take_off(&mut inventory, 1), Err(MoveRefused::NoRoom));
        assert_eq!(
            inventory.move_item(id(1), Holder::Pockets, 32, 30, Rotation::None),
            Ok(())
        );
    }

    #[test]
    fn a_backpack_and_a_full_suit_cant_be_worn_together() {
        use ItemKind::{Backpack, FullArmour};
        let mut inventory = carrying(&[Backpack, FullArmour]);
        inventory.wear(id(1)).unwrap();
        assert_eq!(inventory.check_wear(id(2)), Err(WearRefused::Clash(id(1))));
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
            ..Item::new(id(1), ItemKind::Gloves)
        };
        let mut inventory = Inventory {
            creature: kobold,
            ..Inventory::default()
        };
        inventory.give(gloves(false)).unwrap();
        assert_eq!(inventory.check_wear(id(1)), Err(WearRefused::WrongFingers));
        let mut magic = Inventory {
            creature: kobold,
            ..Inventory::default()
        };
        magic.give(gloves(true)).unwrap();
        assert_eq!(magic.check_wear(id(1)), Ok(Fit::Exact));
        // Fingerless ones don't care.
        let mut fingerless = carrying_for(kobold, &[ItemKind::FingerlessGloves]);
        assert_eq!(fingerless.wear(id(1)), Ok(Fit::Exact));
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
        assert_eq!(shoes.fit(false, Size::Medium), Some(Fit::Exact));
        assert_eq!(shoes.fit(false, Size::Small), Some(Fit::Loose(1)));
        assert_eq!(shoes.fit(false, Size::Tiny), None);
        assert_eq!(shoes.fit(false, Size::Large), None);
    }

    #[test]
    fn a_belt_adjusts_and_magic_fits_anyone() {
        let belt = medium(ItemKind::Belt);
        let adjustable = ItemKind::Belt.def().wear.unwrap().adjustable;
        assert!(adjustable);
        assert_eq!(belt.fit(adjustable, Size::Small), Some(Fit::Exact));
        assert_eq!(belt.fit(adjustable, Size::Huge), Some(Fit::Exact));
        assert_eq!(belt.fit(adjustable, Size::Giant), None);

        let magic = Made {
            magical: true,
            ..medium(ItemKind::Shoes)
        };
        assert_eq!(magic.fit(false, Size::Giant), Some(Fit::Exact));
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
        inventory.give(shirt).unwrap();
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
            inventory.move_item(id(2), Holder::Pockets, 0, 0, Rotation::None),
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
        // A chicken's 2 × 2 pockets: the armour is too big and the ring goes first.
        let lost = inventory.change_body(CreatureKind::Chicken);
        assert!(inventory.worn.is_empty());
        assert_eq!(inventory.pockets_size, (2, 2));
        assert_eq!(lost.len() + inventory.pockets.len(), 3);
        assert!(lost.iter().any(|item| item.kind == FullArmour));
    }
}
