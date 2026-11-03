//! The moveable panels the menu opens: Previous and Next for what a place draws,
//! and one showing every widget.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::body::Frame;
use shared::creature::{CreatureKind, Size};
use shared::dungeon::Cell;
use shared::id::Id;
use shared::item::{Favour, Item, ItemKind, Quality};
use shared::item_form::{Cut, GemForm, give_form};
use shared::material::Material;
use shared::poi::PoiKind;
use shared::rng::Seed;
use shared::site::{Generator, MAX_RUIN, StairsStyle};

use shared::protocol::ClientMessage;
use shared::world::MINUTES_PER_HOUR;

use super::actions::{DebugAction, kind_name};
use super::bodies::BodiesToShow;
use crate::common::{CurrentDungeon, GameView, HerbGrowthShift, Preferences, ServerClock};
use crate::connection::ToServer;
use crate::dungeon_map::{CROP_STAGES, HERB_STAGES};
use crate::item_art::{ItemPictures, Look, look_of};
use crate::widgets::{
    ALLOWED, BUTTON_PADDING, ButtonPressed, ButtonState, ButtonStyle, Dropdown, DropdownChosen,
    GRID_COLOUR, INK, Pressable, RaisePanel, TabChosen, TextBox, TextSize, field_label, raised_box,
    spawn_button, spawn_check_button, spawn_close_button, spawn_dropdown, spawn_icon_button,
    spawn_moveable_panel, spawn_progress_bar, spawn_scroll_area, spawn_slider, spawn_tabs,
    spawn_text_box, spawn_toggle_row, spawn_tooltip,
};
use shared::blade::Family;

mod armour;

const FONT: TextSize = TextSize::Medium;
/// How much of the window's height Show Widgets may take before it scrolls.
const EVERY_WIDGET_HEIGHT: f32 = 80.0;
const CHECK_ROW: f32 = 34.0;
/// Where the first panel goes, and how far along each one after it.
const FIRST_AT: Vec2 = Vec2::new(60.0, 110.0);
const NEXT_AT: Vec2 = Vec2::new(30.0, 30.0);

/// The panels open, by what opened them.
#[derive(Resource, Default)]
pub(super) struct OpenPanels {
    panels: HashMap<DebugAction, Entity>,
    opened: u32,
}

/// A button that moves what `action`'s panel is for on or back a step.
#[derive(Component, Clone, Copy)]
pub(super) struct StepButton {
    action: DebugAction,
    forward: bool,
}

impl OpenPanels {
    pub(super) fn is_open(&self, action: DebugAction) -> bool {
        self.panels.contains_key(&action)
    }

    /// Closes this action's panel if it is open, otherwise opens it.
    pub(super) fn toggle(
        &mut self,
        action: DebugAction,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
        nodes: &Query<(), With<Node>>,
    ) {
        match self.panels.remove(&action) {
            Some(panel) if nodes.contains(panel) => commands.entity(panel).despawn(),
            _ => self.open(action, commands, raise, nodes),
        }
    }

    /// Opens this action's panel, or if it is open already, brings it to the
    /// front. `nodes` says which panels are still there: closing despawns them.
    pub(super) fn open(
        &mut self,
        action: DebugAction,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
        nodes: &Query<(), With<Node>>,
    ) {
        if let Some(&panel) = self.panels.get(&action)
            && nodes.contains(panel)
        {
            raise.write(RaisePanel(panel));
            return;
        }
        let at = FIRST_AT + NEXT_AT * (self.opened % 8) as f32;
        self.opened += 1;
        let (panel, content) = spawn_moveable_panel(commands, action.name(), at);
        commands
            .entity(content)
            .with_children(|content| match action {
                DebugAction::ShowWidgets => spawn_every_widget(content),
                DebugAction::SunsAndMoons => {}
                DebugAction::GiveItem => spawn_give_buttons(content),
                DebugAction::ChangeCreature => spawn_creature_buttons(content),
                DebugAction::ItemPictures => spawn_item_pictures(content),
                _ => spawn_step_buttons(content, action),
            });
        if action == DebugAction::SunsAndMoons {
            commands.entity(content).insert(BodiesToShow);
        }
        self.panels.insert(action, panel);
    }
}

fn spawn_step_buttons(content: &mut ChildSpawnerCommands, action: DebugAction) {
    content
        .spawn(Node {
            column_gap: px(4),
            ..default()
        })
        .with_children(|row| {
            let [back, on] = if action == DebugAction::WorldTime {
                ["-1 hour", "+1 hour"]
            } else {
                ["Previous", "Next"]
            };
            for (label, forward) in [(back, false), (on, true)] {
                let node = Node {
                    padding: BUTTON_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, FONT, node)
                    .insert((StepButton { action, forward }, Pressable));
            }
        });
}

