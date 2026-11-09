//! The bar across the top: which generator, which seed (typed, random or
//! copied), Reset (back to the current world as it is), which layer the
//! picture shows, and Teleport to World....

use bevy::prelude::*;
use shared::surface::SurfaceGenerator;

use super::layer::Layer;
use super::showing::Choice;
use crate::widgets::UiId;
use crate::widgets::{
    BUTTON_PADDING, ButtonState, Dropdown, HoverTip, INK, PANEL_EDGE, Pressable, TextBox, TextSize,
    field_label, panel, spawn_button, spawn_dropdown, spawn_new_seed_button, spawn_text_box,
    stops_presses,
};

const FONT: TextSize = TextSize::Medium;
/// Wide enough for any `u64`.
const SEED_WIDTH: f32 = 230.0;
const ROW_HEIGHT: f32 = 32.0;
const GAP: f32 = 10.0;
/// Wide enough for the longest option and the arrow.
const DROPDOWN_WIDTH: f32 = 140.0;

/// The bar's controls.
pub(super) struct Bar {
    /// The bar itself: the pointer on it is on none of the globe.
    pub(super) root: Entity,
    pub(super) generator: Entity,
    pub(super) seed: Entity,
    pub(super) random: Entity,
    /// Natively only: browsers have no clipboard for the game yet.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) copy: Entity,
    pub(super) reset: Entity,
    pub(super) layer: Entity,
    /// Teleport to World... (`teleport`).
    pub(super) teleport: Entity,
    /// How the World Gates were found.
    pub(super) gate_words: Entity,
}

pub(super) fn spawn_bar(layer: &mut ChildSpawnerCommands, choice: Choice) -> Bar {
    let mut bar = layer.spawn(panel());
    bar.insert(Node {
        position_type: PositionType::Absolute,
        top: px(16),
        border: UiRect::all(px(PANEL_EDGE)),
        padding: UiRect::axes(px(GAP), px(6)),
        column_gap: px(GAP),
        align_items: AlignItems::Center,
        ..default()
    });
    let entity = bar.id();
    bar.insert(stops_presses(entity));
    let mut parts = None;
    bar.with_children(|row| {
        let control = |width: Option<f32>| Node {
            width: width.map_or(Val::Auto, px),
            height: px(ROW_HEIGHT),
            padding: BUTTON_PADDING,
            ..default()
        };
        row.spawn(field_label("Generator"));
        let names = SurfaceGenerator::ALL.map(|kind| kind.name().to_string());
        let generator = dropdown(
            UiId::new("planet", "generator"),
            row,
            names.to_vec(),
            choice.generator,
            &SurfaceGenerator::ALL,
        );
        row.spawn(field_label("Seed"));
        let seed_box = TextBox {
            text: choice.seed.value().to_string(),
            digits_only: true,
            takes_keys: true,
            ..default()
        };
        let node = Node {
            padding: UiRect::ZERO,
            ..control(Some(SEED_WIDTH))
        };
        let seed = spawn_text_box(row, seed_box, FONT, node)
            .insert(UiId::new("planet", "seed"))
            .id();
        let random = spawn_new_seed_button(row, ROW_HEIGHT)
            .insert(UiId::new("planet", "new_seed"))
            .id();
        #[cfg(not(target_arch = "wasm32"))]
        let copy = {
            let node = Node {
                width: px(ROW_HEIGHT),
                height: px(ROW_HEIGHT),
                ..default()
            };
            crate::widgets::spawn_copy_button(row, "Copy the seed", FONT, node)
                .insert((Pressable, UiId::new("planet", "copy")))
                .id()
        };
        let reset = spawn_button(row, ButtonState::Normal, "Reset", FONT, control(None))
            .insert((Pressable, UiId::new("planet", "reset")))
            .id();
        row.spawn(field_label("Layer"));
        let names = Layer::ALL.map(|layer| layer.name().to_string());
        let id = UiId::new("planet", "layer");
        let layer = dropdown(id, row, names.to_vec(), choice.layer, &Layer::ALL);
        let teleport = spawn_button(
            row,
            ButtonState::Normal,
            "Teleport to World...",
            FONT,
            control(None),
        )
        .insert((
            Pressable,
            HoverTip(String::new()),
            UiId::new("planet", "teleport"),
        ))
        .id();
        let gate_words = row
            .spawn((Text::new(""), TextSize::Small.font(), TextColor(INK)))
            .id();
        parts = Some(Bar {
            root: entity,
            generator,
            seed,
            random,
            #[cfg(not(target_arch = "wasm32"))]
            copy,
            reset,
            layer,
            teleport,
            gate_words,
        });
    });
    parts.expect("the bar's children are spawned straight away")
}

fn dropdown<T: PartialEq>(
    id: UiId,
    row: &mut ChildSpawnerCommands,
    options: Vec<String>,
    chosen: T,
    all: &[T],
) -> Entity {
    let dropdown = Dropdown {
        options,
        selected: all.iter().position(|kind| *kind == chosen).unwrap_or(0),
        ..default()
    };
    let node = Node {
        min_width: px(DROPDOWN_WIDTH),
        padding: BUTTON_PADDING,
        ..default()
    };
    spawn_dropdown(row, dropdown, FONT, node).insert(id).id()
}
