//! A colour picker: square swatches edge to edge in a box, wrapping onto more lines
//! like words, each a colour or a picture (a pattern, a gradient). The chosen one
//! has a flat border and a white one inside it.

use bevy::prelude::*;

mod sample;
pub use sample::SAMPLE_GRADIENT;
pub(super) use sample::make_sample_gradient;

use super::text_box::line_height;
use super::theme::{INK, RAISED_BACKGROUND, TextSize};
use super::{BUTTON_BORDER, ButtonPressed, ButtonState, ButtonTooltip, Pressable};

/// The white edge inside the chosen swatch's own.
const CHOSEN_EDGE: f32 = 2.0;
const CHOSEN_COLOUR: Color = Color::WHITE;

/// What one swatch shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Swatch {
    Colour(Color),
    /// Stretched to fill the square: for a pattern or a gradient.
    Picture(Handle<Image>),
}

/// The choices and which is chosen. Setting `selected` shows it, sending nothing.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct ColourPicker {
    pub swatches: Vec<Swatch>,
    /// Each swatch's name, for its tooltip (one per swatch).
    pub names: Vec<String>,
    pub selected: usize,
}

/// The player chose swatch `.1` of picker `.0`.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColourChosen(pub Entity, pub usize);

/// One swatch of a picker.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct SwatchOf {
    pub picker: Entity,
    pub index: usize,
}

/// What fills a swatch, inside its border: where the white edge goes.
#[derive(Component)]
pub(super) struct SwatchFill;

/// A picker filling `node`'s width (give it one), one line as tall as a dropdown
/// with writing of `size`. It looks like an active button, its swatches like
/// buttons standing in it.
pub fn spawn_colour_picker<'a>(
    parent: &'a mut ChildSpawnerCommands,
    picker: ColourPicker,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let side = line_height(size);
    let swatches: Vec<(Swatch, String)> = picker
        .swatches
        .iter()
        .cloned()
        .zip(picker.names.iter().cloned())
        .collect();
    let selected = picker.selected;
    let mut holder = parent.spawn((
        picker,
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            border: UiRect::all(px(BUTTON_BORDER)),
            ..node
        },
        BackgroundColor(RAISED_BACKGROUND),
        BorderColor::from(INK),
    ));
    let id = holder.id();
    holder.with_children(|holder| {
        for (index, (swatch, name)) in swatches.into_iter().enumerate() {
            let chosen = index == selected;
            let state = if chosen {
                ButtonState::Active
            } else {
                ButtonState::Normal
            };
            let (background, border, _) = state.colours();
            holder
                .spawn((
                    state,
                    SwatchOf { picker: id, index },
                    Pressable,
                    ButtonTooltip(name),
                    Node {
                        width: px(side),
                        height: px(side),
                        flex_shrink: 0.0,
                        border: UiRect::all(px(BUTTON_BORDER)),
                        ..default()
                    },
                    BackgroundColor(background),
                    border,
                ))
                .with_children(|button| {
                    let mut fill = button.spawn((
                        SwatchFill,
                        Node {
                            width: percent(100),
                            height: percent(100),
                            border: UiRect::all(px(chosen_edge(chosen))),
                            ..default()
                        },
                        BorderColor::from(CHOSEN_COLOUR),
                    ));
                    match swatch {
                        Swatch::Colour(colour) => fill.insert(BackgroundColor(colour)),
                        Swatch::Picture(picture) => {
                            fill.insert(ImageNode::new(picture).with_mode(NodeImageMode::Stretch))
                        }
                    };
                });
        }
    });
    holder
}

/// The white edge's width: only on the chosen swatch.
fn chosen_edge(chosen: bool) -> f32 {
    if chosen { CHOSEN_EDGE } else { 0.0 }
}

/// A press on a swatch chooses it.
pub(super) fn press_swatches(
    mut pressed: MessageReader<ButtonPressed>,
    swatches: Query<&SwatchOf>,
    mut pickers: Query<&mut ColourPicker>,
    mut chosen: MessageWriter<ColourChosen>,
) {
    for press in pressed.read() {
        let Ok(swatch) = swatches.get(press.0) else {
            continue;
        };
        if let Ok(mut picker) = pickers.get_mut(swatch.picker) {
            picker.selected = swatch.index;
            chosen.write(ColourChosen(swatch.picker, swatch.index));
        }
    }
}

/// Each swatch's look, by whether it is the chosen one.
pub(super) fn show_swatches(
    pickers: Query<&ColourPicker, Changed<ColourPicker>>,
    mut swatches: Query<(&SwatchOf, &mut ButtonState, &Children)>,
    mut fills: Query<&mut Node, With<SwatchFill>>,
) {
    for (swatch, mut state, children) in &mut swatches {
        let Ok(picker) = pickers.get(swatch.picker) else {
            continue;
        };
        let chosen = picker.selected == swatch.index;
        state.set_if_neq(if chosen {
            ButtonState::Active
        } else {
            ButtonState::Normal
        });
        for child in children.iter() {
            if let Ok(mut node) = fills.get_mut(child) {
                let edge = UiRect::all(px(chosen_edge(chosen)));
                if node.border != edge {
                    node.border = edge;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