/// How wide the rows of buttons wrap, so a long list stays on the screen.
const WRAPPED_WIDTH: f32 = 520.0;

fn wrapped() -> Node {
    Node {
        width: px(WRAPPED_WIDTH),
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(4),
        row_gap: px(4),
        ..default()
    }
}

/// How new items are made, and a button for each kind.
fn spawn_give_buttons(content: &mut ChildSpawnerCommands) {
    let sizes: Vec<String> = std::iter::once("Mine".to_owned())
        .chain(Size::ALL.iter().map(|size| format!("{size:?}")))
        .collect();
    let sizes: Vec<&str> = sizes.iter().map(String::as_str).collect();
    let row = spawn_toggle_row(content, &sizes, 0, FONT);
    content.commands().entity(row).insert(GiveSizes);
    let row = spawn_toggle_row(content, &["Plain", "Magical"], 0, FONT);
    content.commands().entity(row).insert(GiveMagic);
    let row = spawn_toggle_row(content, &["Plain", "Blessed", "Cursed"], 0, FONT);
    content.commands().entity(row).insert(GiveFavour);
    let row = spawn_toggle_row(content, &["Alone", "Set"], 0, FONT);
    content.commands().entity(row).insert(GiveSet);
    let qualities: Vec<String> = Quality::ALL.iter().map(|q| format!("{q:?}")).collect();
    let qualities: Vec<&str> = qualities.iter().map(String::as_str).collect();
    let common = Quality::ALL
        .iter()
        .position(|&quality| quality == Quality::default())
        .unwrap_or(0);
    let row = spawn_toggle_row(content, &qualities, common, FONT);
    content.commands().entity(row).insert(GiveQualities);
    content.spawn(wrapped()).with_children(|buttons| {
        for kind in ItemKind::ALL {
            let node = Node {
                padding: BUTTON_PADDING,
                ..default()
            };
            spawn_button(buttons, ButtonState::Normal, kind.def().name, FONT, node)
                .insert((GiveButton(kind), Pressable));
        }
    });
}

/// A button for each kind of creature.
fn spawn_creature_buttons(content: &mut ChildSpawnerCommands) {
    for kind in CreatureKind::ALL {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, kind.def().name, FONT, node)
            .insert((CreatureButton(kind), Pressable));
    }
}

/// How big each picture in Item Pictures' grid is shown, and is drawn, for sharpness.
const PICTURE: f32 = 50.0;
const PICTURE_PIXELS: u32 = 100;
/// How long a weapon is shown in Item Pictures' grid.
const BLADE_PICTURE: f32 = 150.0;
/// How wide a pair of gauntlets is shown in Item Pictures' grid.
const HANDS_PICTURE: f32 = 100.0;
const LARGE_PICTURE: f32 = 256.0;
const LARGE_PICTURE_PIXELS: u32 = 512;
const PICTURE_LABEL: TextSize = TextSize::Small;
/// How many rings Item Pictures shows: a sample of what the server would roll.
const RING_ROWS: u128 = 8;
const RING_COLUMNS: u128 = 8;
/// The most of the window's height the grid takes, before it scrolls.
const PICTURES_HEIGHT: Val = Val::Vh(70.0);
/// Room for the longest page's name in the dropdown, beside its arrow.
const PAGES_WIDTH: f32 = 160.0;
/// Room for the name of a gem beside its row.
const PICTURE_ROW_LABEL: f32 = 84.0;

/// What Item Pictures shows.
#[derive(Resource)]
pub(super) struct PicturesForm {
    quality: Quality,
    chosen: Shown,
}

impl Default for PicturesForm {
    fn default() -> Self {
        Self {
            quality: Quality::default(),
            chosen: Shown::Gem(gem_to_show(Material::Ruby, Cut::Round)),
        }
    }
}

/// One of each stone and cut. Each uncut stone has its own lumps.
fn gem_to_show(material: Material, cut: Cut) -> GemForm {
    let shape_seed = if cut == Cut::Uncut {
        material as u16 + 1
    } else {
        0
    };
    GemForm {
        material,
        cut,
        shape_seed,
    }
}

/// What Item Pictures' grid shows: one kind of item at a time, chosen from a dropdown
/// and kept in `Preferences` by name.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) enum PicturePage {
    Bodies,
    Hands,
    Shoulders,
    Torso,
    Head,
    Weapons,
    #[default]
    Gems,
    Rings,
}

