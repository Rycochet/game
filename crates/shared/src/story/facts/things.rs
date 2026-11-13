//! Facts about things and the world: what kind of item, how many creatures,
//! the season and weather, and the reward.

use serde::{Deserialize, Serialize};

use super::super::fact::fact;

fact! {
    /// What kind of thing a quest's item is, as the story speaks of it.
    ItemCategory as "item-category" {
        Ring = "ring",
        Necklace = "necklace",
        Brooch = "brooch",
        Locket = "locket",
        Weapon = "weapon",
        Tool = "tool",
        Letter = "letter",
        Book = "book",
        Deed = "deed",
        Box = "box",
        Instrument = "instrument",
        Toy = "toy",
        Remedy = "remedy",
        Herb = "herb",
        Goods = "goods",
        Coin = "coin",
    }
}

impl ItemCategory {
    /// Something a person might have owned and left behind (a keepsake).
    pub const fn is_keepsake(self) -> bool {
        matches!(
            self,
            Self::Ring
                | Self::Necklace
                | Self::Brooch
                | Self::Locket
                | Self::Weapon
                | Self::Tool
                | Self::Letter
                | Self::Book
                | Self::Box
                | Self::Instrument
                | Self::Toy
        )
    }
}

fact! {
    /// A part of a creature a quest asks to have brought back (owner: one
    /// natural to it, which only quests drop): what the story calls it.
    BodyPart as "body-part" {
        Heart = "heart",
        Liver = "liver",
        Tooth = "tooth",
        Fang = "fang",
        Claw = "claw",
        Fingernail = "fingernail",
        Toenail = "toenail",
        Tail = "tail",
        Horn = "horn",
        Tusk = "tusk",
        Ear = "ear",
        Feather = "feather",
        Hoof = "hoof",
        Scale = "scale",
    }
}

/// What a "Collect" step asks to have brought back: a part of creatures of a
/// kind ("kobold teeth").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Remains {
    pub kind: crate::creature::CreatureKind,
    pub part: BodyPart,
}

fact! {
    /// Roughly how many creatures: what a giver would say, not a number.
    CountBand as "count" {
        One = "one",
        Few = "few",
        Many = "many",
    }
}

impl CountBand {
    /// The band a real count falls in: 1, up to 5, more.
    pub const fn of(count: u32) -> Self {
        match count {
            0 | 1 => Self::One,
            2..=5 => Self::Few,
            _ => Self::Many,
        }
    }

    /// A number in the band, for the plural forms of the words for creatures.
    pub const fn plural_count(self) -> i64 {
        match self {
            Self::One => 1,
            Self::Few => 3,
            Self::Many => 8,
        }
    }
}

fact! {
    /// The planet's season where the quest is. Only checkable ones are told,
    /// and only in higher-reward quests (owner).
    Season as "season" {
        Spring = "spring",
        Summer = "summer",
        Autumn = "autumn",
        Winter = "winter",
    }
}

fact! {
    /// The weather or climate making the way hard. Checkable, as Season.
    Weather as "weather" {
        Snow = "snow",
        Rain = "rain",
        Heat = "heat",
        Storms = "storms",
        Floods = "floods",
    }
}

/// One part of a quest's real reward, which the text names (owner).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardPart {
    /// Coins worth this much copper, told as the fewest coins.
    Coins(u64),
    /// An item of this kind.
    Item(ItemCategory),
    /// Training in a skill. Which skill isn't worded yet: skills' names aren't
    /// in the language files.
    Training,
    /// A favour owed.
    Favour,
}
