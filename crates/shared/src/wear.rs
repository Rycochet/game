//! Wearing things: the rules both sides check clothes by. See
//! `docs/context/shared/creature.md`.
//!
//! An item has one layer and the locations it covers. It fits a body if every one
//! of them exists there and takes that layer. It can only go on or come off if
//! nothing on a higher layer is at any of its locations.

use crate::creature::{CreatureKind, Layer, Location, Size};
use crate::id::Id;
use crate::item::{Favour, Inventory, Item, ItemDef, Made, Quality};

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

impl WearRefused {
    /// Whether this might only be how it looks from here: magic that hasn't been found
    /// out would fit, so a client lets the server decide.
    pub fn worth_trying(self) -> bool {
        matches!(self, Self::WrongSize | Self::WrongFingers)
    }
}

/// Who is trying to take something off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Remover {
    Wearer,
    Other,
}

impl Made {
    /// How this fits a creature of this size, with its quality deciding how far any
    /// magic reaches: `None` if it doesn't. Past that reach, it is as if it had none.
    pub fn fit(&self, adjustable: bool, size: Size, quality: Option<Quality>) -> Option<Fit> {
        let off = self.size.apart(size);
        let reach = quality.map_or(0, Quality::magic_reach);
        if self.magical && off <= reach && reach > 0 {
            return Some(Fit::Exact);
        }
        if size < self.smallest || size > self.largest {
            return None;
        }
        Some(if adjustable || off == 0 {
            Fit::Exact
        } else {
            Fit::Loose(off)
        })
    }
}

/// Each location a worn item takes, with the layer it takes there: its own layer
/// everywhere, and Middle too where it is padded.
fn coverage(def: &ItemDef) -> Vec<(Location, Layer)> {
    let Some(wear) = def.wear else {
        return Vec::new();
    };
    let own = wear
        .locations
        .iter()
        .map(|&location| (location, wear.layer));
    let padding = wear
        .padded
        .iter()
        .map(|&location| (location, Layer::Middle));
    own.chain(padding).collect()
}

impl Inventory {
    pub fn is_worn(&self, id: Id) -> bool {
        self.worn.iter().any(|item| item.id == id)
    }

    /// What is on a higher layer than any of these, at the same location.
    fn covering(&self, places: &[(Location, Layer)], not: Id) -> Option<Id> {
        self.worn
            .iter()
            .filter(|other| other.id != not)
            .find(|other| {
                coverage(&other.kind.def())
                    .iter()
                    .any(|&(theirs, their_layer)| {
                        places
                            .iter()
                            .any(|&(location, layer)| theirs == location && their_layer > layer)
                    })
            })
            .map(|other| other.id)
    }

