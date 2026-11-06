//! The kind's looks: a colour picker for each colour (hair, skin, coat, feathers),
//! with the chosen colour's name by its label, and a list for the rest (height).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::rng::Seed;
use shared::sheet::{Choice, Pattern, Trait};

use super::draft::Draft;
use super::form::LookList;
use super::screen::FORM;
use crate::widgets::{
    ColourPicker, Dropdown, Swatch, TextSize, UiId, field_label, spawn_colour_picker,
    spawn_dropdown,
};

/// A pattern's picture, across: small, as a swatch is.
const PATTERN_PIXELS: u32 = 16;
/// How many specks a speckled picture has.
const SPECKS: u32 = 14;

/// The words above a colour picker, saying what is chosen.
#[derive(Component)]
pub(super) struct LookLabel(pub Trait);

/// Pictures of patterns, painted once each.
#[derive(Resource, Default)]
pub(super) struct PatternPictures(HashMap<([u8; 3], Pattern), Handle<Image>>);

/// Fills the looks box: a picker or a list for each of the kind's looks.
pub(super) fn spawn_looks(
    looks: &mut ChildSpawnerCommands,
    draft: &Draft,
    pictures: &mut PatternPictures,
    images: &mut Assets<Image>,
) {
    for (index, &which) in draft.kind.def().looks.iter().enumerate() {
        let choices = which.choices_for(draft.kind);
        let selected = draft.appearance.0.get(&which).copied().unwrap_or(0) as usize;
        let id = UiId::new(FORM, "look").at(index);
        let swatches: Option<Vec<Swatch>> = choices
            .iter()
            .map(|choice| swatch(choice, pictures, images))
            .collect();
        let names = choices
            .iter()
            .map(|choice| choice.name.to_owned())
            .collect();
        match swatches {
            // Every choice is a colour: a picker, its label saying which.
            Some(swatches) => {
                looks.spawn((LookLabel(which), field_label(which.name())));
                let picker = ColourPicker {
                    swatches,
                    names,
                    selected,
                };
                spawn_colour_picker(looks, picker, TextSize::Medium, Node::default())
                    .insert((LookList(which), id));
            }
            None => {
                looks.spawn(field_label(which.name()));
                let list = Dropdown {
                    options: names,
                    selected,
                    ..default()
                };
                spawn_dropdown(looks, list, TextSize::Medium, Node::default())
                    .insert((LookList(which), id));
            }
        }
    }
}

/// What a choice's swatch shows, if it is a colour.
fn swatch(
    choice: &Choice,
    pictures: &mut PatternPictures,
    images: &mut Assets<Image>,
) -> Option<Swatch> {
    let [red, green, blue] = choice.colour?;
    Some(match choice.pattern {
        Pattern::Plain => Swatch::Colour(Color::srgb_u8(red, green, blue)),
        pattern => {
            let base = [red, green, blue];
            let picture = pictures
                .0
                .entry((base, pattern))
                .or_insert_with(|| images.add(pattern_picture(base, pattern)));
            Swatch::Picture(picture.clone())
        }
    })
}

/// A colour with its pattern over it, the same every time.
fn pattern_picture(base: [u8; 3], pattern: Pattern) -> Image {
    let side = PATTERN_PIXELS as usize;
    let mut bytes: Vec<u8> = (0..side * side)
        .flat_map(|_| [base[0], base[1], base[2], 255])
        .collect();
    if let Pattern::Speckled(specks) = pattern {
        let mut rng = Seed::new(0x5bec).rng();
        for _ in 0..SPECKS {
            let x = rng.range_u64(0, PATTERN_PIXELS as u64) as usize;
            let y = rng.range_u64(0, PATTERN_PIXELS as u64) as usize;
            // Each speck two pixels wide, so it shows when shrunk.
            for at in [(x, y), ((x + 1) % side, y)] {
                let index = (at.1 * side + at.0) * 4;
                bytes[index..index + 3].copy_from_slice(&specks);
            }
        }
    }
    Image::new(
        Extent3d {
            width: PATTERN_PIXELS,
            height: PATTERN_PIXELS,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Each colour's label says what is chosen: "Hair: Red".
pub(super) fn show_look_labels(draft: Res<Draft>, mut labels: Query<(&LookLabel, &mut Text)>) {
    for (label, mut text) in &mut labels {
        let chosen = draft.appearance.says(draft.kind, label.0);
        let words = format!("{}: {chosen}", label.0.name());
        if text.0 != words {
            text.0 = words;
        }
    }
}

#[cfg(test)]
mod tests;