impl PicturePage {
    const ALL: [Self; 8] = [
        Self::Bodies,
        Self::Hands,
        Self::Shoulders,
        Self::Torso,
        Self::Head,
        Self::Weapons,
        Self::Gems,
        Self::Rings,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Bodies => "Bodies",
            Self::Hands => "Hands",
            Self::Shoulders => "Shoulders",
            Self::Torso => "Torso",
            Self::Head => "Head",
            Self::Weapons => "Weapons",
            Self::Gems => "Gems",
            Self::Rings => "Rings",
        }
    }

    /// The page last chosen, or Gems if none was, or it isn't one any more.
    fn chosen(preferences: &Preferences) -> Self {
        let name = preferences.item_pictures.as_deref();
        Self::ALL
            .into_iter()
            .find(|page| Some(page.name()) == name)
            .unwrap_or_default()
    }

    /// This page's rows.
    fn spawn(self, grid: &mut ChildSpawnerCommands) {
        match self {
            Self::Bodies => spawn_body_row(grid),
            Self::Hands => armour::spawn_hands_rows(grid),
            Self::Shoulders => armour::spawn_shoulders_rows(grid),
            Self::Torso => armour::spawn_torso_rows(grid),
            Self::Head => armour::spawn_head_rows(grid),
            Self::Weapons => Family::ALL
                .into_iter()
                .for_each(|f| spawn_blade_row(grid, f)),
            Self::Gems => {
                spawn_picture_heading(grid);
                for material in Material::GEMS {
                    spawn_picture_row(grid, material);
                }
            }
            Self::Rings => (0..RING_ROWS).for_each(|row| spawn_ring_row(grid, row)),
        }
    }
}

/// What one picture in Item Pictures is of.
#[derive(Clone, Copy)]
enum Shown {
    Gem(GemForm),
    /// A ring as the server would roll it from this number, at the quality chosen.
    Ring(u128),
    /// A weapon of this family, likewise.
    Blade(Family, u128),
    /// A bare body to dress.
    Body(CreatureKind),
    /// A piece of armour.
    Armour(armour::ArmourShown),
}

impl Shown {
    fn look(self, quality: Quality) -> Option<Look> {
        match self {
            Self::Gem(gem) => Some(Look::Gem {
                gem,
                quality: Some(quality),
            }),
            Self::Ring(seed) => {
                let mut ring = Item::new(Id::for_test(seed), ItemKind::Ring);
                ring.quality = Some(quality);
                let mut gems = seed << 32;
                give_form(&mut ring, Some(Size::Medium), &mut || {
                    gems += 1;
                    Id::for_test(gems)
                });
                look_of(&ring)
            }
            Self::Blade(family, seed) => {
                let mut blade = Item::new(Id::for_test(seed), family.kind());
                blade.quality = Some(quality);
                give_form(&mut blade, Some(Size::Medium), &mut || Id::for_test(0));
                look_of(&blade)
            }
            Self::Body(kind) => Frame::of(kind).map(|_| Look::Body { kind }),
            Self::Armour(shown) => shown.look(quality),
        }
    }
}

/// A picture in Item Pictures' grid: pressing it shows it large.
#[derive(Component, Clone, Copy)]
pub(super) struct GemPicture(Shown);

#[derive(Component)]
pub(super) struct LargePicture;

#[derive(Component)]
pub(super) struct PictureQualities;

/// The dropdown choosing Item Pictures' page.
#[derive(Component)]
pub(super) struct PicturePages;

/// Holds the rows of Item Pictures' page, and says which page they are, once filled.
#[derive(Component, Default)]
pub(super) struct PicturesGrid(Option<PicturePage>);

/// The page of pictures last chosen, filled in by `fill_pictures`, and whichever
/// picture was last pressed, large.
fn spawn_item_pictures(content: &mut ChildSpawnerCommands) {
    let qualities: Vec<String> = Quality::ALL.iter().map(|q| format!("{q:?}")).collect();
    let qualities: Vec<&str> = qualities.iter().map(String::as_str).collect();
    let common = Quality::ALL
        .iter()
        .position(|&quality| quality == Quality::default())
        .unwrap_or(0);
    content
        .spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|choices| {
            let pages = Dropdown {
                options: PicturePage::ALL.map(|page| page.name().to_owned()).to_vec(),
                selected: 0,
                searchable: false,
            };
            let node = Node {
                padding: BUTTON_PADDING,
                min_width: px(PAGES_WIDTH),
                ..default()
            };
            spawn_dropdown(choices, pages, FONT, node).insert(PicturePages);
            let row = spawn_toggle_row(choices, &qualities, common, FONT);
            choices.commands().entity(row).insert(PictureQualities);
        });
    content
        .spawn(Node {
            column_gap: px(8),
            ..default()
        })
        .with_children(|sides| {
            sides
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|column| {
                    let area = spawn_scroll_area(column, PICTURES_HEIGHT);
                    column.commands().entity(area).with_children(|area| {
                        area.spawn((
                            PicturesGrid::default(),
                            Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                        ));
                    });
                });
            sides.spawn((
                LargePicture,
                Node {
                    width: px(LARGE_PICTURE),
                    height: px(LARGE_PICTURE),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::default(),
            ));
        });
}

