//! Talking at a door: clicking it, walking there, and asking the server what
//! the giver has and (for a shop) what is in stock; asking again with
//! truesight.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, QuestAsk};

use super::{Heading, Keepers, OpenTalks, beside, front_of, key_of};
use crate::common::{CurrentDungeon, MapPosition, Truesight};
use crate::connection::ToServer;
use crate::dom::Dom;
use crate::dungeon_map::DoorClicked;
use crate::player::{Arrived, Player, TakeStep, TravelTo, Walking};
use crate::quests::DoorMarkers;
use crate::widgets::UiId;

/// Walking anywhere else, or a step of their own: they aren't going to the door.
pub(super) fn change_of_mind(
    (mut steps, mut walking): (MessageReader<TakeStep>, MessageReader<Walking>),
    mut journeys: MessageReader<TravelTo>,
    mut heading: ResMut<Heading>,
) {
    let stepped = steps.read().count() + walking.read().count() > 0;
    let front = heading.0.map(|(_, front)| front);
    let elsewhere = journeys
        .read()
        .any(|journey| Some((journey.x, journey.y)) != front);
    if stepped || elsewhere {
        heading.0 = None;
    }
}

/// What the server is asked on coming to a door: what its giver has, in the
/// town the player is in, and its stock if it is a shop. The panel opens on
/// the answers.
fn ask_at_door(
    building: u16,
    keepers: &Keepers,
    doors: &DoorMarkers,
    to_server: &mut MessageWriter<ToServer>,
) {
    if let Some(town) = doors.town {
        let talk = QuestAsk::Talk { town, building };
        to_server.write(ToServer(ClientMessage::Quest(talk)));
    }
    if let Some(&with) = keepers.0.get(&building) {
        to_server.write(ToServer(ClientMessage::OpenTrade { with }));
    }
}

/// A click on a door brings its panel to the front if it is open, asks the
/// server if the player is beside it, or else walks them to it.
#[allow(clippy::too_many_arguments)]
pub(super) fn follow_clicked_door(
    mut clicks: MessageReader<DoorClicked>,
    dungeon: Res<CurrentDungeon>,
    (keepers, doors): (Res<Keepers>, Res<DoorMarkers>),
    open: Res<OpenTalks>,
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
        if open.is_open(building.id) {
            dom.raise(&UiId::of("talk", key_of(building.id), "panel").to_string());
        } else if beside(**player, building.door) {
            heading.0 = None;
            ask_at_door(building.id, &keepers, &doors, &mut to_server);
        } else {
            heading.0 = Some((building.id, front));
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
    open: Res<OpenTalks>,
    mut to_server: MessageWriter<ToServer>,
) {
    if truesight.is_changed() && !truesight.is_added() {
        for with in open.0.iter().filter_map(|talk| talk.keeper()) {
            to_server.write(ToServer(ClientMessage::OpenTrade { with }));
        }
    }
}

pub(super) fn open_on_arrival(
    mut arrivals: MessageReader<Arrived>,
    mut heading: ResMut<Heading>,
    open: Res<OpenTalks>,
    (keepers, doors): (Res<Keepers>, Res<DoorMarkers>),
    mut to_server: MessageWriter<ToServer>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    if let Some((building, front)) = heading.0.take()
        && front == (x, y)
        && !open.is_open(building)
    {
        // After the steps there, so the server has the player at it.
        ask_at_door(building, &keepers, &doors, &mut to_server);
    }
}
