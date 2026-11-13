//! Reset and Random above the triangles, changing only the dots: Reset puts
//! each back in the middle of its triangle, Random each somewhere at random in
//! its own (owner, 2026-11-13). The turn stays as it is.

use bevy::prelude::*;
use shared::rng::{Rng, Seed};
use shared::sheet::{Dot, WEIGHT_TOTAL};

use super::draft::Draft;
use super::screen::FORM;
use crate::dom::{UiEvent, UiValue};

/// A dot anywhere in its triangle, every place as likely: two weights drawn,
/// folded back in when they come to more than the whole, the third what is
/// left, so the three always add up to `WEIGHT_TOTAL`.
pub(super) fn random_dot(rng: &mut Rng) -> Dot {
    let whole = WEIGHT_TOTAL as u64;
    let (mut first, mut second) = (rng.range_u64(0, whole + 1), rng.range_u64(0, whole + 1));
    if first + second > whole {
        (first, second) = (whole - first, whole - second);
    }
    let (first, second) = (first as u16, second as u16);
    Dot([first, second, WEIGHT_TOTAL - first - second])
}

/// Where Random's dots come from: seeded from the clock once, then rolling on.
#[derive(Default)]
pub(super) struct DotRolls(Option<Rng>);

pub(super) fn press_triangle_buttons(
    mut events: MessageReader<UiEvent>,
    mut draft: ResMut<Draft>,
    mut rolls: Local<DotRolls>,
    time: Res<Time<Real>>,
) {
    for event in events.read() {
        if !matches!(event.value, UiValue::Pressed) {
            continue;
        }
        match event.id.parts() {
            (FORM, "reset_triangles") => draft.build.dots = [Dot::CENTRE; 3],
            (FORM, "random_triangles") => {
                let rng = rolls
                    .0
                    .get_or_insert_with(|| Seed::new(time.elapsed().as_nanos() as u64).rng());
                draft.build.dots = [(); 3].map(|_| random_dot(rng));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
