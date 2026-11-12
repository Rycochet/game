//! Opening a shop's panel: clicking its door, walking there, and asking again with truesight.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::{Heading, Keepers, OpenTrades, beside, front_of};
use crate::common::{CurrentDungeon, MapPosition, Truesight};
use crate::connection::ToServer;
use crate::dom::Dom;
use crate::dungeon_map::DoorClicked;
use crate::player::{Arrived, Player, TakeStep, TravelTo};
use crate::widgets::UiId;

/// Walking anywhere else, or a step of their own: they aren't going to the shop.
pub(super) fn change_of_mind(
    mut steps: MessageReader<TakeStep>,
    mut journeys: MessageReader<TravelTo>,
    mut heading: ResMut<Heading>,
) {
    let stepped = steps.read().count() > 0;
    let front = heading.0.map(|(_, front)| front);
    let elsewhere = journeys
        .read()
        .any(|journey| Some((journey.x, journey.y)) != front);
    if stepped || elsewhere {
        heading.0 = None;
    }
}

/// A click on a shop's door brings its panel to the front if it is open, opens it
/// if the player is beside it, or else walks them to it.
#[allow(clippy::too_many_arguments)]
pub(super) fn follow_clicked_door(
    mut clicks: MessageReader<DoorClicked>,
    dungeon: Res<CurrentDungeon>,
    keepers: Res<Keepers>,
    open: Res<OpenTrades>,
    player: Single<&MapPosition, With<Player>>,
    mut heading: ResMut<Heading>,
    mut travel: MessageWriter<TravelTo>,
    mut dom: Dom,
    mut to_server: MessageWriter<ToServer>,
) {
    for click in clicks.read() {
        let building = dungeon.buildings.iter().find(|b| b.id == click.building);
        let Some(building) = building else {
            continue;
        };
        let front = front_of(building);
        // Only somewhere to walk to if it isn't a shop.
        let Some(&with) = keepers.0.get(&building.id) else {
            travel.write(TravelTo {
                x: front.0,
                y: front.1,
            });
            continue;
        };
        if open.is_open_with(with) {
            dom.raise(&UiId::of("trade", with, "panel").to_string());
        } else if beside(**player, building.door) {
            heading.0 = None;
            to_server.write(ToServer(ClientMessage::OpenTrade { with }));
        } else {
            heading.0 = Some((with, front));
            travel.write(TravelTo {
                x: front.0,
                y: front.1,
            });
        }
    }
}

/// Truesight changes what the server tells of what is shown here.
pub(super) fn ask_again_with_truesight(
    truesight: Res<Truesight>,
    open: Res<OpenTrades>,
    mut to_server: MessageWriter<ToServer>,
) {
    if truesight.is_changed() && !truesight.is_added() {
        for trade in &open.0 {
            let with = trade.grid.with;
            to_server.write(ToServer(ClientMessage::OpenTrade { with }));
        }
    }
}

pub(super) fn open_on_arrival(
    mut arrivals: MessageReader<Arrived>,
    mut heading: ResMut<Heading>,
    open: Res<OpenTrades>,
    mut to_server: MessageWriter<ToServer>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    if let Some((with, front)) = heading.0.take()
        && front == (x, y)
        && !open.is_open_with(with)
    {
        // After the steps there, so the server has the player at it.
        to_server.write(ToServer(ClientMessage::OpenTrade { with }));
    }
}
