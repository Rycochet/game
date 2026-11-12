//! Which way the player faces (`common::Facing`): the way they move, turned
//! by Turn Left and Turn Right (keys or the pad's buttons), towards a point
//! pointed at on the map, and towards whoever they use an ability on. Turns
//! made without moving are told to the server (`ClientMessage::Face`); it
//! turns them by their moves itself.

use bevy::prelude::*;
use shared::facing::{Cone, EIGHTH, Heading, TURN};
use shared::protocol::ClientMessage;
use shared::spot::{CELL_SPAN, Spot};

use super::Player;
use crate::common::{
    Action, FaceTowards, Facing, GameView, Other, PadSteps, PlayerCone, PlayerField, Preferences,
    SmoothPosition, SpotPosition,
};
use crate::connection::ToServer;

/// How long Turn Left or Right is held before it keeps turning, in seconds.
const HOLD_DELAY: f32 = 0.3;

/// How fast a held turn goes: half a turn a second (owner), in steps.
const HELD_PER_SECOND: f32 = (TURN / 2) as f32;

/// How often a turn in place is told at most, in seconds.
const TELL_EVERY: f32 = 0.1;

/// Moving less than this in a frame, in cells, doesn't turn them (a
/// correction, say); more than `TOO_FAR` is a jump to somewhere else.
const MOVED: f64 = 0.004;
const TOO_FAR: f64 = 2.0;

/// A turn in place not yet told to the server, and when one was last told.
#[derive(Resource, Default)]
pub(super) struct UntoldTurn {
    heading: Option<Heading>,
    since_told: f32,
}

/// How a turn by hand is going: which way is held, and for how long.
#[derive(Default)]
pub(super) struct Held {
    way: i64,
    for_seconds: f32,
    /// Part of a step turned while held, carried to the next frame.
    part: f32,
}

/// Turn Left and Turn Right, by key or pad: a tap turns an eighth, holding
/// keeps turning. Only on a level or in the Hall.
#[allow(clippy::too_many_arguments)]
pub(super) fn turn_by_hand(
    (keys, preferences): (Res<ButtonInput<KeyCode>>, Res<Preferences>),
    pad: Res<PadSteps>,
    view: Res<State<GameView>>,
    time: Res<Time>,
    mut facing: Single<&mut Facing, With<Player>>,
    mut untold: ResMut<UntoldTurn>,
    mut held: Local<Held>,
) {
    let active = |action| preferences.controls.active(&keys, action);
    let keys_way = i64::from(active(Action::FaceRight)) - i64::from(active(Action::FaceLeft));
    let way = (keys_way + pad.turn).clamp(-1, 1);
    if *view.get() != GameView::Dungeon || way == 0 {
        *held = Held::default();
        return;
    }
    let mut steps = 0;
    if held.way != way {
        // A new press: an eighth at once.
        *held = Held { way, ..default() };
        steps = i32::from(EIGHTH);
    } else {
        held.for_seconds += time.delta_secs();
        if held.for_seconds >= HOLD_DELAY {
            held.part += HELD_PER_SECOND * time.delta_secs();
            steps = held.part.floor() as i32;
            held.part -= steps as f32;
        }
    }
    if steps > 0 {
        let heading = turned_by_hand(facing.0, way, steps);
        facing.0 = heading;
        untold.heading = Some(heading);
    }
}

/// `heading` turned `steps` to the left (`way` -1) or right (1). Left is
/// anticlockwise on the map with north up: from east towards north, as the
/// heading falls.
fn turned_by_hand(heading: Heading, way: i64, steps: i32) -> Heading {
    heading.turned(steps * way as i32)
}

/// Turns to a point pointed at on the map (`FaceTowards`).
pub(super) fn face_points(
    mut asked: MessageReader<FaceTowards>,
    player: Single<(&mut Facing, &SmoothPosition), With<Player>>,
    mut untold: ResMut<UntoldTurn>,
) {
    let (mut facing, smooth) = player.into_inner();
    for FaceTowards(to) in asked.read() {
        if let Some(heading) = Heading::towards(spot_of(smooth), *to) {
            facing.0 = heading;
            untold.heading = Some(heading);
        }
    }
}

/// Using an ability on someone turns the player to them (the server turns
/// them too, so it isn't told).
pub(super) fn face_ability_targets(
    mut sent: MessageReader<ToServer>,
    others: Query<(&Other, &SpotPosition)>,
    player: Single<(&mut Facing, &SmoothPosition), With<Player>>,
    view: Res<State<GameView>>,
) {
    let (mut facing, smooth) = player.into_inner();
    for ToServer(message) in sent.read() {
        let ClientMessage::UseAbility {
            target: Some(id), ..
        } = message
        else {
            continue;
        };
        let at = others.iter().find(|(other, _)| other.id == *id);
        if let Some((_, at)) = at.filter(|_| *view.get() == GameView::Dungeon)
            && let Some(heading) = Heading::towards(spot_of(smooth), at.0)
        {
            facing.0 = heading;
        }
    }
}

/// Moving turns them the way they go, on either map.
pub(super) fn face_where_moving(
    player: Single<(&mut Facing, &SmoothPosition), With<Player>>,
    mut was: Local<Option<(f64, f64)>>,
) {
    let (mut facing, smooth) = player.into_inner();
    let now = (smooth.x, smooth.y);
    let Some(before) = was.replace(now) else {
        return;
    };
    let (dx, dy) = (now.0 - before.0, now.1 - before.1);
    let moved = dx.hypot(dy);
    if (MOVED..TOO_FAR).contains(&moved) {
        facing.set_if_neq(Facing(Heading::from_radians(dy.atan2(dx) as f32)));
    }
}

/// Tells the server of a turn in place, at most every `TELL_EVERY`.
pub(super) fn tell_turns(
    time: Res<Time>,
    mut untold: ResMut<UntoldTurn>,
    mut to_server: MessageWriter<ToServer>,
) {
    untold.since_told += time.delta_secs();
    if untold.since_told < TELL_EVERY {
        return;
    }
    if let Some(heading) = untold.heading.take() {
        to_server.write(ToServer(ClientMessage::Face(heading)));
        untold.since_told = 0.0;
    }
}

/// The player's cone where they are drawn, on a level or in the Hall.
pub(super) fn keep_cone(
    view: Res<State<GameView>>,
    field: Res<PlayerField>,
    player: Single<(&Facing, &SmoothPosition), With<Player>>,
    mut cone: ResMut<PlayerCone>,
) {
    let (facing, smooth) = *player;
    let now =
        (*view.get() == GameView::Dungeon).then(|| Cone::new(spot_of(smooth), facing.0, field.0));
    cone.set_if_neq(PlayerCone(now));
}

/// Where the player is drawn, in units.
fn spot_of(smooth: &SmoothPosition) -> Spot {
    let units = |cells: f64| (cells * CELL_SPAN as f64).round() as i64;
    Spot::new(units(smooth.x), units(smooth.y))
}

#[cfg(test)]
mod tests;
