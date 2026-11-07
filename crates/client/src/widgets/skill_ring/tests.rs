use bevy::ecs::schedule::ScheduleBuildSettings;
use bevy::prelude::*;
use shared::item::Quality;

use super::{SkillRing, SkillRingMaterial, give_rings};
use crate::common::BandShift;

fn world_with_rings() -> World {
    let mut world = World::new();
    world.init_resource::<Assets<SkillRingMaterial>>();
    world.init_resource::<BandShift>();
    world
}

fn despawn_rings(rings: Query<Entity, With<SkillRing>>, mut commands: Commands) {
    for ring in &rings {
        commands.entity(ring).despawn();
    }
}

#[test]
fn a_ring_despawned_as_its_material_is_given_is_no_crash() {
    // As when the Skills page drew its tree again just as a new ring was given its
    // material: the despawn lands first, then the material.
    let mut world = world_with_rings();
    world.spawn(SkillRing::of(10.0, false));
    let mut schedule = Schedule::default();
    schedule.set_build_settings(ScheduleBuildSettings {
        auto_insert_apply_deferred: false,
        ..default()
    });
    schedule.add_systems((despawn_rings, give_rings).chain());
    schedule.run(&mut world);
    assert_eq!(world.query::<&SkillRing>().iter(&world).count(), 0);
}

#[test]
fn a_changed_ring_keeps_its_material_with_new_settings() {
    let mut world = world_with_rings();
    let ring = world.spawn(SkillRing::of(10.0, false)).id();
    let mut schedule = Schedule::default();
    schedule.add_systems(give_rings);
    schedule.run(&mut world);
    let first = world
        .get::<MaterialNode<SkillRingMaterial>>(ring)
        .unwrap()
        .0
        .clone();
    *world.get_mut::<SkillRing>(ring).unwrap() = SkillRing::of(50.0, false);
    schedule.run(&mut world);
    let now = &world
        .get::<MaterialNode<SkillRingMaterial>>(ring)
        .unwrap()
        .0;
    assert_eq!(now.id(), first.id(), "the same material");
    let materials = world.resource::<Assets<SkillRingMaterial>>();
    assert_eq!(materials.len(), 1);
    assert!((materials.get(now).unwrap().params.fill - 0.5).abs() < 1e-6);
}

#[test]
fn a_ring_fills_to_100_in_fifths_of_quality() {
    assert_eq!(SkillRing::of(0.0, false).fill, 0.0);
    assert_eq!(SkillRing::of(0.5, false).fill, 0.02, "a sliver at first");
    for (value, quality) in [
        (10.0, Quality::Common),
        (20.4, Quality::Uncommon),
        (45.0, Quality::Rare),
        (79.9, Quality::Epic),
        (80.0, Quality::Legendary),
        (100.0, Quality::Legendary),
    ] {
        let ring = SkillRing::of(value, false);
        assert_eq!(ring.quality, quality, "{value}");
        assert!(
            (ring.fill - (value / 100.0).max(0.02)).abs() < 1e-6,
            "{value}"
        );
    }
    assert_eq!(SkillRing::of(150.0, false).fill, 1.0);
}

#[test]
fn only_a_specialised_skill_fills_its_second_ring_from_100_to_120() {
    assert_eq!(SkillRing::of(110.0, false).special, 0.0);
    assert_eq!(SkillRing::of(90.0, true).special, 0.0);
    assert!((SkillRing::of(110.0, true).special - 0.5).abs() < 1e-6);
    assert_eq!(SkillRing::of(130.0, true).special, 1.0);
}
