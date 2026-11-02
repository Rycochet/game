//! Drawing what is carried: a moveable panel per open grid, the button and key
//! for the pockets, the item being held, and its name.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
use shared::id::Id;
use shared::item::{Holder, Inventory, ItemKind, Placed, Rotation, Shape};

use super::body::{ClothesPanel, worn_notes};
use super::give::NearbyPanel;
use super::hold::Target;
use super::{Carried, Holding, item_cell_in_front};
use crate::common::{GameView, PointerPosition};
use crate::widgets::{
    ButtonPressed, ButtonState, INK, Pressable, RaisePanel, spawn_icon_button,
    spawn_moveable_panel, spawn_tooltip,
};

/// A grid cell, across, in logical pixels.
pub(super) const CELL: f32 = 32.0;

const BUTTON: f32 = 34.0;
const BIN: f32 = 36.0;
const CORNER_GAP: f32 = 10.0;
const FONT: f32 = 16.0;
const LABEL_FONT: f32 = 13.0;
const CELL_BORDER: f32 = 1.0;
/// Above moveable panels, below full-screen layers.
const HELD_Z: i32 = 900_000;
/// How far from the pointer an item's name goes.
const TOOLTIP_OFFSET: Vec2 = Vec2::new(16.0, 16.0);
/// How far above a held item its name goes, to its top.
const HELD_NAME_ABOVE: f32 = 48.0;
/// Where the first panel goes, and how far along each one after it.
const FIRST_AT: Vec2 = Vec2::new(90.0, 140.0);
const NEXT_AT: Vec2 = Vec2::new(36.0, 36.0);

pub(super) const GRID_COLOUR: Color = Color::srgb(0.55, 0.46, 0.33);
pub(super) const GRID_LINE: Color = Color::srgb(0.42, 0.34, 0.24);
pub(super) const ITEM_EDGE: Color = Color::srgb(0.15, 0.1, 0.05);
pub(super) const ALLOWED: Color = Color::srgba(0.2, 0.8, 0.25, 0.55);
pub(super) const REFUSED: Color = Color::srgba(0.85, 0.15, 0.15, 0.55);

/// The panels open, in the order they were opened.
#[derive(Resource, Default)]
pub(super) struct OpenGrids {
    /// Each with the size its grid was drawn at.
    open: Vec<(Holder, Entity, (u8, u8))>,
    opened: u32,
}

/// The grid of a panel: where its cells are, for working out what is under the
/// pointer.
#[derive(Component, Clone, Copy)]
pub(super) struct GridNode {
    pub holder: Holder,
}

/// In a grid, what holds the items drawn in it.
#[derive(Component, Clone, Copy)]
pub(super) struct ItemLayer {
    holder: Holder,
}

/// In a grid, what holds the colours saying what a drop would do.
#[derive(Component, Clone, Copy)]
pub(super) struct AimLayer {
    holder: Holder,
}

/// The words under a grid saying how heavy things are.
#[derive(Component, Clone, Copy)]
pub(super) struct WeightText {
    holder: Holder,
}

/// One cell of an item drawn in a grid: a press on it picks the item up by this
/// cell. Also on the label over it, for finding what a finger is on.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct ItemCell {
    pub id: Id,
    pub rotation: Rotation,
    pub grip: (u8, u8),
}

#[derive(Component)]
pub(super) struct PocketsButton;

#[derive(Component)]
pub(super) struct HeldItem;

#[derive(Component)]
pub(super) struct NameBox;

#[derive(Component)]
pub(super) struct NameText;

/// Follows the pointer above everything: not what the pointer is on.
#[derive(Component)]
pub(super) struct Floating;

/// The bin in the pockets' panel: what is dropped on it is destroyed.
#[derive(Component)]
pub(super) struct BinNode;

/// The bin's picture.
#[derive(Resource)]
pub(super) struct BinIcon(Handle<Image>);

/// What a kind of item looks like until it has a picture: its colour.
pub(super) fn colour(kind: ItemKind) -> Color {
    let index = ItemKind::ALL
        .iter()
        .position(|&other| other == kind)
        .unwrap_or(0);
    // Spread round the colour wheel by a step that doesn't repeat soon.
    Color::hsl((index as f32 * 67.0) % 360.0, 0.45, 0.5)
}

