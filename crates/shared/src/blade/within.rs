//! Finding lengths and rows that fit a request's limits.

use super::table::Row;
use super::{BladeForm, BladeWidth, Build, CELL_LENGTH, Span, overlap, roll_properties};
use crate::creature::Size;
use crate::item::{MAX_SHAPE_SIDE, Quality, Shape};
use crate::item_form::Limits;
use crate::material::Material;
use crate::rng::Rng;

/// The lengths at its ordinary size that keep a weapon scaled to `size` within limits.
/// `None` if none do.
pub(super) fn lengths_within(limits: &Limits, size: Size) -> Option<Span> {
    let long_at = |cells: u8| Shape::rectangle(1, cells).scaled(size).height;
    let longest = match limits.fits {
        Some((one, other)) => {
            let room = one.max(other);
            (1..=MAX_SHAPE_SIDE)
                .rev()
                .find(|&cells| long_at(cells) <= room)?
        }
        None => MAX_SHAPE_SIDE,
    };
    let shortest = match limits.at_least_long {
        Some(least) => (1..=MAX_SHAPE_SIDE).find(|&cells| long_at(cells) >= least)?,
        None => 1,
    };
    let cell = CELL_LENGTH as u16;
    let from = match shortest {
        1 => 0,
        _ => shortest as u16 * cell - cell / 2,
    };
    let to = match longest {
        MAX_SHAPE_SIDE => u16::MAX,
        _ => longest as u16 * cell + cell / 2,
    };
    (from < to).then_some((from, to))
}

/// The handles from which some blade in `blade` gives a length within `lengths`.
pub(super) fn handles_within(
    build: Build,
    blade: Span,
    handle: Span,
    lengths: Span,
) -> Option<Span> {
    match build {
        Build::Side => overlap(handle, lengths),
        Build::Along => {
            let from = lengths.0.saturating_sub(blade.1 - 1);
            let to = lengths.1.saturating_sub(blade.0);
            overlap(handle, (from, to))
        }
    }
}

/// The blades that, with this handle, give a length within `lengths`.
pub(super) fn blades_within(build: Build, blade: Span, handle: u16, lengths: Span) -> Option<Span> {
    match build {
        Build::Side => Some(blade),
        Build::Along => {
            let from = lengths.0.saturating_sub(handle);
            let to = lengths.1.saturating_sub(handle);
            overlap(blade, (from, to))
        }
    }
}

/// The first number in a span that passes, given that all those after it pass too, or
/// the span's end if none do.
pub(super) fn first_passing(span: Span, passes: impl Fn(u16) -> bool) -> u16 {
    let (mut low, mut high) = span;
    while low < high {
        let middle = low + (high - low) / 2;
        if passes(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// The blades in a span that keep this weapon's weight within limits: a longer one is
/// never lighter.
pub(super) fn blades_by_weight(
    form: &BladeForm,
    blade: Span,
    size: Size,
    limits: &Limits,
) -> Option<Span> {
    let grams = |length: u16| {
        let longer = BladeForm {
            blade: length,
            ..*form
        };
        longer.grams_at(size)
    };
    let from = match limits.least_grams {
        Some(least) => first_passing(blade, |length| grams(length) >= least),
        None => blade.0,
    };
    let to = match limits.most_grams {
        Some(most) => first_passing(blade, |length| grams(length) > most),
        None => blade.1,
    };
    (from < to).then_some((from, to))
}

/// One row within limits, at a size. Lengths are cut down to what fits rather than
/// rolled again. If its weight still can't fit, it tries the lightest it could be made
/// of before giving up on this row.
pub(super) fn roll_row_within(
    rng: &mut Rng,
    row: &Row,
    quality: Quality,
    limits: &Limits,
    size: Size,
) -> Option<BladeForm> {
    let lengths = lengths_within(limits, size)?;
    let places: Vec<((Span, Span), u64)> = row
        .places()
        .filter_map(|(blade, handle)| {
            let handle = handles_within(row.build, blade, handle, lengths)?;
            Some(((blade, handle), 1))
        })
        .collect();
    if places.is_empty() {
        return None;
    }
    let (blade, handle) = rng.pick_weighted(&places);
    let handle = rng.range_u64(handle.0 as u64, handle.1 as u64) as u16;
    let blade = blades_within(row.build, blade, handle, lengths)?;
    let mut form = roll_properties(rng, row, quality, blade.0, handle);
    let span = blades_by_weight(&form, blade, size, limits).or_else(|| {
        form.width = row.width.first().copied().unwrap_or(BladeWidth::Narrow);
        form.blade_metal = Material::Iron;
        form.handle_material = Material::Wood;
        blades_by_weight(&form, blade, size, limits)
    })?;
    form.blade = rng.range_u64(span.0 as u64, span.1 as u64) as u16;
    limits
        .allows(form.shape().scaled(size), form.grams_at(size))
        .then_some(form)
}

/// Picks by the usual weights among what the row allows. A row may allow only things
/// that are usually never seen on such a weapon, so those then count equally.
pub(super) fn pick<T: Copy + PartialEq>(rng: &mut Rng, allowed: &[T], usual: &[(T, u64)]) -> T {
    let open = |option: &T| allowed.is_empty() || allowed.contains(option);
    let options: Vec<(T, u64)> = usual
        .iter()
        .copied()
        .filter(|(option, weight)| *weight > 0 && open(option))
        .collect();
    if options.is_empty() {
        let only: Vec<(T, u64)> = allowed.iter().map(|option| (*option, 1)).collect();
        rng.pick_weighted(&only)
    } else {
        rng.pick_weighted(&options)
    }
}
