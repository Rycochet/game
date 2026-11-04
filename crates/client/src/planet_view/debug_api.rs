//! `client.planet`: with View Planet open, choose its generator, seed and
//! layer, and save its picture of the surface (as far as it is filled), so a
//! generator can be looked at closely without clicking through the bar.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params, refused};
use shared::rng::Seed;
use shared::surface::SurfaceGenerator;

use super::layer::Layer;
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
fn planet(
    In(request): In<Option<Value>>,
    mut view: ResMut<PlanetView>,
    images: Res<Assets<Image>>,
) -> BrpResult {
    let asked: Asked = params(request)?;
    let Some(showing) = view.showing.as_mut() else {
        return Err(refused("View Planet isn't open"));
    };
    if let Some(name) = asked.generator {
        showing.choice.generator = SurfaceGenerator::ALL
            .into_iter()
            .find(|kind| kind.name().eq_ignore_ascii_case(&name))
            .ok_or_else(|| refused("generator is Classic or Continents"))?;
    }
    if let Some(seed) = asked.seed {
        showing.choice.seed = Seed::new(seed);
    }
    if let Some(name) = asked.layer {
        showing.choice.layer = Layer::ALL
            .into_iter()
            .find(|layer| layer.name().eq_ignore_ascii_case(&name))
            .ok_or_else(|| refused("layer is Surface or Elevation"))?;
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
    Ok(json!({ "filled": filled, "seed": showing.choice.seed.value() }))
}
