//! The Suns and Moons panel: changes the colours of this world's suns and moons,
//! and how strong the suns are. Only what this client draws.

use crate::widgets::UiId;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::celestial::{BodyKind, colour_along};

use crate::common::CurrentWorld;
use crate::dom::{Dom, Pictures, Template, document_in};
use crate::html;
use crate::widgets::{INK, SliderMoved};

/// The panel's form, for the debug API.
const FORM: &str = "debug.suns_and_moons";
/// How many colours a track shows, and how many are tried to place a mark.
const STEPS: u32 = 128;
/// What the end of the intensity slider means.
const MOST_INTENSITY: f64 = 2.0;

/// A panel's contents to fill, once there is a world to fill them from.
#[derive(Component)]
pub(super) struct BodiesToShow;

/// Which body a slider changes, and what about it.
#[derive(Component, Clone, Copy)]
pub(super) enum BodySlider {
    Colour(usize),
    Intensity(usize),
}

/// Shows the colour of this body.
#[derive(Component)]
pub(super) struct BodySwatch(usize);

fn to_colour([red, green, blue]: [f64; 3]) -> Color {
    Color::linear_rgb(red as f32, green as f32, blue as f32)
}

/// A picture to go along a slider, colours in order.
fn track_picture(images: &mut Assets<Image>, colour_at: impl Fn(f64) -> Color) -> Handle<Image> {
    let mut data = Vec::with_capacity(STEPS as usize * 4);
    for step in 0..STEPS {
        let colour = colour_at(f64::from(step) / f64::from(STEPS - 1)).to_srgba();
        data.extend_from_slice(&colour.to_u8_array());
    }
    images.add(Image::new(
        Extent3d {
            width: STEPS,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

/// Where along its slider a colour is nearest. A generated body's colour may be
/// paler than any there, so only the swatch shows it exactly until it is moved.
fn nearest_along(kind: BodyKind, colour: [f64; 3]) -> f32 {
    let distance = |step: u32| {
        let other = colour_along(kind, f64::from(step) / f64::from(STEPS - 1));
        (0..3).map(|i| (other[i] - colour[i]).powi(2)).sum::<f64>()
    };
    let best = (0..STEPS)
        .min_by(|&a, &b| distance(a).total_cmp(&distance(b)))
        .unwrap_or(0);
    best as f32 / (STEPS - 1) as f32
}

/// The panel's contents, as markup: a row per sun, a gap, a row per moon,
/// each a copy of its template, numbered by the body's place among the
/// world's. The sliders' tracks are pictures named by `fill_panels`.
static SUNS_AND_MOONS: Template = html!(
    "debug_menu.suns_and_moons",
    r#"
<form id="debug.suns_and_moons" class="rows">
  <div id="suns"/>
  <div id="moons"/>
  <template id="sun">
    <span nowrap valign="middle">
      <span id="name" width="60"/>
      <canvas id="swatch" width="18" height="18"/>
      <input id="colour" type="range" width="180" src="picture:sun_colours"/>
      <input id="intensity" type="range" width="180" src="picture:greys"/>
    </span>
  </template>
  <template id="moon">
    <span nowrap valign="middle">
      <span id="name" width="60"/>
      <canvas id="swatch" width="18" height="18"/>
      <input id="colour" type="range" width="180" src="picture:moon_colours"/>
    </span>
  </template>
</form>
"#
);

/// Fills a panel's contents, once there is a world to fill them from: the
/// document, then a row for each body, set to how it is.
pub(super) fn fill_panels(
    mut commands: Commands,
    world: Option<Res<CurrentWorld>>,
    to_fill: Query<Entity, With<BodiesToShow>>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
    mut dom: Dom,
    mut named: Local<bool>,
) {
    let Some(world) = world else {
        return;
    };
    for content in &to_fill {
        if !*named {
            *named = true;
            let sun = |t| to_colour(colour_along(BodyKind::Sun, t));
            pictures.name("sun_colours", track_picture(&mut images, sun));
            let moon = |t| to_colour(colour_along(BodyKind::Moon, t));
            pictures.name("moon_colours", track_picture(&mut images, moon));
            let grey = |t: f64| Color::linear_rgb(t as f32, t as f32, t as f32);
            pictures.name("greys", track_picture(&mut images, grey));
        }
        commands.entity(content).remove::<BodiesToShow>();
        commands.spawn(document_in(&SUNS_AND_MOONS, content));
        // Filled as it is built, at the end of this frame.
        let (mut suns, mut moons) = (0, 0);
        for (i, body) in world.celestial.bodies().iter().enumerate() {
            let (template, into, name) = match body.kind {
                BodyKind::Sun => {
                    suns += 1;
                    ("sun", "suns", format!("Sun {suns}"))
                }
                BodyKind::Moon => {
                    moons += 1;
                    ("moon", "moons", format!("Moon {moons}"))
                }
            };
            dom.copy_template(&format!("{FORM}/{template}"), &format!("{FORM}/{into}"), i);
            dom.set_text(&format!("{FORM}/name.{i}"), name);
            // A slider's value is a percentage of the way along it.
            let along = nearest_along(body.kind, body.colour) * 100.0;
            dom.set(&format!("{FORM}/colour.{i}"), "value", &along.to_string());
            if body.kind == BodyKind::Sun {
                let strength = world.celestial.sun_strength(i) / MOST_INTENSITY * 100.0;
                dom.set(
                    &format!("{FORM}/intensity.{i}"),
                    "value",
                    &strength.to_string(),
                );
            }
        }
    }
}

/// Gives each built slider the body it changes, and each swatch's place the
/// colour of its body, found by their ids.
pub(super) fn bind_bodies(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
    world: Option<Res<CurrentWorld>>,
) {
    for (entity, id) in &new {
        let ((FORM, name), Some(number)) = (id.parts(), id.index()) else {
            continue;
        };
        let mut bound = commands.entity(entity);
        match name {
            "colour" => bound.insert(BodySlider::Colour(number)),
            "intensity" => bound.insert(BodySlider::Intensity(number)),
            "swatch" => {
                let bodies = world.as_ref().map(|world| world.celestial.bodies());
                let colour = bodies.and_then(|bodies| bodies.get(number).map(|body| body.colour));
                bound.with_child((
                    BodySwatch(number),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(to_colour(colour.unwrap_or_default())),
                    BorderColor::from(INK),
                ))
            }
            _ => continue,
        };
    }
}

/// What the sliders do. `CurrentWorld` is only touched when one moves, as the
/// terrain's window is cut again when it changes.
pub(super) fn apply_sliders(
    mut moved: MessageReader<SliderMoved>,
    sliders: Query<&BodySlider>,
    world: Option<ResMut<CurrentWorld>>,
    mut swatches: Query<(&BodySwatch, &mut BackgroundColor)>,
) {
    let Some(mut world) = world else {
        return;
    };
    for &SliderMoved(entity, value) in moved.read() {
        match sliders.get(entity) {
            Ok(&BodySlider::Colour(number)) => {
                let Some(kind) = world.celestial.bodies().get(number).map(|body| body.kind) else {
                    continue;
                };
                let colour = colour_along(kind, f64::from(value));
                world.celestial.set_colour(number, colour);
                for (swatch, mut background) in &mut swatches {
                    if swatch.0 == number {
                        background.0 = to_colour(colour);
                    }
                }
            }
            Ok(&BodySlider::Intensity(number)) => {
                let strength = f64::from(value) * MOST_INTENSITY;
                world.celestial.set_sun_strength(number, strength);
            }
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests;