/// What it says on the first cell of an item.
pub(super) fn label(kind: ItemKind) -> String {
    kind.def().name.chars().take(2).collect()
}

fn kilograms(grams: u32) -> String {
    format!("{:.2} kg", grams as f32 / 1000.0)
}

/// A bag, drawn here so there is no file to ship for it.
fn bag_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (across, down) = (x as f32 - middle, y as f32 - middle);
            // A round body, and a neck tied above it.
            let body = across.hypot(down - 4.0) < 11.0;
            let neck = across.abs() < 3.5 && (-13.0..-5.0).contains(&down);
            let flare = across.abs() < 6.0 && (-14.0..-11.0).contains(&down);
            if body || neck || flare {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// A waste bin, drawn here so there is no file to ship for it.
fn bin_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (x, y) = (x as i32, y as i32);
            let handle = (13..19).contains(&x) && (4..6).contains(&y);
            let lid = (6..26).contains(&x) && (6..9).contains(&y);
            // Narrowing towards the bottom, with slots down it.
            let inset = (y - 10) / 5;
            let body = (10..28).contains(&y) && (8 + inset..24 - inset).contains(&x);
            let slot = (13..25).contains(&y) && (x == 12 || x == 15 || x == 16 || x == 19);
            if handle || lid || (body && !slot) {
                bytes[(y as usize * SIZE + x as usize) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// The button, and what stands above panels: the held item and an item's name.
pub(super) fn spawn_fixed_parts(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icon = images.add(bag_icon());
    commands.insert_resource(BinIcon(images.add(bin_icon())));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(CORNER_GAP),
                left: px(CORNER_GAP),
                ..default()
            },
            // Above the maps, as the warning button is.
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(BUTTON),
                height: px(BUTTON),
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node)
                .insert((PocketsButton, Pressable));
        });
    commands.spawn((
        HeldItem,
        Floating,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        GlobalZIndex(HELD_Z),
        Visibility::Hidden,
    ));
    commands
        .spawn((
            NameBox,
            Floating,
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            GlobalZIndex(HELD_Z + 1),
            Visibility::Hidden,
        ))
        .with_children(|holder| spawn_tooltip(holder, NameText, ""));
}

impl OpenGrids {
    /// Opens this grid's panel, or brings it to the front if it is open.
    pub(super) fn open(
        &mut self,
        holder: Holder,
        carried: &Inventory,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
    ) {
        if let Some(&(_, panel, _)) = self.open.iter().find(|(open, ..)| *open == holder) {
            raise.write(RaisePanel(panel));
            return;
        }
        let Some((width, height)) = carried.grid_size(holder) else {
            return;
        };
        let title = match holder {
            Holder::Pockets => "Pockets",
            Holder::Inside(id) => carried.find(id).map_or("", |item| item.kind.def().name),
        };
        let at = FIRST_AT + NEXT_AT * (self.opened % 8) as f32;
        self.opened += 1;
        let (panel, content) = spawn_moveable_panel(commands, title, at);
        commands.entity(content).with_children(|content| {
            content
                .spawn((
                    GridNode { holder },
                    Node {
                        width: px(width as f32 * CELL),
                        height: px(height as f32 * CELL),
                        ..default()
                    },
                    BackgroundColor(GRID_COLOUR),
                ))
                .with_children(|grid| {
                    for y in 0..height {
                        for x in 0..width {
                            grid.spawn((at_cell(x as f32, y as f32), BorderColor::from(GRID_LINE)));
                        }
                    }
                    grid.spawn((ItemLayer { holder }, covering()));
                    grid.spawn((AimLayer { holder }, covering()));
                });
            content
                .spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        WeightText { holder },
                        Text::default(),
                        TextFont::from_font_size(FONT),
                        TextColor(INK),
                    ));
                    if holder == Holder::Pockets {
                        row.spawn((
                            BinNode,
                            Node {
                                width: px(BIN),
                                height: px(BIN),
                                ..default()
                            },
                        ));
                    }
                });
        });
        self.open.push((holder, panel, (width, height)));
    }

    /// Whether the pockets' panel is open.
    fn pockets_open(&self) -> bool {
        self.open.iter().any(|(open, ..)| *open == Holder::Pockets)
    }

    fn toggle_pockets(
        &mut self,
        carried: &Inventory,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
    ) {
        let open = self
            .open
            .iter()
            .position(|(open, ..)| *open == Holder::Pockets);
        match open {
            Some(index) => {
                let (_, panel, _) = self.open.remove(index);
                commands.entity(panel).despawn();
            }
            None => self.open(Holder::Pockets, carried, commands, raise),
        }
    }
}

