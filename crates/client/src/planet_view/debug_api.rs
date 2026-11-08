//! `client.planet`: with View Planet open, choose its generator, seed and
//! layer, and save its picture of the surface (as far as it is filled), so a
//! generator can be looked at closely without clicking through the bar.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params, refused};
use shared::rng::Seed;
use shared::surface::SurfaceGenerator;

use bevy::ui::UiGlobalTransform;

use super::layer::Layer;
use super::pick::{gate_spot, on_globe, on_picture};
use super::showing::PlanetView;

#[derive(Deserialize)]
struct Asked {
    generator: Option<String>,
    seed: Option<u64>,
    layer: Option<String>,
    /// Where to save the picture, as a PNG.
    path: Option<String>,
}

pub(super) fn add(app: &mut App) {
    use server::debug_api::AddDebugMethod;
    app.add_debug_method("client.planet", planet);
}

/// `client.planet {generator?, seed?, layer?, path?}`: names as the bar shows
/// them. Answers whether the picture is filled yet; a change starts it again.
/// Also where each gate shows in the window (`client.move`'s pixels, null
/// round the back), and the gate whose ring the pointer is on.
fn planet(
    In(request): In<Option<Value>>,
    mut view: ResMut<PlanetView>,
    images: Res<Assets<Image>>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    places: Query<&Transform>,
    ui_scale: Res<UiScale>,
) -> BrpResult {
    let asked: Asked = params(request)?;
    let Some(showing) = view.showing.as_mut() else {
        return Err(refused("View Planet isn't open"));
    };
    if let Some(name) = asked.generator {
        showing.choice.generator = SurfaceGenerator::ALL
            .into_iter()
            .find(|kind| kind.name().eq_ignore_ascii_case(&name))
            .ok_or_else(|| refused("generator is Desert, Continents or Plates"))?;
    }
    if let Some(seed) = asked.seed {
        showing.choice.seed = Seed::new(seed);
    }
    if let Some(name) = asked.layer {
        showing.choice.layer = Layer::ALL
            .into_iter()
            .find(|layer| layer.name().eq_ignore_ascii_case(&name))
            .ok_or_else(|| {
                refused("layer is Surface, Elevation, Plates, Temperature or Moisture")
            })?;
    }
    let globe = &showing.globe;
    let up_to_date = globe.built == (showing.choice.seed, showing.choice.generator)
        && globe.layer == showing.choice.layer;
    let filled = up_to_date && globe.fill.done();
    if let Some(path) = asked.path {
        let image = images
            .get(&globe.texture)
            .ok_or_else(|| refused("no picture yet"))?;
        let picture = image
            .clone()
            .try_into_dynamic()
            .map_err(|error| refused(format!("can't convert the picture: {error:?}")))?;
        picture
            .save(&path)
            .map_err(|error| refused(format!("can't save {path}: {error}")))?;
    }
    let width = globe.planet.surface.width();
    let shown = |spot| {
        let (node, transform) = nodes.get(showing.image).ok()?;
        let camera = places.get(globe.scene.camera).ok()?;
        let turn = places.get(globe.scene.globe).ok()?.rotation;
        let point = turn * on_globe(spot);
        if point.dot(camera.translation - point) <= 0.0 {
            return None;
        }
        let scale = node.inverse_scale_factor;
        let size = node.size() * scale;
        let corner = transform.translation * scale - size / 2.0;
        let at = (corner + on_picture(point, camera)? * size) * ui_scale.0;
        Some([at.x, at.y])
    };
    let gates: Vec<_> = globe
        .planet
        .gates
        .gates
        .iter()
        .map(|gate| {
            let at = shown(gate_spot((gate.x, gate.y), width));
            json!({ "x": gate.x, "y": gate.y, "round": gate.round, "shown_at": at })
        })
        .collect();
    Ok(json!({
        "filled": filled,
        "seed": showing.choice.seed.value(),
        "world_gates": gates,
        "gate_tries": globe.planet.gates.tries,
        "pointed": showing.pointed.map(|gate| json!({ "number": gate.number, "can_go": gate.can_go })),
    }))
}