    /// What is on top of this worn item, stopping it coming off.
    pub fn covered_by(&self, id: Id) -> Option<Id> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        self.covering(&coverage(&item.kind.def()), id)
    }

    /// Whether this carried item can be put on, and how it would fit.
    pub fn check_wear(&self, id: Id) -> Result<Fit, WearRefused> {
        if self.is_worn(id) {
            return Err(WearRefused::AlreadyWorn);
        }
        let item = self.find(id).ok_or(WearRefused::NoSuchItem)?;
        self.check_wear_new(item)
    }

    /// Whether this could be put on, carried or not, and how it would fit.
    pub fn check_wear_new(&self, item: &Item) -> Result<Fit, WearRefused> {
        let id = item.id;
        let wear = item.kind.def().wear.ok_or(WearRefused::NotClothing)?;
        let body = self.creature.def();
        let places = coverage(&item.kind.def());
        for &(location, layer) in &places {
            let part = body
                .part(location)
                .ok_or(WearRefused::WrongBody(location))?;
            if !part.layers.contains(&layer) {
                return Err(WearRefused::WrongBody(location));
            }
        }
        let fit = item
            .made
            .fit(wear.adjustable, body.size, item.quality)
            .ok_or(WearRefused::WrongSize)?;
        let whole_fingers = !wear.one_place && wear.locations.contains(&Location::Fingers);
        if whole_fingers && !item.is_magical() && item.made.fingers != body.fingers() {
            return Err(WearRefused::WrongFingers);
        }
        if let Some(other) = self.clash(item) {
            return Err(WearRefused::Clash(other));
        }
        if let Some(other) = self.covering(&places, id) {
            return Err(WearRefused::Covered(other));
        }
        Ok(fit)
    }

    /// What on the same layer is in its way. Things taking one place only get in
    /// each other's way once every place is taken.
    fn clash(&self, item: &Item) -> Option<Id> {
        let wear = item.kind.def().wear?;
        for (location, layer) in coverage(&item.kind.def()) {
            let here = || {
                self.worn
                    .iter()
                    .filter(move |other| coverage(&other.kind.def()).contains(&(location, layer)))
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

    /// Puts on something not carried, if it can go on.
    pub fn wear_new(&mut self, item: Item) -> Result<Fit, WearRefused> {
        let fit = self.check_wear_new(&item)?;
        self.worn.push(item);
        Ok(fit)
    }

    /// Whether this person may take this off, by what lies on it: anything not worn
    /// may go. What this says belongs to whoever is acting, and not to `move_item` or
    /// `hand_over`: a body falling, or an ability, isn't someone trying.
    pub fn may_remove(&self, id: Id, by: Remover) -> bool {
        let Some(item) = self.worn.iter().find(|item| item.id == id) else {
            return true;
        };
        match item.favour {
            None => true,
            Some(Favour::Blessed) => by == Remover::Wearer,
            Some(Favour::Cursed) => false,
        }
    }

    /// Finds out that this worn thing is cursed or blessed.
    pub fn favour_found_out(&mut self, id: Id) {
        if let Some(item) = self.worn.iter_mut().find(|item| item.id == id) {
            item.known.favour = true;
        }
    }

    /// Whether putting this on just now was only possible by its magic, which gives
    /// it away. A thing that would fit anyway keeps its secret.
    pub fn magic_found_out(&mut self, id: Id) -> bool {
        let body = self.creature.def();
        let Some(item) = self.worn.iter_mut().find(|item| item.id == id) else {
            return false;
        };
        let Some(wear) = item.kind.def().wear else {
            return false;
        };
        if !item.is_magical() {
            return false;
        }
        let without = Made {
            magical: false,
            ..item.made
        };
        let plain = without.fit(wear.adjustable, body.size, item.quality);
        let with = item.made.fit(wear.adjustable, body.size, item.quality);
        let whole_fingers = !wear.one_place && wear.locations.contains(&Location::Fingers);
        let needed = (plain != with) || (whole_fingers && item.made.fingers != body.fingers());
        if needed {
            item.known.magic = true;
        }
        needed
    }

    /// As a client may see it: see `Item::hide_secrets`.
    pub fn hide_secrets(&mut self) {
        let placed = self.pockets.iter_mut().map(|placed| &mut placed.item);
        for item in placed.chain(&mut self.worn).chain(&mut self.bin) {
            item.hide_secrets();
        }
    }

    /// Takes out something to hand over, with all in it, unless it isn't carried or
    /// is worn under something.
    pub fn hand_over(&mut self, id: Id) -> Option<Item> {
        if self.covered_by(id).is_some() {
            return None;
        }
        self.take(id)
    }

    /// Everything carried and worn, each with all in it, as what is left of a body.
    pub fn spilled(self) -> Vec<Item> {
        let pockets = self.pockets.into_iter().map(|placed| placed.item);
        pockets.chain(self.worn).collect()
    }

    /// How a worn item fits its wearer.
    pub fn worn_fit(&self, id: Id) -> Option<Fit> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        let wear = item.kind.def().wear?;
        item.made
            .fit(wear.adjustable, self.creature.def().size, item.quality)
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
        assert!(!inventory.to_bin(id(1)));
        take_off(&mut inventory, 2).unwrap();
        assert_eq!(take_off(&mut inventory, 1), Err(MoveRefused::NoRoom));
        assert_eq!(
            inventory.move_item(id(1), Holder::Pockets, 32, 30, Rotation::None),
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
        inventory.give(with_favour(Some(Favour::Cursed))).unwrap();
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