/// A node one cell across at this place in a grid.
fn at_cell(x: f32, y: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(x * CELL),
        top: px(y * CELL),
        width: px(CELL),
        height: px(CELL),
        border: UiRect::all(px(CELL_BORDER)),
        ..default()
    }
}

/// A node filling a grid, for its children to be placed in.
fn covering() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

/// Gives bins their picture, and shows whether dropping would destroy.
pub(super) fn show_bins(
    icon: Res<BinIcon>,
    holding: Res<Holding>,
    new_bins: Query<Entity, Added<BinNode>>,
    mut bins: Query<&mut BackgroundColor, With<BinNode>>,
    mut commands: Commands,
) {
    for bin in &new_bins {
        commands
            .entity(bin)
            .insert((ImageNode::new(icon.0.clone()), BackgroundColor(Color::NONE)));
    }
    let tint = if holding.aim == Some(Target::Bin) {
        REFUSED
    } else {
        Color::NONE
    };
    for mut background in &mut bins {
        background.set_if_neq(BackgroundColor(tint));
    }
}

/// The button or `I` opens or closes the pockets.
#[allow(clippy::too_many_arguments)]
pub(super) fn toggle_pockets(
    mut pressed: MessageReader<ButtonPressed>,
    button: Query<(), With<PocketsButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<State<GameView>>,
    carried: Res<Carried>,
    mut open: ResMut<OpenGrids>,
    mut clothes: ResMut<ClothesPanel>,
    mut nearby: ResMut<NearbyPanel>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
) {
    let by_button = pressed.read().any(|press| button.contains(press.0));
    let by_key = keys.just_pressed(KeyCode::KeyI);
    if (by_button || by_key) && *view.get() != GameView::Connecting {
        // The clothes go with the pockets, as things are dragged between them.
        if open.pockets_open() {
            clothes.close(&mut commands, &nodes);
            nearby.close(&mut commands, &nodes);
        } else {
            clothes.open(&mut commands, &mut raise, &nodes);
            nearby.open(&mut commands, &mut raise, &nodes);
        }
        open.toggle_pockets(&carried.0, &mut commands, &mut raise);
    }
}

/// Forgets panels closed by their own close buttons, closes those of bags no
/// longer carried, and draws the pockets again if the body changed their size.
pub(super) fn forget_closed(
    carried: Res<Carried>,
    mut open: ResMut<OpenGrids>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
) {
    let mut redraw_pockets = false;
    open.open.retain(|&(holder, panel, size)| {
        if !nodes.contains(panel) {
            return false;
        }
        let keep = carried.0.grid_size(holder) == Some(size);
        if !keep {
            commands.entity(panel).despawn();
            redraw_pockets |= holder == Holder::Pockets;
        }
        keep
    });
    if redraw_pockets {
        open.open(Holder::Pockets, &carried.0, &mut commands, &mut raise);
    }
}

/// One cell of an item, in a parent placing it: the parent's corner is the shape's.
fn spawn_cells(
    parent: &mut ChildSpawnerCommands,
    kind: ItemKind,
    shape: Shape,
    corner: (f32, f32),
    cell_extra: impl Fn(u8, u8) -> Option<ItemCell>,
) {
    let mut first = true;
    for (x, y) in shape.cells() {
        let mut node = parent.spawn((
            at_cell(corner.0 + x as f32, corner.1 + y as f32),
            BackgroundColor(colour(kind)),
            BorderColor::from(ITEM_EDGE),
        ));
        let cell = cell_extra(x, y);
        match cell {
            Some(cell) => node.insert((cell, Pressable)),
            None => node.insert(Floating),
        };
        if first {
            first = false;
            let mut text = node.commands();
            let id = text
                .spawn((
                    Text::new(label(kind)),
                    TextFont::from_font_size(LABEL_FONT),
                    TextColor(ITEM_EDGE),
                ))
                .id();
            match cell {
                Some(cell) => text.entity(id).insert(cell),
                None => text.entity(id).insert(Floating),
            };
            node.add_child(id);
        }
    }
}

