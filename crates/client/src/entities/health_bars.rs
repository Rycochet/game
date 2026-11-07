//! Health bars: under the token of anyone hurt, as wide as its base, and only
//! while they aren't at full Health (owner, 2026-11-07). The player's from
//! their own Health; others' from what the player's blows left them, refilled
//! here as Health refills (`shared::vitals`), as nobody tells it again.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::id::Id;
use shared::protocol::{Landed, ServerMessage};
use shared::vitals::{CALM_AFTER, REFILL_TICKS};

use crate::common::{Other, OwnVitals};
use crate::connection::FromServer;
use crate::player::Player;
use crate::view3d::{TOKEN_DIAMETER, TokenView, ViewCamera};
use crate::widgets::Floating;

const RED: Color = Color::srgb(0.8, 0.1, 0.08);
const EMPTY: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
/// How tall a bar is, as a share of its width.
const HEIGHT_OF_WIDTH: f32 = 0.12;

/// What the player's blows last left others with, per mille, and when.
#[derive(Resource, Default)]
pub(super) struct SeenHealth(BTreeMap<Id, (u16, f64)>);

/// A bar, following one token's owner.
#[derive(Component)]
pub(super) struct HealthBar(Entity);

/// The red part of a bar.
#[derive(Component)]
pub(super) struct HealthFill;

pub(super) fn take_health(
    mut from_server: MessageReader<FromServer>,
    time: Res<Time>,
    mut seen: ResMut<SeenHealth>,
) {
    let now = time.elapsed_secs_f64();
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::AbilityLanded {
                target: Some(id),
                landed: Landed::Hit { left, .. } | Landed::Healed { left, .. },
                ..
            } => {
                seen.0.insert(*id, (*left, now));
            }
            ServerMessage::EntityGone { id, .. } => {
                seen.0.remove(id);
            }
            _ => {}
        }
    }
}

/// How much of their Health someone has, from 0 to 1, refilled since they were
/// last seen hurt.
pub(super) fn refilled_share(left: u16, at: f64, now: f64) -> f32 {
    let seconds = |ticks: u64| ticks as f64 / ACTION_TICKS_PER_SECOND;
    let refilling = (now - at - seconds(CALM_AFTER)).max(0.0);
    let share = f64::from(left) / 1000.0 + refilling / seconds(REFILL_TICKS);
    share.min(1.0) as f32
}

/// A bar's place under a token, as wide as its base: `row` 0 just under it, 1
/// under that. `None` if the token is off screen.
pub(super) fn under_token(
    camera: &Camera,
    eye: &GlobalTransform,
    token: &GlobalTransform,
    ui_scale: f32,
    row: u8,
) -> Option<Node> {
    let middle = token.translation();
    let side = middle + eye.right() * (TOKEN_DIAMETER / 2.0);
    let at = camera.world_to_viewport(eye, middle).ok()?;
    let edge = camera.world_to_viewport(eye, side).ok()?;
    let scale = ui_scale.max(0.01);
    let width = 2.0 * (edge - at).length() / scale;
    let height = (width * HEIGHT_OF_WIDTH).max(3.0);
    Some(Node {
        position_type: PositionType::Absolute,
        left: px(at.x / scale - width / 2.0),
        top: px(at.y / scale + width * 0.3 + f32::from(row) * (height + 2.0)),
        width: px(width),
        height: px(height),
        ..default()
    })
}

/// Places a bar under every hurt token in sight, and takes away the rest.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_health_bars(
    time: Res<Time>,
    seen: Res<SeenHealth>,
    own: Res<OwnVitals>,
    ui_scale: Res<UiScale>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    tokens: Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    owners: Query<(Option<&Other>, Has<Player>)>,
    mut bars: Query<(Entity, &HealthBar, &mut Node, &Children)>,
    mut fills: Query<&mut Node, (With<HealthFill>, Without<HealthBar>)>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    let (camera, eye) = *camera;
    let share_of = |owner: Entity| -> Option<f32> {
        let (other, is_player) = owners.get(owner).ok()?;
        let share = if is_player {
            let told = own.0?;
            let (health, _) = told.now(now);
            health / told.health.most.max(1) as f32
        } else {
            let &(left, at) = seen.0.get(&other?.id)?;
            refilled_share(left, at, now)
        };
        (share < 1.0).then_some(share)
    };
    let mut placed = Vec::new();
    for (view, token, shown) in &tokens {
        let Some(share) = share_of(view.owner).filter(|_| shown.get()) else {
            continue;
        };
        let Some(node) = under_token(camera, eye, token, ui_scale.0, 0) else {
            continue;
        };
        placed.push((view.owner, node, share));
    }
    for (entity, bar, mut node, children) in &mut bars {
        let Some(index) = placed.iter().position(|(owner, ..)| *owner == bar.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let (_, new, share) = placed.swap_remove(index);
        *node = new;
        for child in children {
            if let Ok(mut fill) = fills.get_mut(*child) {
                fill.width = percent(share * 100.0);
            }
        }
    }
    for (owner, node, share) in placed {
        commands
            .spawn((HealthBar(owner), Floating, node, BackgroundColor(EMPTY)))
            .with_child((
                HealthFill,
                Node {
                    width: percent(share * 100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(RED),
            ));
    }
}
