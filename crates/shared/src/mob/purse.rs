//! How much money a mob starts with: one amount in copper, rolled on its own, which
//! the server turns into the fewest coins (`item::coin_items`).

use crate::creature::CreatureKind;
use crate::rng::Seed;

/// Which roll of a mob's own is its money's, clear of its gear's places.
const COIN_ROLL: i64 = 1100;

/// For each kind with money: the chance in 100 of having any, and the least and
/// most it then has, in copper. First guesses.
fn purse_of(kind: CreatureKind) -> Option<(u64, u64, u64)> {
    match kind {
        CreatureKind::Beggar => Some((40, 1, 30)),
        CreatureKind::Kobold => Some((30, 5, 150)),
        CreatureKind::Goblin => Some((30, 5, 200)),
        CreatureKind::Orc => Some((30, 20, 800)),
        CreatureKind::Giant => Some((30, 100, 5000)),
        _ => None,
    }
}

/// How much money, in copper, a mob of this kind starts with, by its rolls.
pub(super) fn roll_coins(roll: &impl Fn(i64) -> u64, kind: CreatureKind) -> u64 {
    let Some((chance, least, most)) = purse_of(kind) else {
        return 0;
    };
    let mut rng = Seed::new(roll(COIN_ROLL)).rng();
    if rng.range_u64(0, 100) >= chance {
        return 0;
    }
    rng.range_u64(least, most + 1)
}
