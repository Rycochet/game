//! The Suns and Moons panel: changes the colours of this world's suns and moons,
//! and how strong the suns are. Only what this client draws.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::celestial::{Body, BodyKind, colour_along};

use crate::common::CurrentWorld;
use crate::widgets::{INK, SliderMoved, spawn_slider};

const FONT: f32 = 16.0;
const NAME_WIDTH: f32 = 60.0;
const SWATCH: f32 = 18.0;
const SLIDER_WIDTH: f32 = 180.0;
const ROW_GAP: f32 = 8.0;
/// Between the suns and the moons.
const SECTION_GAP: f32 = 14.0;
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

/// Fills a panel's contents: a row per sun, a gap, a row per moon.
pub(super) fn fill_panels(
    mut commands: Commands,
    world: Option<Res<CurrentWorld>>,
    to_fill: Query<Entity, With<BodiesToShow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(world) = world else {
        return;
    };
    for content in &to_fill {
        let sun_track = track_picture(&mut images, |t| to_colour(colour_along(BodyKind::Sun, t)));
        let moon_track = track_picture(&mut images, |t| to_colour(colour_along(BodyKind::Moon, t)));
        let intensity_track = track_picture(&mut images, |t| {
            let shade = t as f32;
            Color::linear_rgb(shade, shade, shade)
        });
        let bodies = world.celestial.bodies();
        commands
            .entity(content)
            .remove::<BodiesToShow>()
            .with_children(|content| {
                let (mut suns, mut moons) = (0, 0);
                for (i, body) in bodies.iter().enumerate() {
                    if body.kind == BodyKind::Moon && moons == 0 && suns > 0 {
                        content.spawn(Node {
                            height: px(SECTION_GAP),
                            ..default()
                        });
                    }
                    let (name, track) = match body.kind {
                        BodyKind::Sun => {
                            suns += 1;
                            (format!("Sun {suns}"), sun_track.clone())
                        }
                        BodyKind::Moon => {
                            moons += 1;
                            (format!("Moon {moons}"), moon_track.clone())
                        }
                    };
                    let intensity = (body.kind == BodyKind::Sun).then(|| {
                        (
                            (world.celestial.sun_strength(i) / MOST_INTENSITY) as f32,
                            intensity_track.clone(),
                        )
                    });
                    spawn_row(content, i, name, body, track, intensity);
                }
            });
    }
}

fn spawn_row(
    content: &mut ChildSpawnerCommands,
    number: usize,
    name: String,
    body: &Body,
    track: Handle<Image>,
    intensity: Option<(f32, Handle<Image>)>,
) {
    content
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(ROW_GAP),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(name),
                TextFont::from_font_size(FONT),
                TextColor(INK),
                Node {
                    width: px(NAME_WIDTH),
                    ..default()
                },
            ));
            row.spawn((
                BodySwatch(number),
                Node {
                    width: px(SWATCH),
                    height: px(SWATCH),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(to_colour(body.colour)),
                BorderColor::from(INK),
            ));
            let value = nearest_along(body.kind, body.colour);
            spawn_slider(row, value, Some(track), SLIDER_WIDTH).insert(BodySlider::Colour(number));
            if let Some((value, track)) = intensity {
                spawn_slider(row, value, Some(track), SLIDER_WIDTH)
                    .insert(BodySlider::Intensity(number));
            }
        });
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
mod tests {
    use super::*;

    #[test]
    fn a_colour_from_a_slider_finds_its_place_again() {
        for kind in [BodyKind::Sun, BodyKind::Moon] {
            for step in [0.0, 0.25, 0.6, 0.85] {
                let found = nearest_along(kind, colour_along(kind, step));
                assert!((f64::from(found) - step).abs() < 0.02, "{kind:?} {step}");
            }
        }
    }
}
