use bevy::ecs::system::RunSystemOnce;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};

use super::Finder;
use crate::dom::parts::{Element, Inert, Reach, Round, Solid};
use crate::dom::{Attrs, Tag};

/// A node as if laid out at `corner`, `size` across, `stack` from the back.
pub(in crate::dom) fn laid(corner: Vec2, size: Vec2, stack: u32) -> impl Bundle {
    (
        ComputedNode {
            size,
            inverse_scale_factor: 1.0,
            ..default()
        },
        UiGlobalTransform::from(Affine2::from_translation(corner + size / 2.0)),
        InheritedVisibility::VISIBLE,
        ComputedStackIndex(stack),
    )
}

pub(in crate::dom) fn element() -> Element {
    Element {
        tag: Tag::Div,
        attrs: Attrs::default(),
    }
}

fn target(world: &mut World, at: Vec2) -> Option<Entity> {
    let found = world.run_system_once(move |finder: Finder| finder.target_at(at));
    found.unwrap()
}

const BIG: Vec2 = Vec2::splat(100.0);
const SMALL: Vec2 = Vec2::splat(20.0);

#[test]
fn the_thing_in_front_is_the_target() {
    let mut world = World::new();
    let behind = world.spawn((laid(Vec2::ZERO, BIG, 1), Solid)).id();
    let front = world.spawn((laid(Vec2::splat(50.0), BIG, 2), Solid)).id();
    assert_eq!(target(&mut world, Vec2::splat(60.0)), Some(front));
    assert_eq!(target(&mut world, Vec2::splat(10.0)), Some(behind));
    assert_eq!(target(&mut world, Vec2::splat(500.0)), None);
}

#[test]
fn plain_nodes_are_not_in_the_way() {
    let mut world = World::new();
    let behind = world.spawn((laid(Vec2::ZERO, BIG, 1), Solid)).id();
    world.spawn(laid(Vec2::ZERO, BIG, 2));
    assert_eq!(target(&mut world, Vec2::splat(10.0)), Some(behind));
}

#[test]
fn the_search_drills_down_through_elements() {
    let mut world = World::new();
    let outer = world.spawn((laid(Vec2::ZERO, BIG, 1), element())).id();
    let inner = world.spawn((laid(Vec2::ZERO, SMALL, 2), element(), ChildOf(outer)));
    let inner = inner.id();
    assert_eq!(target(&mut world, Vec2::splat(10.0)), Some(inner));
    assert_eq!(target(&mut world, Vec2::splat(50.0)), Some(outer));
    let chain = world.run_system_once(move |finder: Finder| finder.chain_of(inner));
    assert_eq!(chain.unwrap(), [inner, outer]);
}

#[test]
fn nothing_in_an_element_is_found_outside_it() {
    let mut world = World::new();
    let outer = world.spawn((laid(Vec2::ZERO, SMALL, 1), element())).id();
    // Laid out beyond what holds it, which markup never does.
    world.spawn((laid(Vec2::splat(50.0), SMALL, 2), element(), ChildOf(outer)));
    assert_eq!(target(&mut world, Vec2::splat(55.0)), None);
}

#[test]
fn other_nodes_may_hold_things_anywhere() {
    let mut world = World::new();
    let holder = world.spawn(laid(Vec2::ZERO, SMALL, 1)).id();
    let far = world.spawn((laid(Vec2::splat(50.0), SMALL, 2), Solid, ChildOf(holder)));
    let far = far.id();
    assert_eq!(target(&mut world, Vec2::splat(55.0)), Some(far));
}

#[test]
fn inert_and_hidden_things_are_passed_over_with_all_they_hold() {
    let mut world = World::new();
    let behind = world.spawn((laid(Vec2::ZERO, BIG, 1), Solid)).id();
    let inert = world
        .spawn((laid(Vec2::ZERO, BIG, 2), element(), Inert))
        .id();
    world.spawn((laid(Vec2::ZERO, SMALL, 3), element(), ChildOf(inert)));
    let hidden = world.spawn((laid(Vec2::ZERO, BIG, 4), Solid)).id();
    world.entity_mut(hidden).insert(InheritedVisibility::HIDDEN);
    assert_eq!(target(&mut world, Vec2::splat(10.0)), Some(behind));
}

#[test]
fn a_round_thing_is_pointed_at_only_inside_its_circle() {
    let mut world = World::new();
    let behind = world.spawn((laid(Vec2::ZERO, BIG, 1), Solid)).id();
    let round = (laid(Vec2::ZERO, BIG, 2), element(), Round);
    let round = world.spawn(round).id();
    assert_eq!(target(&mut world, Vec2::splat(50.0)), Some(round));
    assert_eq!(target(&mut world, Vec2::new(50.0, 2.0)), Some(round));
    // A corner of its box is off the circle: what is behind.
    assert_eq!(target(&mut world, Vec2::splat(5.0)), Some(behind));
}

#[test]
fn a_reach_counts_a_little_outside_over_what_is_behind() {
    let mut world = World::new();
    let behind = world.spawn((laid(Vec2::ZERO, BIG, 1), Solid)).id();
    let button = (laid(Vec2::splat(40.0), SMALL, 2), Solid, Reach(10.0));
    let button = world.spawn(button).id();
    let over = world.spawn((laid(Vec2::new(70.0, 0.0), SMALL, 3), Solid));
    let over = over.id();
    assert_eq!(target(&mut world, Vec2::splat(50.0)), Some(button));
    assert_eq!(target(&mut world, Vec2::new(35.0, 50.0)), Some(button));
    assert_eq!(target(&mut world, Vec2::new(25.0, 50.0)), Some(behind));
    assert_eq!(target(&mut world, Vec2::new(50.0, 65.0)), Some(button));
    // What is in front of it keeps its own place.
    assert_eq!(target(&mut world, Vec2::new(75.0, 10.0)), Some(over));
    world.entity_mut(button).insert(Inert);
    assert_eq!(target(&mut world, Vec2::new(35.0, 50.0)), Some(behind));
}