fn picture_label(words: &str, width: f32) -> impl Bundle {
    (
        Text::new(words),
        PICTURE_LABEL.font(),
        TextColor(INK),
        Node {
            width: px(width),
            ..default()
        },
    )
}

fn spawn_picture_heading(grid: &mut ChildSpawnerCommands) {
    grid.spawn(Node::default()).with_children(|row| {
        row.spawn(picture_label("", PICTURE_ROW_LABEL));
        for cut in Cut::ALL {
            row.spawn(picture_label(cut.name(), PICTURE));
        }
    });
}

fn spawn_picture_row(grid: &mut ChildSpawnerCommands, material: Material) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn(picture_label(material.name(), PICTURE_ROW_LABEL));
        for cut in Cut::ALL {
            spawn_picture(row, Shown::Gem(gem_to_show(material, cut)));
        }
    });
}

fn spawn_ring_row(grid: &mut ChildSpawnerCommands, row: u128) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label(
            if row == 0 { "Rings" } else { "" },
            PICTURE_ROW_LABEL,
        ));
        for cell in 0..RING_COLUMNS {
            spawn_picture(cells, Shown::Ring(row * RING_COLUMNS + cell + 1));
        }
    });
}

/// Every kind that stands on two legs, bare.
fn spawn_body_row(grid: &mut ChildSpawnerCommands) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label("Bodies", PICTURE_ROW_LABEL));
        let bipeds = CreatureKind::ALL
            .into_iter()
            .filter(|&kind| Frame::of(kind).is_some());
        for kind in bipeds {
            spawn_picture(cells, Shown::Body(kind));
        }
    });
}

fn spawn_blade_row(grid: &mut ChildSpawnerCommands, family: Family) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label(&format!("{family:?}"), PICTURE_ROW_LABEL));
        for cell in 0..RING_COLUMNS {
            let seed = (family as u128 + 1) * 1000 + cell;
            spawn_picture(cells, Shown::Blade(family, seed));
        }
    });
}

fn spawn_picture(row: &mut ChildSpawnerCommands, shown: Shown) {
    row.spawn((
        GemPicture(shown),
        Pressable,
        Node {
            width: px(PICTURE),
            height: px(PICTURE),
            flex_shrink: 0.0,
            ..default()
        },
        ImageNode::default(),
    ));
}

/// What Item Pictures' page dropdown, quality tabs and pictures do.
#[allow(clippy::too_many_arguments)]
pub(super) fn choose_pictures(
    mut chosen: MessageReader<TabChosen>,
    mut pages: MessageReader<DropdownChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    rows: Query<(), With<PictureQualities>>,
    dropdowns: Query<(), With<PicturePages>>,
    pictures: Query<&GemPicture>,
    mut form: ResMut<PicturesForm>,
    mut preferences: ResMut<Preferences>,
) {
    for &TabChosen(row, index) in chosen.read() {
        if rows.contains(row) {
            form.quality = Quality::ALL[index];
        }
    }
    for &DropdownChosen(dropdown, index) in pages.read() {
        let name = PicturePage::ALL[index].name();
        if dropdowns.contains(dropdown) && preferences.item_pictures.as_deref() != Some(name) {
            preferences.item_pictures = Some(name.to_owned());
        }
    }
    for press in pressed.read() {
        if let Ok(&GemPicture(shown)) = pictures.get(press.0) {
            form.chosen = shown;
        }
    }
}

