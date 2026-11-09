//! Small helpers for the tables of weighted choices every garment rolls from.

/// Whether `wanted` is in the list with a weight above 0, so a roll could give it.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}

/// `weight` when `yes`, otherwise 0: a choice only some earlier ones allow.
pub(super) const fn when(yes: bool, weight: u64) -> u64 {
    if yes { weight } else { 0 }
}
