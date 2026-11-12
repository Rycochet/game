//! Long wind-ups shown: a bar filling under the token of whoever is winding up
//! an ability that takes a while to land, so a player waiting on their own
//! doesn't think the game has stopped, and can see one aimed at them coming.
//! Quick ones (every attack so far) show only on the action bar.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::id::Id;
use shared::protocol::ServerMessage;

use super::dressing_wait::DressingWait;
use super::health_bars::under_token;
use crate::common::{Other, OwnAbility};
use crate::connection::FromServer;
use crate::player::Player;
use crate::view3d::{TokenView, ViewCamera};

/// Wind-ups at least this long, in seconds, get a bar.
const LONG: f64 = 1.0;
const GOLD: Color = Color::srgb(0.95, 0.75, 0.2);
const EMPTY: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);

/// Others' wind-ups aimed at the player: when each began and lands.
#[derive(Resource, Default)]
pub(super) struct SeenWindUps(BTreeMap<Id, (f64, f64)>);

/// A bar, following one token's owner.
#[derive(Component)]
pub(super) struct WindUpBar(Entity);

#[derive(Component)]
pub(super) struct WindUpFill;

pub(super) fn take_wind_ups(
    mut from_server: MessageReader<FromServer>,
    time: Res<Time>,
    mut seen: ResMut<SeenWindUps>,
) {
    let now = time.elapsed_secs_f64();
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::AbilityStarted {
                who: Some(id),
                lands_in,
                ..
            } => {
                let lands = now + f64::from(*lands_in) / ACTION_TICKS_PER_SECOND;
                seen.0.insert(*id, (now, lands));
            }
            ServerMessage::AbilityLanded { who: Some(id), .. }
            | ServerMessage::EntityGone { id, .. } => {
                seen.0.remove(id);
            }
            _ => {}
        }
    }
}

/// How far through a wind-up from `started` to `lands` is, if it is long and
/// not over.
fn progress(started: f64, lands: f64, now: f64) -> Option<f32> {
    let whole = lands - started;
    (whole >= LONG && now < lands).then(|| ((now - started) / whole).clamp(0.0, 1.0) as f32)
}

/// Places a bar under every token winding up something long.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_wind_up_bars(
    time: Res<Time>,
    seen: Res<SeenWindUps>,
    own: Res<OwnAbility>,
    dressing: Res<DressingWait>,
    ui_scale: Res<UiScale>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    tokens: Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    owners: Query<(Option<&Other>, Has<Player>)>,
    mut bars: Query<(Entity, &WindUpBar, &mut Node, &Children)>,
    mut fills: Query<&mut Node, (With<WindUpFill>, Without<WindUpBar>)>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    let (camera, eye) = *camera;
    let progress_of = |owner: Entity| -> Option<f32> {
        let (other, is_player) = owners.get(owner).ok()?;
        if is_player {
            let ability = progress(own.started, own.lands, now);
            return ability.or_else(|| progress(dressing.started, dressing.ends, now));
        }
        let &(started, lands) = seen.0.get(&other?.id)?;
        progress(started, lands, now)
    };
    let mut placed = Vec::new();
    for (view, token, shown) in &tokens {
        let Some(done) = progress_of(view.owner).filter(|_| shown.get()) else {
            continue;
        };
        if let Some(node) = under_token(camera, eye, token, ui_scale.0, 1) {
            placed.push((view.owner, node, done));
        }
    }
    for (entity, bar, mut node, children) in &mut bars {
        let Some(index) = placed.iter().position(|(owner, ..)| *owner == bar.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let (_, new, done) = placed.swap_remove(index);
        *node = new;
        for child in children {
            if let Ok(mut fill) = fills.get_mut(*child) {
                fill.width = percent(done * 100.0);
            }
        }
    }
    for (owner, node, done) in placed {
        commands
            .spawn((WindUpBar(owner), node, BackgroundColor(EMPTY)))
            .with_child((
                WindUpFill,
                Node {
                    width: percent(done * 100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(GOLD),
            ));
    }
}

#[cfg(test)]
mod tests;
