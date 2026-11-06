//! Saves from before the tree kept five skills by name with a level (`Climbing: 1`):
//! where each went.

/// The node an old skill's name became, if it is one.
pub(super) fn old_skill(name: &str) -> Option<&'static str> {
    Some(match name {
        "Climbing" => "athletics/climbing",
        "Foraging" => "gathering/foraging",
        "Running" => "athletics/running",
        "Stealth" => "stealth/sneaking",
        "Swimming" => "athletics/swimming",
        _ => return None,
    })
}

/// Where a node that moved now is: shields went under melee's defence (owner,
/// 2026-11-06), so all defending practises Defence.
pub(super) fn moved(path: &str) -> Option<&'static str> {
    Some(match path {
        "combat/shield" => "combat/melee/defence",
        "combat/shield/buckler" => "combat/melee/defence/buckler",
        "combat/shield/round" => "combat/melee/defence/roundshield",
        "combat/shield/kite" => "combat/melee/defence/kiteshield",
        "combat/shield/tower" => "combat/melee/defence/towershield",
        _ => return None,
    })
}
