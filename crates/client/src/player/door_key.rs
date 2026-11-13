//! The Open/Close Doors key (O): closes the open doors beside the player, or
//! if there are none, opens the closed door they face or stand beside
//! (Claude's rule, in the quests plan). The server checks either.

use bevy::prelude::*;
use shared::door::Swing;
use shared::dungeon::{Cell, Level};
use shared::facing::Heading;
use shared::protocol::DoorAsk;

use super::Player;
use super::doors::AskDoor;
use crate::common::{Action, CurrentDungeon, Facing, GameView, MapPosition, Preferences};

/// O on a level: what `door_ask` says to do.
pub(super) fn open_or_close_doors(
    preferences: Res<Preferences>,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    player: Single<(&MapPosition, &Facing), With<Player>>,
    mut asks: MessageWriter<AskDoor>,
) {
    if *view.get() != GameView::Dungeon
        || !preferences
            .controls
            .just_pressed(&keys, Action::OpenCloseDoors)
    {
        return;
    }
    let (at, facing) = player.into_inner();
    if let Some(ask) = door_ask(&dungeon.level, (at.x, at.y), facing.0) {
        asks.write(AskDoor(ask));
    }
}

/// Close every open door beside `at` if there is one; else open the closed
/// door beside it nearest the way it faces (the one ahead before one to the
/// side, then the one behind); else nothing.
pub(super) fn door_ask(level: &Level, at: (i64, i64), facing: Heading) -> Option<DoorAsk> {
    let beside =
        beside(at).filter(|&(x, y)| matches!(level.cell(x, y), Cell::Door | Cell::SecretDoor));
    let doors: Vec<(i64, i64)> = beside.collect();
    let open = doors
        .iter()
        .any(|&(x, y)| matches!(level.swing(x, y), Some(Swing::Open(_))));
    if open {
        return Some(DoorAsk::CloseNear);
    }
    let shut = doors.into_iter().filter(|&(x, y)| level.is_shut(x, y));
    let nearest_way = shut.min_by_key(|&(x, y)| {
        Heading::of(x - at.0, y - at.1).map_or(u16::MAX, |way| way.apart(facing))
    });
    nearest_way.map(|(x, y)| DoorAsk::Open { x, y })
}

/// The eight cells round `at`.
fn beside(at: (i64, i64)) -> impl Iterator<Item = (i64, i64)> {
    (-1..=1)
        .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
        .filter(|&step| step != (0, 0))
        .map(move |(dx, dy)| (at.0 + dx, at.1 + dy))
}

#[cfg(test)]
mod tests;
