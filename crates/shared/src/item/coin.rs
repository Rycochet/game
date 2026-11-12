//! Coins: copper, silver and gold, each 100 of one worth one of the next
//! (owner, 2026-11-12). Real items with weight; stacks of them turn into the
//! next kind as they pass 99 (`inventory/coins.rs`).

use crate::id::Id;
use crate::item::{Item, ItemKind, MadeAt};

/// How many of one coin make one of the next.
pub const COINS_PER_NEXT: u32 = 100;

/// The coins, cheapest first.
pub const COINS: [ItemKind; 3] = [
    ItemKind::CopperCoin,
    ItemKind::SilverCoin,
    ItemKind::GoldCoin,
];

impl ItemKind {
    /// What one is worth in copper, for coins.
    pub const fn coin_value(self) -> Option<u64> {
        match self {
            Self::CopperCoin => Some(1),
            Self::SilverCoin => Some(COINS_PER_NEXT as u64),
            Self::GoldCoin => Some(COINS_PER_NEXT as u64 * COINS_PER_NEXT as u64),
            _ => None,
        }
    }

    /// The coin 100 of these turn into: none for gold, or what isn't a coin.
    pub const fn next_coin(self) -> Option<ItemKind> {
        match self {
            Self::CopperCoin => Some(Self::SilverCoin),
            Self::SilverCoin => Some(Self::GoldCoin),
            _ => None,
        }
    }
}

/// The fewest coins worth `copper`, as stacks of no more than `most` (at least
/// 1): gold first, then silver, then copper. 12,345 copper is 1 gold, 23 silver
/// and 45 copper; gold past `most` is in more than one stack.
pub fn coin_stacks(copper: u64, most: u32) -> Vec<(ItemKind, u32)> {
    let most = most.max(1) as u64;
    let mut left = copper;
    let mut stacks = Vec::new();
    for kind in COINS.into_iter().rev() {
        let value = kind.coin_value().expect("every one is a coin");
        let mut count = left / value;
        left %= value;
        while count > 0 {
            let here = count.min(most);
            stacks.push((kind, here as u32));
            count -= here;
        }
    }
    stacks
}

/// Coins worth `copper` as items, in the stacks `coin_stacks` gives, each stack
/// one item with a count and a new id: for the server, which alone makes ids.
pub fn coin_items(
    copper: u64,
    most: u32,
    made_at: Option<MadeAt>,
    new_id: &mut impl FnMut() -> Id,
) -> Vec<Item> {
    coin_stacks(copper, most)
        .into_iter()
        .map(|(kind, count)| Item {
            made_at,
            count,
            ..Item::new(new_id(), kind)
        })
        .collect()
}