/// Fills Item Pictures' grid with the page chosen, and shows its name in the dropdown.
/// Only that page's pictures are drawn.
pub(super) fn fill_pictures(
    mut commands: Commands,
    preferences: Res<Preferences>,
    mut grids: Query<(Entity, &mut PicturesGrid)>,
    mut dropdowns: Query<&mut Dropdown, With<PicturePages>>,
) {
    let page = PicturePage::chosen(&preferences);
    for (entity, mut grid) in &mut grids {
        if grid.0 != Some(page) {
            grid.0 = Some(page);
            commands
                .entity(entity)
                .despawn_children()
                .with_children(|grid| page.spawn(grid));
        }
    }
    let index = PicturePage::ALL
        .iter()
        .position(|&p| p == page)
        .unwrap_or(0);
    for mut dropdown in &mut dropdowns {
        if dropdown.selected != index {
            dropdown.selected = index;
        }
    }
}

/// Keeps Item Pictures' images to what is chosen.
pub(super) fn show_pictures(
    form: Res<PicturesForm>,
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut grid: Query<(Ref<GemPicture>, &mut ImageNode, &mut Node), Without<LargePicture>>,
    mut large: Query<(Ref<LargePicture>, &mut ImageNode, &mut Node), Without<GemPicture>>,
) {
    for (picture, mut image, mut node) in &mut grid {
        if (form.is_changed() || picture.is_added())
            && let Some(look) = picture.0.look(form.quality)
        {
            // A weapon or gauntlets need their full size for the black edge to be right, and
            // more room.
            let (box_side, pixels) = match picture.0 {
                Shown::Armour(_) => (
                    HANDS_PICTURE,
                    look.longest() as u32 * crate::widgets::PICTURE_PIXELS_PER_CELL,
                ),
                Shown::Blade(..) | Shown::Body(_) => (
                    BLADE_PICTURE,
                    look.longest() as u32 * crate::widgets::PICTURE_PIXELS_PER_CELL,
                ),
                _ => (PICTURE, PICTURE_PIXELS),
            };
            let size = look.picture_size(1000).as_vec2() / 1000.0 * box_side;
            node.width = px(size.x);
            node.height = px(size.y);
            node.margin = UiRect::horizontal(px((PICTURE - size.x).max(0.0) / 2.0));
            image.image = pictures.picture(look, pixels, &mut images);
        }
    }
    for (picture, mut image, mut node) in &mut large {
        if (form.is_changed() || picture.is_added())
            && let Some(look) = form.chosen.look(form.quality)
        {
            let size = look.picture_size(1000).as_vec2() / 1000.0 * LARGE_PICTURE;
            node.width = px(size.x);
            node.height = px(size.y);
            node.margin = UiRect::horizontal(px((LARGE_PICTURE - size.x) / 2.0));
            image.image = pictures.picture(look, LARGE_PICTURE_PIXELS, &mut images);
        }
    }
}

/// Asks the server for an item of this kind.
#[derive(Component, Clone, Copy)]
pub(super) struct GiveButton(ItemKind);

/// Asks the server to make the character this kind of creature.
#[derive(Component, Clone, Copy)]
pub(super) struct CreatureButton(CreatureKind);

/// The tabs choosing what size new items are made for.
#[derive(Component)]
pub(super) struct GiveSizes;

/// The tabs choosing whether new items are magical.
#[derive(Component)]
pub(super) struct GiveMagic;

/// The tabs choosing whether new items are blessed or cursed.
#[derive(Component)]
pub(super) struct GiveFavour;

/// The tabs choosing whether new items are part of a set.
#[derive(Component)]
pub(super) struct GiveSet;

/// The tabs choosing how good new items are.
#[derive(Component)]
pub(super) struct GiveQualities;

/// How Give Item makes things.
#[derive(Resource, Default)]
pub(super) struct GiveForm {
    /// `None` for the character's own.
    size: Option<Size>,
    magical: bool,
    favour: Option<Favour>,
    quality: Quality,
    set: bool,
}

/// Follows Give Item's tabs.
pub(super) fn choose_give_options(
    mut chosen: MessageReader<TabChosen>,
    sizes: Query<(), With<GiveSizes>>,
    magic: Query<(), With<GiveMagic>>,
    favour: Query<(), With<GiveFavour>>,
    set: Query<(), With<GiveSet>>,
    qualities: Query<(), With<GiveQualities>>,
    mut form: ResMut<GiveForm>,
) {
    for &TabChosen(row, index) in chosen.read() {
        if sizes.contains(row) {
            form.size = index.checked_sub(1).map(|index| Size::ALL[index]);
        }
        if magic.contains(row) {
            form.magical = index == 1;
        }
        if favour.contains(row) {
            form.favour = match index {
                1 => Some(Favour::Blessed),
                2 => Some(Favour::Cursed),
                _ => None,
            };
        }
        if set.contains(row) {
            form.set = index == 1;
        }
        if qualities.contains(row) {
            form.quality = Quality::ALL[index];
        }
    }
}

