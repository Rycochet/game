//! The screen's parts: a full-screen layer with the choices on the left and the
//! triangles on the right, spawned once and shown while a character is made.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::sheet::Attribute;

use super::draft::Draft;
use super::picture::PIXELS;
use crate::widgets::{
    ButtonState, ButtonStyle, ButtonTooltip, Dropdown, INK, Pressable, TextBox, TextSize, UiId,
    field_label, panel_title, spawn_button, spawn_dropdown, spawn_fullscreen_layer,
    spawn_new_seed_button, spawn_text_box,
};

/// The form all its widgets' ids are in.
pub(super) const FORM: &str = "character_create";
/// How big the triangles are drawn, in interface pixels.
pub(super) const CHOOSER_SIDE: f32 = 480.0;
const COLUMN_WIDTH: f32 = 300.0;
/// The random name button: as tall as the name box.
const RANDOM_SIDE: f32 = 24.0;

#[derive(Component)]
pub(super) struct CreateLayer;
#[derive(Component)]
pub(super) struct NameBox;
#[derive(Component)]
pub(super) struct KindList;
/// The button beside the name box suggesting another.
#[derive(Component)]
pub(super) struct RandomName;
/// Holds a list for each of the kind's looks, filled again for each kind.
#[derive(Component)]
pub(super) struct LooksBox;
#[derive(Component)]
pub(super) struct ValuesText;
#[derive(Component)]
pub(super) struct BeginButton;
/// The picture of the triangles, which takes presses on them.
#[derive(Component)]
pub(super) struct Chooser(pub Handle<Image>);
/// A short name by its corner of a triangle.
#[derive(Component)]
pub(super) struct CornerLabel(pub Attribute);

pub(super) fn spawn_screen(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let picture = images.add(Image::new_fill(
        Extent3d {
            width: PIXELS,
            height: PIXELS,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    let content = Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: px(32),
        padding: UiRect::all(px(24)),
        ..default()
    };
    let (layer, panel) = spawn_fullscreen_layer(&mut commands, 0, content);
    commands.entity(layer).insert(CreateLayer);
    commands.entity(panel).with_children(|panel| {
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                width: px(COLUMN_WIDTH),
                row_gap: px(8),
                ..default()
            })
            .with_children(|column| spawn_choices(column, &Draft::default()));
        spawn_chooser(panel, picture);
    });
}

/// The left: name, kind, looks, what the attributes come to, and Begin.
fn spawn_choices(column: &mut ChildSpawnerCommands, draft: &Draft) {
    column.spawn((panel_title("New character"), Node::default()));
    column.spawn(field_label("Name"));
    let name = TextBox {
        takes_keys: true,
        ..default()
    };
    column
        .spawn(Node {
            column_gap: px(4),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            let grows = Node {
                flex_grow: 1.0,
                ..default()
            };
            spawn_text_box(row, name, TextSize::Medium, grows)
                .insert((NameBox, UiId::new(FORM, "name")));
            spawn_new_seed_button(row, RANDOM_SIDE).insert((
                RandomName,
                ButtonTooltip("Random name".into()),
                UiId::new(FORM, "random_name"),
            ));
        });
    column.spawn(field_label("Kind"));
    let kinds = Draft::playable();
    let options = kinds
        .iter()
        .map(|kind| kind.def().name.to_owned())
        .collect();
    let selected = kinds
        .iter()
        .position(|&kind| kind == draft.kind)
        .unwrap_or(0);
    let list = Dropdown {
        options,
        selected,
        ..default()
    };
    spawn_dropdown(column, list, TextSize::Medium, Node::default())
        .insert((KindList, UiId::new(FORM, "kind")));
    column.spawn((
        LooksBox,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
    ));
    column.spawn((
        ValuesText,
        Text::new(""),
        TextSize::Small.font(),
        TextColor(INK),
        Node {
            margin: UiRect::vertical(px(8)),
            ..default()
        },
    ));
    spawn_button(
        column,
        ButtonState::Disabled,
        "Begin",
        TextSize::Large,
        Node::default(),
    )
    .insert((
        BeginButton,
        ButtonStyle::Main,
        Pressable,
        UiId::new(FORM, "begin"),
    ));
}

/// The right: the triangles' picture, with their corners' names over it.
fn spawn_chooser(panel: &mut ChildSpawnerCommands, picture: Handle<Image>) {
    panel
        .spawn((
            Chooser(picture.clone()),
            ImageNode::new(picture),
            Pressable,
            UiId::new(FORM, "triangles"),
            Node {
                width: px(CHOOSER_SIDE),
                height: px(CHOOSER_SIDE),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|chooser| {
            for which in Attribute::ALL {
                chooser.spawn((
                    CornerLabel(which),
                    Text::new(&which.name()[..3]),
                    TextSize::Small.font(),
                    TextColor(INK),
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                ));
            }
        });
}

/// Fills the looks box with a list for each of the kind's looks.
pub(super) fn spawn_looks(looks: &mut ChildSpawnerCommands, draft: &Draft) {
    for (index, &which) in draft.kind.def().looks.iter().enumerate() {
        looks.spawn(field_label(which.name()));
        let options = which
            .choices_for(draft.kind)
            .iter()
            .map(|choice| choice.name.to_owned())
            .collect();
        let selected = draft.appearance.0.get(&which).copied().unwrap_or(0) as usize;
        let list = Dropdown {
            options,
            selected,
            ..default()
        };
        let id = UiId::new(FORM, "look").at(index);
        spawn_dropdown(looks, list, TextSize::Medium, Node::default())
            .insert((super::form::LookList(which), id));
    }
}
