//! The World Gates the player has gone through, as the server tells them, and
//! the water in every gate's opening: orange for those, blue for the rest. Any
//! gate's water, in the Hall or on the world map, is a `GateWater`, given its
//! material here.

use bevy::prelude::*;
use shared::protocol::ServerMessage;
use std::collections::HashSet;

use super::GateModel;
use crate::connection::FromServer;

/// The Hall gates (`GatePlace::number`) whose pairs the player has gone
/// through, either way.
#[derive(Resource, Default)]
pub struct VisitedGates(pub HashSet<u32>);

/// The sheet of water in the opening of a gate leading to Hall gate `0` (a
/// `GatePlace::number`), or that Hall gate itself: its material is put on
/// here, and changed when the player goes through.
#[derive(Component, Clone, Copy)]
pub struct GateWater(pub u32);

/// The two waters, shared by every gate. Their pictures slide down while in
/// the Hall (`run_water`).
#[derive(Resource)]
pub(super) struct WaterMaterials {
    pub(super) blue: Handle<StandardMaterial>,
    pub(super) orange: Handle<StandardMaterial>,
}

impl FromWorld for WaterMaterials {
    fn from_world(world: &mut World) -> Self {
        let blue = GateModel::water_picture(false);
        let orange = GateModel::water_picture(true);
        let mut images = world.resource_mut::<Assets<Image>>();
        let (blue, orange) = (images.add(blue), images.add(orange));
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let mut material = |picture| {
            materials.add(StandardMaterial {
                base_color_texture: Some(picture),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                double_sided: true,
                ..default()
            })
        };
        Self {
            blue: material(blue),
            orange: material(orange),
        }
    }
}

impl WaterMaterials {
    fn of(&self, place: u32, visited: &VisitedGates) -> Handle<StandardMaterial> {
        if visited.0.contains(&place) {
            self.orange.clone()
        } else {
            self.blue.clone()
        }
    }
}

/// Keeps what the server says of the gates gone through.
pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut visited: ResMut<VisitedGates>,
) {
    for message in from_server.read() {
        if let ServerMessage::VisitedGates(places) = &message.0 {
            visited.0 = places.iter().copied().collect();
        }
    }
}

/// Gives new water its material, and all of it again when the gates gone
/// through change.
pub(super) fn colour_water(
    visited: Res<VisitedGates>,
    materials: Res<WaterMaterials>,
    all: Query<(Entity, &GateWater)>,
    new: Query<(Entity, &GateWater), Added<GateWater>>,
    mut commands: Commands,
) {
    let waters = if visited.is_changed() {
        all.iter().collect::<Vec<_>>()
    } else {
        new.iter().collect()
    };
    for (entity, &GateWater(place)) in waters {
        let material = materials.of(place, &visited);
        commands.entity(entity).insert(MeshMaterial3d(material));
    }
}

#[cfg(test)]
mod tests;
