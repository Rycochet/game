//! The Hall of Worlds on the client. The server only says where the player
//! is (`EnterHall`); its cells are fixed rules (`shared::hall`), worked out
//! here a stretch at a time round the player, all known from the start, like
//! a town. The stretch moves along as they walk.

use bevy::prelude::*;
use shared::hall::{self, LEVEL_ROWS, LEVEL_TOP};
use shared::protocol::{LevelInfo, LevelKind};

use crate::common::{CurrentDungeon, MapPosition};
use crate::player::Player;

/// How much of the Hall is held at once, in tiles east to west.
const STRETCH: i64 = 300;
/// How near its end the player may come before it is moved along.
const MARGIN: i64 = 75;

/// Makes the current level the stretch of the Hall round `x`, all known; `x`
/// counts from `origin`, as the server sends it.
pub(super) fn show_around(dungeon: &mut CurrentDungeon, (x, y): (i64, i64), origin: i64) {
    let min_x = x - STRETCH / 2;
    dungeon.enter(LevelInfo {
        kind: LevelKind::Hall { origin },
        ruined: false,
        building: false,
        index: 0,
        levels: 1,
        basement: false,
        bounds: (min_x, LEVEL_TOP, STRETCH, LEVEL_ROWS),
        arrival: (x, y),
        entering_site: true,
        place_name: None,
        ground: Default::default(),
    });
    dungeon.level = hall::level(min_x, STRETCH);
    for y in LEVEL_TOP..LEVEL_TOP + LEVEL_ROWS {
        for x in min_x..min_x + STRETCH {
            dungeon.known.insert(x, y);
            dungeon.remembered.insert(x, y);
        }
    }
}

/// Moves the stretch along when the player comes near its end (walking, or
/// jumping with dev tools).
pub(super) fn follow_player(
    mut dungeon: ResMut<CurrentDungeon>,
    player: Single<&MapPosition, (With<Player>, Changed<MapPosition>)>,
) {
    let Some(info) = &dungeon.info else {
        return;
    };
    let LevelKind::Hall { origin } = info.kind else {
        return;
    };
    let (min_x, _, width, _) = info.bounds;
    if !(min_x + MARGIN..min_x + width - MARGIN).contains(&player.x) {
        show_around(&mut dungeon, (player.x, player.y), origin);
    }
}