/// Draws what is in each open grid again when it changes. What is held isn't
/// drawn where it was, though it is still there until dropped.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_grids(
    carried: Res<Carried>,
    holding: Res<Holding>,
    new_layers: Query<(), Added<ItemLayer>>,
    layers: Query<(Entity, &ItemLayer)>,
    mut weights: Query<(&WeightText, &mut Text)>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let held = holding.held.map(|held| held.item.id);
    if !carried.is_changed() && new_layers.is_empty() && *drawn_without == held {
        return;
    }
    *drawn_without = held;
    for (entity, layer) in &layers {
        commands.entity(entity).despawn_related::<Children>();
        let Some(grid) = carried.0.grid(layer.holder) else {
            continue;
        };
        commands.entity(entity).with_children(|layer| {
            for placed in grid.iter().filter(|placed| Some(placed.item.id) != held) {
                spawn_placed(layer, placed);
            }
        });
    }
    for (weight, mut text) in &mut weights {
        **text = weight_words(&carried.0, weight.holder);
    }
}

fn spawn_placed(layer: &mut ChildSpawnerCommands, placed: &Placed) {
    let (id, rotation) = (placed.item.id, placed.rotation);
    spawn_cells(
        layer,
        placed.item.kind,
        placed.shape(),
        (placed.x as f32, placed.y as f32),
        |x, y| {
            Some(ItemCell {
                id,
                rotation,
                grip: (x, y),
            })
        },
    );
}

fn weight_words(carried: &Inventory, holder: Holder) -> String {
    match holder {
        Holder::Pockets => format!("Carrying {}", kilograms(carried.total_weight())),
        Holder::Inside(id) => {
            let Some(item) = carried.find(id) else {
                return String::new();
            };
            let held: u32 = item
                .contents
                .iter()
                .map(|placed| placed.item.total_weight())
                .sum();
            match item.kind.def().container.and_then(|bag| bag.max_total) {
                Some(most) => format!("Holding {} of {}", kilograms(held), kilograms(most)),
                None => format!("Holding {}", kilograms(held)),
            }
        }
    }
}

/// Colours the cells a drop would take, green if it can go there and red if not.
pub(super) fn show_aim(
    carried: Res<Carried>,
    holding: Res<Holding>,
    layers: Query<(Entity, &AimLayer)>,
    mut commands: Commands,
    mut drawn: Local<Option<(Target, Rotation)>>,
) {
    let held = holding.held.map(|held| held.item);
    let aim = holding
        .aim
        .zip(held)
        .map(|(aim, held)| (aim, held.rotation));
    if *drawn == aim {
        return;
    }
    *drawn = aim;
    for (entity, _) in &layers {
        commands.entity(entity).despawn_related::<Children>();
    }
    let (Some((aim, rotation)), Some(held)) = (aim, held) else {
        return;
    };
    let Some(item) = carried.0.find(held.id) else {
        return;
    };
    let (holder, cells, allowed) = match aim {
        Target::Nowhere | Target::Bin | Target::Body { .. } | Target::Someone(_) => return,
        Target::Cells {
            holder,
            x,
            y,
            allowed,
        } => {
            let shape = item.kind.def().shape.turned(rotation);
            let cells: Vec<_> = shape
                .cells()
                .map(|(cell_x, cell_y)| (x + cell_x as i32, y + cell_y as i32))
                .collect();
            (holder, cells, allowed)
        }
        Target::Bag { bag, place } => {
            let Some((holder, placed)) = carried.0.placed(bag) else {
                return;
            };
            let cells = placed.cells().map(|(x, y)| (x as i32, y as i32)).collect();
            (holder, cells, place.is_some())
        }
    };
    let Some((width, height)) = carried.0.grid_size(holder) else {
        return;
    };
    let tint = if allowed { ALLOWED } else { REFUSED };
    for (entity, layer) in &layers {
        if layer.holder != holder {
            continue;
        }
        commands.entity(entity).with_children(|layer| {
            for &(x, y) in &cells {
                if (0..width as i32).contains(&x) && (0..height as i32).contains(&y) {
                    layer.spawn((at_cell(x as f32, y as f32), BackgroundColor(tint)));
                }
            }
        });
    }
}

