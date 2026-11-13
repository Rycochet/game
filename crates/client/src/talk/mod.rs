//! Talking at town doors: clicking one opens a panel named for who is behind
//! it, with the quests they have (taking and handing them in) and, for a
//! shop, a Trade tab with its stock (walking there first if the player isn't
//! beside it). What there is comes from the server, which only tells someone
//! in reach. See `docs/context/client/talk.md`.

use bevy::prelude::*;

use crate::common::PointerSystems;
use crate::dom::DomInput;
use crate::player::PlayerSystems;

// Split into files (`cargo xtask lint` keeps files small).
mod close;
mod detail;
mod input;
mod markup;
mod news;
mod open;
mod quests_page;
mod spawn;
mod state;
mod stock;
mod words;

use close::{close_on_escape, close_when_left};
use open::{ask_again_with_truesight, change_of_mind, follow_clicked_door, open_on_arrival};
use state::{Heading, Keepers, OpenTalk, OpenTalks, Tab, beside, front_of, key_of};
use stock::{bind_grids, draw_grids, note_gifts, read_server, show_gift_aim};

/// To the left, leaving the player in view.
const AT: Vec2 = Vec2::new(40.0, 120.0);
/// How far along each panel open already puts the next.
const NEXT_AT: Vec2 = Vec2::new(36.0, 36.0);
/// How many cells across a shop's grid is.
const COLUMNS: u8 = 12;

pub struct TalkPlugin;

impl Plugin for TalkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenTalks>()
            .init_resource::<Heading>()
            .init_resource::<Keepers>()
            .add_systems(
                PreUpdate,
                close_on_escape
                    .after(PointerSystems)
                    .before(DomInput::Front),
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
                        news::read_quest_news,
                        input::read_events,
                        bind_grids,
                        draw_grids,
                        show_gift_aim,
                        quests_page::show_pages,
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