/// What Give Item's buttons do.
pub(super) fn press_give_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&GiveButton>,
    form: Res<GiveForm>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(&GiveButton(kind)) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevGiveItem {
                kind,
                size: form.size,
                magical: form.magical,
                quality: form.quality,
                set: form.set,
                favour: form.favour,
            }));
        }
    }
}

/// What Change Creature's buttons do.
pub(super) fn press_creature_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&CreatureButton>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(&CreatureButton(kind)) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevSetCreature(kind)));
        }
    }
}

/// One of each thing `widgets` makes, to see how they look together.
fn spawn_every_widget(content: &mut ChildSpawnerCommands) {
    let area = spawn_scroll_area(content, Val::Vh(EVERY_WIDGET_HEIGHT));
    content.commands().entity(area).with_children(|area| {
        area.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        })
        .with_children(spawn_widgets);
    });
}

fn spawn_widgets(content: &mut ChildSpawnerCommands) {
    for (state, label) in [
        (ButtonState::Normal, "Normal"),
        (ButtonState::Selected, "Selected"),
        (ButtonState::Active, "Active"),
        (ButtonState::Disabled, "Disabled"),
        (ButtonState::DisabledSelected, "Disabled, selected"),
    ] {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, state, label, FONT, node).insert(Pressable);
    }
    for (style, label) in [(ButtonStyle::Main, "Main"), (ButtonStyle::Quiet, "Quiet")] {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, label, FONT, node).insert((style, Pressable));
    }
    for (on, label) in [
        (Some(true), "Check on"),
        (Some(false), "Check off"),
        (None, "No check"),
    ] {
        let node = Node {
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_check_button(content, ButtonState::Normal, on, label, FONT, node).insert(Pressable);
    }
    spawn_tabs(content, &["Tab", "Other tab", "Third tab"], 0, FONT);
    spawn_toggle_row(content, &["Toggle", "Other", "Third"], 0, FONT);
    spawn_progress_bar(
        content,
        0.6,
        (GRID_COLOUR, ALLOWED),
        Node {
            height: px(CHECK_ROW),
            ..default()
        },
        (Text::new("Progress"), FONT.font(), TextColor(INK)),
    );
    spawn_close_button(content, default());
    spawn_slider(content, 0.5, None, 160.0);
    let kinds = || PoiKind::ALL.map(kind_name).to_vec();
    for searchable in [false, true] {
        let dropdown = Dropdown {
            options: kinds(),
            selected: 0,
            searchable,
        };
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_dropdown(content, dropdown, FONT, node);
    }
    let text_box = TextBox {
        text: "Text box".into(),
        takes_keys: true,
        ..default()
    };
    let node = Node {
        height: px(CHECK_ROW),
        ..default()
    };
    content.spawn(field_label("Label"));
    spawn_text_box(content, text_box, FONT, node.clone());
    let wrong = TextBox {
        text: "Something wrong".into(),
        takes_keys: true,
        error: true,
        ..default()
    };
    spawn_text_box(content, wrong, FONT, node);
    content.spawn(raised_box()).with_child((
        Text::new("Raised box"),
        FONT.font(),
        TextColor(crate::widgets::INK),
    ));
    content
        .spawn(Node::default())
        .with_children(|holder| spawn_tooltip(holder, SampleTooltip, "Tooltip"));
}

/// The sample tooltip's text.
#[derive(Component)]
pub(super) struct SampleTooltip;

/// Open the Generate POI panel for this kind (the submenu chose it).
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct GenerateKind(pub PoiKind);

/// What the Generate POI panel is set to, besides its seed box.
#[derive(Resource, Default)]
pub(super) struct GenerateForm {
    kind: Option<PoiKind>,
    ruin: u8,
    seeds_made: i64,
}

#[derive(Component, Clone, Copy)]
pub(super) struct RuinButton(u8);

#[derive(Component)]
pub(super) struct SeedBox;

#[derive(Component)]
pub(super) struct NewSeedButton;

#[derive(Component)]
pub(super) struct GenerateButton;

/// The most digits of a seed the new seed button makes: enough for many, and
/// short enough to write down and type in again.
const NEW_SEEDS: u64 = 1_000_000_000;
const SEED_WIDTH: f32 = 240.0;

/// A seed to try. `now` is any number that keeps changing, `made` how many so far.
fn new_seed(now: u64, made: i64) -> u64 {
    Seed::new(now).at(&[made]) % NEW_SEEDS
}

