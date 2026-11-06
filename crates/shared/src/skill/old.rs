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
