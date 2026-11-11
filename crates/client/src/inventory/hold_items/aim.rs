//! What letting go of what is held, where it is, would do.

use bevy::prelude::*;
use shared::item::StackLimits;

use super::super::hold::{Held, Target};
use super::super::wear_note::try_wearing;
use super::super::{Carried, Holding};
use super::{Under, worth_wearing};
use crate::common::{GROUND, GameView, HeldGift, HeldOverGifts, HeldOverText};
use crate::dom::HeldOver;
use crate::widgets::CELL;

/// What letting go of `held` at `at`, over this, would do. Says too whose
/// grid of stock it is over, and what text it would be written into.
pub(super) fn aim_at(
    held: Held,
    (at, over): (Option<Vec2>, HeldOver),
    holding: &Holding,
    carried: &Carried,
    under: &Under,
    view: &State<GameView>,
    (over_gifts, over_text): (&mut ResMut<HeldOverGifts>, &mut ResMut<HeldOverText>),
) -> Target {
    let Under {
        grids,
        marks,
        parents,
        stacks,
        god,
    } = under;
    let from_stock = holding.stock.as_ref().map(|(with, _)| *with);
    // One of our grids, and the cell of it the pointer is in.
    let cell = over.cell.and_then(|(grid, x, y)| {
        let owner = grids.get(grid).ok()?;
        Some((owner.holder, x, y))
    });
    // Only what is in front counts: the bin destroys things.
    // The thing in front, or the nearest thing it is in, that is somewhere to
    // let go: markup nests what it shows (a row's words are in the row).
    let front = over.front.and_then(|front| {
        let up = std::iter::once(front).chain(parents.iter_ancestors(front));
        up.filter_map(|entity| marks.get(entity).ok()).find(
            |(bin, drop, body, row, gifts, text)| {
                *bin || *drop || *body || row.is_some() || gifts.is_some() || text.is_some()
            },
        )
    });
    let (on_bin, on_drop, on_body, on_row, on_gifts, on_text) = front
        .map(|(is_bin, is_drop, is_body, row, gifts, text)| {
            (
                is_bin,
                is_drop,
                is_body,
                row.map(|row| row.0),
                gifts.map(|gifts| gifts.0),
                text.map(|text| text.0),
            )
        })
        .unwrap_or_default();
    let on_row = on_row.or(on_gifts);
    // Something of theirs can only go back to them.
    let on_gifts = on_gifts.filter(|&to| from_stock.is_none_or(|from| from == to));
    over_gifts.0 = on_gifts.zip(at).map(|(to, at)| HeldGift {
        to,
        item: held.id,
        rotation: held.rotation,
        corner: at - (Vec2::new(held.grip.0 as f32, held.grip.1 as f32) + 0.5) * CELL,
    });
    // Written into the box: the item itself stays where it was.
    over_text.set_if_neq(HeldOverText(on_text));
    if on_text.is_some() {
        return Target::Nowhere;
    }
    if let Some((with, item)) = &holding.stock {
        let from_ground = *with == GROUND;
        if on_drop {
            // On the ground already, or someone else's.
            return Target::Drop { allowed: false };
        }
        if from_ground && on_bin {
            return Target::Bin;
        }
        if from_ground && on_body {
            // Only the top of a stack goes on.
            let tried = try_wearing(&carried.0, (held.id, Some(item)), god.0, stacks.0);
            return Target::Body {
                allowed: worth_wearing(tried),
            };
        }
        if on_bin || on_body || on_row.is_some() {
            return Target::Nowhere;
        }
        // Only what is on the ground can go on a stack, not someone's stock.
        let limits = if from_ground {
            stacks.0
        } else {
            StackLimits::NONE
        };
        return super::super::hold::target_for_new(&carried.0, item, held, cell, limits);
    }
    if on_drop {
        // Only sites have ground.
        return Target::Drop {
            allowed: *view.get() == GameView::Dungeon,
        };
    }
    if on_bin {
        // In it already: nothing happens.
        return match carried.0.in_bin(held.id) {
            true => Target::Nowhere,
            false => Target::Bin,
        };
    }
    if let Some(someone) = on_row {
        return Target::Someone(someone);
    }
    if on_body {
        // On it already: nothing happens, whatever it is over.
        if carried.0.is_worn(held.id) {
            return Target::Nowhere;
        }
        // What is in its way comes off (owner, 2026-11-07).
        let tried = try_wearing(&carried.0, (held.id, None), god.0, stacks.0);
        return Target::Body {
            allowed: worth_wearing(tried),
        };
    }
    super::super::hold::target(&carried.0, held, cell, stacks.0)
}