/// A round arrow, drawn here as there is no such letter in the font.
fn new_seed_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [64, 43, 26, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (dx, dy) = (x as f32 - middle, y as f32 - middle);
            let reach = dx.hypot(dy);
            // Clockwise from the top, leaving a gap for the arrow's head.
            let round = dx.atan2(-dy).rem_euclid(std::f32::consts::TAU);
            let ring = (8.0..12.0).contains(&reach) && round > 0.9;
            // The head, pointing round the ring at the top right.
            let head_along = dx + 2.0;
            let head =
                (0.0..8.0).contains(&head_along) && (dy + 10.0).abs() < (8.0 - head_along) * 0.9;
            if ring || head {
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

fn row() -> Node {
    Node {
        column_gap: px(4),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn words(text: impl Into<String>) -> impl Bundle {
    (Text::new(text), FONT.font(), TextColor(INK))
}

/// Opens the Generate POI panel for the kind chosen, in place of any open.
#[allow(clippy::too_many_arguments)]
pub(super) fn open_generate_panel(
    mut kinds: MessageReader<GenerateKind>,
    mut commands: Commands,
    mut open: ResMut<OpenPanels>,
    mut form: ResMut<GenerateForm>,
    mut images: ResMut<Assets<Image>>,
    nodes: Query<(), With<Node>>,
    time: Res<Time<Real>>,
) {
    let Some(&GenerateKind(kind)) = kinds.read().last() else {
        return;
    };
    let action = DebugAction::GeneratePoi;
    if let Some(panel) = open.panels.remove(&action)
        && nodes.contains(panel)
    {
        commands.entity(panel).despawn();
    }
    let at = FIRST_AT + NEXT_AT * (open.opened % 8) as f32;
    open.opened += 1;
    let (panel, content) = spawn_moveable_panel(&mut commands, action.name(), at);
    open.panels.insert(action, panel);
    form.kind = Some(kind);
    form.ruin = 0;
    form.seeds_made += 1;
    let seed = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
    let icon = images.add(new_seed_icon());
    commands.entity(content).with_children(|content| {
        content.spawn(words(kind_name(kind)));
        content.spawn(row()).with_children(|row| {
            row.spawn(words("Ruin"));
            for ruin in 0..=MAX_RUIN {
                let node = Node {
                    padding: BUTTON_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, ruin.to_string(), FONT, node)
                    .insert((RuinButton(ruin), Pressable));
            }
        });
        content.spawn(row()).with_children(|row| {
            row.spawn(words("Seed"));
            let text_box = TextBox {
                text: seed.to_string(),
                digits_only: true,
                takes_keys: true,
                ..default()
            };
            let node = Node {
                width: px(SEED_WIDTH),
                height: px(CHECK_ROW),
                ..default()
            };
            spawn_text_box(row, text_box, FONT, node).insert(SeedBox);
            let node = Node {
                width: px(CHECK_ROW),
                height: px(CHECK_ROW),
                ..default()
            };
            spawn_icon_button(row, ButtonState::Normal, icon, node)
                .insert((NewSeedButton, Pressable));
        });
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, "Generate", FONT, node)
            .insert(ButtonStyle::Main)
            .insert((GenerateButton, Pressable));
    });
}

/// What the Generate POI panel's buttons do.
#[allow(clippy::too_many_arguments)]
pub(super) fn press_generate_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    ruins: Query<&RuinButton>,
    new_seed_button: Query<(), With<NewSeedButton>>,
    generate: Query<(), With<GenerateButton>>,
    mut seed_box: Query<&mut TextBox, With<SeedBox>>,
    mut form: ResMut<GenerateForm>,
    time: Res<Time<Real>>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(ruin) = ruins.get(press.0) {
            form.ruin = ruin.0;
        }
        let Ok(mut seed) = seed_box.single_mut() else {
            continue;
        };
        if new_seed_button.contains(press.0) {
            form.seeds_made += 1;
            let new = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
            seed.text = new.to_string();
        }
        if generate.contains(press.0)
            && let (Some(kind), Ok(seed)) = (form.kind, seed.text.parse())
        {
            to_server.write(ToServer(ClientMessage::DevGenerateSite {
                kind,
                seed,
                ruin: form.ruin,
            }));
        }
    }
}

/// Shows the ruin chosen, and greys what can't be used: ruin for kinds that have
/// none, and Generate without a seed.
pub(super) fn update_generate_buttons(
    form: Res<GenerateForm>,
    mut ruins: Query<(&RuinButton, &mut ButtonState), Without<GenerateButton>>,
    mut generate: Query<&mut ButtonState, With<GenerateButton>>,
    seed_box: Query<&TextBox, With<SeedBox>>,
) {
    let can_ruin = form.kind.is_some_and(Generator::can_be_ruined);
    for (button, mut state) in &mut ruins {
        let chosen = button.0 == form.ruin;
        state.set_if_neq(match (can_ruin, chosen) {
            (true, true) => ButtonState::Selected,
            (true, false) => ButtonState::Normal,
            (false, true) => ButtonState::DisabledSelected,
            (false, false) => ButtonState::Disabled,
        });
    }
    let has_seed = seed_box
        .single()
        .is_ok_and(|seed| seed.text.parse::<u64>().is_ok());
    for mut state in &mut generate {
        state.set_if_neq(if has_seed {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}

/// Whether a panel's buttons can do anything here.
fn can_step(action: DebugAction, view: GameView, dungeon: &CurrentDungeon) -> bool {
    action == DebugAction::WorldTime
        || view == GameView::Dungeon
            && match action {
                DebugAction::StairsStyle => !dungeon.stairs.is_empty(),
                DebugAction::FieldsState => !dungeon.fields.is_empty(),
                DebugAction::HerbsState => dungeon.level_has(Cell::HerbBed),
                _ => true,
            }
}

/// Forgets panels closed by their own close buttons.
pub(super) fn forget_closed(mut open: ResMut<OpenPanels>, nodes: Query<(), With<Node>>) {
    let closed = open.panels.values().any(|&panel| !nodes.contains(panel));
    if closed {
        open.panels.retain(|_, panel| nodes.contains(*panel));
    }
}

/// The style before this one, going round.
fn previous_style(style: StairsStyle) -> StairsStyle {
    let all = StairsStyle::ALL;
    let at = all.iter().position(|&other| other == style).unwrap_or(0);
    all[(at + all.len() - 1) % all.len()]
}

/// Moves what the pressed button's panel is for on or back a step: only what is
/// drawn, and the next level goes back to its own.
pub(super) fn press_panel_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&StepButton>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut shift: ResMut<HerbGrowthShift>,
    clock: Res<ServerClock>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(&StepButton { action, forward }) = buttons.get(press.0) else {
            continue;
        };
        match action {
            DebugAction::StairsStyle => {
                for stairs in &mut dungeon.stairs {
                    stairs.style = if forward {
                        stairs.style.next()
                    } else {
                        previous_style(stairs.style)
                    };
                }
            }
            DebugAction::FieldsState => {
                for field in &mut dungeon.fields {
                    // As crops, which alone go through every stage.
                    field.livestock = false;
                    let by = if forward { 1 } else { CROP_STAGES as u32 - 1 };
                    field.variety = field.variety.wrapping_add(by);
                }
            }
            DebugAction::HerbsState => {
                // Back is nearly all the way round, as the shift can't go below 0.
                let by = if forward { 1 } else { HERB_STAGES - 1 };
                shift.0 = (shift.0 + by) % HERB_STAGES;
            }
            DebugAction::WorldTime => {
                let by = if forward {
                    MINUTES_PER_HOUR
                } else {
                    -MINUTES_PER_HOUR
                };
                to_server.write(ToServer(ClientMessage::DevSetTime {
                    minutes: clock.minutes + by,
                }));
            }
            _ => {}
        }
    }
}

/// Greys the buttons where they can do nothing; the panel stays open.
pub(super) fn update_panel_buttons(
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut buttons: Query<(&StepButton, &mut ButtonState)>,
) {
    for (button, mut state) in &mut buttons {
        state.set_if_neq(if can_step(button.action, *view.get(), &dungeon) {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_seeds_are_short_and_differ() {
        let seeds: Vec<u64> = (0..50).map(|made| new_seed(12345, made)).collect();
        assert!(seeds.iter().all(|&seed| seed < NEW_SEEDS));
        assert!(seeds.iter().any(|&seed| seed != seeds[0]));
    }

    #[test]
    fn previous_undoes_next_for_every_style() {
        for style in StairsStyle::ALL {
            assert_eq!(previous_style(style.next()), style);
        }
    }

    #[test]
    fn panels_need_a_site_with_what_they_change() {
        let dungeon = CurrentDungeon::default();
        let in_site = |action| can_step(action, GameView::Dungeon, &dungeon);
        assert!(!in_site(DebugAction::StairsStyle));
        assert!(!in_site(DebugAction::FieldsState));
        assert!(!in_site(DebugAction::HerbsState));
        assert!(!can_step(
            DebugAction::StairsStyle,
            GameView::WorldMap,
            &dungeon
        ));
    }
}
