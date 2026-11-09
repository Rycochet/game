//! Trading panels: for now a town shop's, opened by clicking its door, walking
//! there first if the player isn't beside it. What there is comes from the server,
//! which only tells someone in reach. See `docs/context/client/trade.md`.

use bevy::prelude::*;

use crate::common::PointerSystems;
use crate::player::PlayerSystems;
use crate::widgets::WidgetSystems;

// Split into files (`cargo xtask lint` keeps files small).
mod close;
mod open;
mod state;
mod stock;

use close::{close_on_escape, close_when_left};
use open::{ask_again_with_truesight, change_of_mind, follow_clicked_door, open_on_arrival};
use state::{Heading, Keepers, OpenTrade, OpenTrades, beside, front_of};
use stock::{draw_grids, note_gifts, read_server, show_gift_aim};

/// To the left, leaving the player in view.
const AT: Vec2 = Vec2::new(40.0, 120.0);
/// How far along each panel open already puts the next.
const NEXT_AT: Vec2 = Vec2::new(36.0, 36.0);
/// How many cells across a panel's grid is.
const COLUMNS: u8 = 12;
/// The most of the window's height a grid takes, before it scrolls.
const MOST_HEIGHT: Val = Val::Vh(70.0);

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenTrades>()
            .init_resource::<Heading>()
            .init_resource::<Keepers>()
            .add_systems(
                PreUpdate,
                close_on_escape
                    .after(PointerSystems)
                    .before(WidgetSystems::Front),
            )
            // After the steps there have gone, so the server has the player at it.
            .add_systems(PostUpdate, open_on_arrival)
            .add_systems(
                Update,
                (
                    (change_of_mind, follow_clicked_door)
                        .chain()
                        .in_set(PlayerSystems::Input),
                    (
                        ask_again_with_truesight,
                        note_gifts,
                        read_server,
                        draw_grids,
                        show_gift_aim,
                    )
                        .chain()
                        // A panel opened this frame isn't spawned until its
                        // commands run, and `close_when_left` forgets a panel
                        // that doesn't exist: it would be left with no grid.
                        .after(close_when_left),
                    close_when_left.after(PlayerSystems::Travel),
                ),
            );
    }
}

#[cfg(test)]
mod tests;
