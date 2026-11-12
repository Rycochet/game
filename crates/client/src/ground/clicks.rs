//! The bag the pointer is on, and walking to a clicked bag to open it.

use bevy::prelude::*;

use shared::ground::in_reach;
use shared::spot::Spot;

use super::bag::size_of;
use super::{Bags, Heading, STAR_LONG, in_cells, sparkle_scale, sparkle_up};
use crate::common::{
    ClickMode, DrawnGround, HoveredBag, PlayerSize, PointerPosition, SpotPosition,
};
use crate::dungeon_map::{CELL_SIZE, Shown};
use crate::inventory::OpenCharacter;
use crate::player::{Arrived, Player, TakeStep, TravelTo, Walking};
use crate::view3d::{ClickedToken, ViewCamera};

/// Which bag the pointer is on, measured on screen, nearest the camera first. Its
/// sparkle counts while it is showing.
pub(super) fn find_hovered(
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    bags: Res<Bags>,
    shown: Res<Shown>,
    heights: Res<DrawnGround>,
    mut hovered: ResMut<HoveredBag>,
) {
    let (camera, transform) = *camera;
    let found = pointer.0.and_then(|cursor| {
        let mut best: Option<(f32, Spot)> = None;
        for (&at, bag) in &bags.0 {
            let size = size_of(bag.load);
            let (x, y) = in_cells(at);
            let ground = heights.drawn_between(x, y) * CELL_SIZE;
            let foot = Vec3::new(x * CELL_SIZE, ground, y * CELL_SIZE);
            let cell = at.cell();
            let in_sight = shown
                .sight
                .as_ref()
                .is_some_and(|sight| sight.is_seen(cell.0, cell.1));
            let tallest = if in_sight {
                size.height
                    .max(sparkle_up(bag.load) + STAR_LONG * sparkle_scale(bag.load))
            } else {
                size.height
            };
            let top = foot + Vec3::Y * tallest;
            let side = foot + transform.right() * size.radius;
            let (Ok(foot_at), Ok(top_at), Ok(side_at)) = (
                camera.world_to_viewport(transform, foot),
                camera.world_to_viewport(transform, top),
                camera.world_to_viewport(transform, side),
            ) else {
                continue;
            };
            let reach = foot_at.distance(side_at);
            let on_it = (cursor.x - foot_at.x).abs() <= reach
                && cursor.y >= top_at.y - reach / 2.0
                && cursor.y <= foot_at.y + reach / 2.0;
            let distance = transform.translation().distance(foot);
            if on_it && best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, at));
            }
        }
        best.map(|(_, at)| at)
    });
    hovered.set_if_neq(HoveredBag(found));
}

/// Walking anywhere else, or a step of their own: they aren't going to the bag.
pub(super) fn change_of_mind(
    (mut steps, mut walking): (MessageReader<TakeStep>, MessageReader<Walking>),
    mut journeys: MessageReader<TravelTo>,
    mut heading: ResMut<Heading>,
) {
    let stepped = steps.read().count() + walking.read().count() > 0;
    let elsewhere = journeys
        .read()
        .any(|journey| Some((journey.x, journey.y)) != heading.0.map(Spot::cell));
    if stepped || elsewhere {
        heading.0 = None;
    }
}

/// A click on a bag opens the Character panel if the player can reach it, or else
/// walks them to its cell to open it on arriving. A tap arrives as a click.
#[allow(clippy::too_many_arguments)]
pub(super) fn follow_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mode: Res<ClickMode>,
    clicked: Res<ClickedToken>,
    hovered: Res<HoveredBag>,
    player: Single<&SpotPosition, With<Player>>,
    size: Res<PlayerSize>,
    mut heading: ResMut<Heading>,
    mut travel: MessageWriter<TravelTo>,
    mut open: MessageWriter<OpenCharacter>,
) {
    let Some(at) = hovered.0 else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left) || clicked.0.is_some() || *mode != ClickMode::Normal {
        return;
    }
    if in_reach(size.0, player.0, at) {
        heading.0 = None;
        open.write(OpenCharacter);
    } else {
        heading.0 = Some(at);
        let (x, y) = at.cell();
        travel.write(TravelTo { x, y });
    }
}

pub(super) fn open_on_arrival(
    mut arrivals: MessageReader<Arrived>,
    mut heading: ResMut<Heading>,
    bags: Res<Bags>,
    mut open: MessageWriter<OpenCharacter>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    // At its cell's middle, which reaches anywhere in the cell.
    let heading_to = heading.0.take();
    if let Some(at) = heading_to.filter(|at| at.cell() == (x, y))
        && bags.0.contains_key(&at)
    {
        open.write(OpenCharacter);
    }
}