/// The held item, at its true size, with the cell it was picked up by under the
/// pointer.
#[allow(clippy::type_complexity)]
pub(super) fn show_held(
    carried: Res<Carried>,
    holding: Res<Holding>,
    held_item: Single<(Entity, &mut Node, &mut Visibility), With<HeldItem>>,
    mut commands: Commands,
    mut drawn: Local<Option<(Id, Rotation)>>,
) {
    let (entity, mut node, mut visibility) = held_item.into_inner();
    let held = holding.held.map(|held| held.item);
    let item = held.and_then(|held| carried.0.find(held.id));
    let (Some(held), Some(item)) = (held, item) else {
        visibility.set_if_neq(Visibility::Hidden);
        if drawn.take().is_some() {
            commands.entity(entity).despawn_related::<Children>();
        }
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let now = Some((held.id, held.rotation));
    if *drawn != now {
        *drawn = now;
        commands.entity(entity).despawn_related::<Children>();
        let (kind, shape) = (item.kind, item.kind.def().shape.turned(held.rotation));
        commands.entity(entity).with_children(|parent| {
            spawn_cells(parent, kind, shape, (0.0, 0.0), |_, _| None);
        });
    }
    let grip = Vec2::new(held.grip.0 as f32 + 0.5, held.grip.1 as f32 + 0.5) * CELL;
    if let Some(at) = holding.at {
        let corner = at - grip;
        node.left = px(corner.x);
        node.top = px(corner.y);
    }
}

/// The name of what is under the pointer, or of what is held.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn show_tooltip(
    carried: Res<Carried>,
    holding: Res<Holding>,
    pointer: Res<PointerPosition>,
    open: Res<OpenGrids>,
    clothes: Res<ClothesPanel>,
    nodes: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&ItemCell>,
        ),
        (With<Node>, Without<NameBox>),
    >,
    name_box: Single<(&mut Node, &mut Visibility), With<NameBox>>,
    mut text: Single<&mut Text, With<NameText>>,
) {
    let (mut node, mut visibility) = name_box.into_inner();
    let shown = if let Some(held) = holding.held {
        Some(held.item.id)
    } else if open.open.is_empty() && !clothes.is_open() {
        None
    } else {
        pointer
            .0
            .and_then(|at| item_cell_in_front(at, nodes.iter()))
            .map(|cell| cell.id)
    };
    let name = shown.and_then(|id| {
        let item = carried.0.find(id)?;
        Some(format!(
            "{}{}",
            item.kind.def().name,
            worn_notes(&carried.0, id)
        ))
    });
    let (Some(name), Some(at)) = (name, pointer.0.or(holding.at)) else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let words = if let Some(Target::Someone(_)) = holding.aim {
        format!("Give {name}")
    } else if holding.aim == Some(Target::Bin) {
        format!("Destroy {name}")
    } else {
        name
    };
    if text.0 != words {
        text.0 = words;
    }
    node.left = px(at.x + TOOLTIP_OFFSET.x);
    node.top = match holding.held {
        // Above what is held, so as not to hide any of it.
        Some(held) => px(at.y - (held.item.grip.1 as f32 + 0.5) * CELL - HELD_NAME_ABOVE),
        None => px(at.y + TOOLTIP_OFFSET.y),
    };
}

/// The button is hidden while connecting, as the warning button is.
pub(super) fn show_button(
    view: Res<State<GameView>>,
    mut button: Single<&mut Visibility, With<PocketsButton>>,
) {
    button.set_if_neq(if *view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
}
