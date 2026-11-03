//! Rings: `RingForm`, its band (`Profile`, `BandWidth`, `Pattern`) and its settings (`Setting`, `Mount`).

use serde::{Deserialize, Serialize};

use crate::creature::Size;
use crate::item::Item;
use crate::item_form::words::{counted, gem_material, plural};
use crate::item_form::{Cut, ItemForm};
use crate::material::Material;

/// The shape of a ring's band, across.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Profile {
    Domed,
    Flat,
    Knife,
    Round,
    Square,
}

/// How wide a ring's band is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BandWidth {
    Thin,
    Medium,
    Wide,
}

/// What has been done to the surface of a ring's band.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pattern {
    Engraved,
    Hammered,
    Plain,
    Twisted,
}

/// How a setting holds its gem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mount {
    Bezel,
    Claws,
}

/// A place on a ring for a gem, which stays when the gem is taken out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    /// How far round the band from the top, in 256ths of a turn.
    pub angle: u8,
    pub size: Size,
    pub mount: Mount,
    /// A whole item, with its own id, or nothing.
    pub gem: Option<Item>,
}

impl Setting {
    /// Whether this could be set here: a cut gem no more than one size too big.
    pub fn takes(&self, item: &Item) -> bool {
        let cut = match &item.form {
            Some(ItemForm::Gem(gem)) => gem.cut != Cut::Uncut,
            _ => false,
        };
        cut && item.made.size <= self.size.stepped(1)
    }
}

/// One ring.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingForm {
    pub metal: Material,
    pub profile: Profile,
    pub width: BandWidth,
    pub pattern: Pattern,
    pub settings: Vec<Setting>,
}

impl RingForm {
    /// Its metal and what is in its settings, as it is now.
    pub(super) fn name(&self) -> String {
        let mut stones: Vec<(Material, usize)> = Vec::new();
        let mut empty = 0;
        for setting in &self.settings {
            match setting.gem.as_ref().and_then(gem_material) {
                Some(stone) => match stones.iter_mut().find(|(other, _)| *other == stone) {
                    Some((_, count)) => *count += 1,
                    None => stones.push((stone, 1)),
                },
                None => empty += 1,
            }
        }
        let mut parts: Vec<String> = stones
            .into_iter()
            .map(|(stone, count)| counted(count, stone.name(), plural(stone)))
            .collect();
        if empty > 0 {
            parts.push(counted(empty, "Empty Setting", "Empty Settings".to_owned()));
        }
        let ring = format!("{} Ring", self.metal.name());
        match parts.split_last() {
            None => ring,
            Some((only, [])) => format!("{ring} with {only}"),
            Some((last, others)) => format!("{ring} with {} and {last}", others.join(", ")),
        }
    }
}
