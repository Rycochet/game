//! Drawing the tree: a round node for each known skill, with its name under it,
//! joined to its parent (or to the character in the middle) by a line. Drawn
//! again whenever what is known changes; `place` moves them as the page is moved
//! and zoomed.

use bevy::prelude::*;
use shared::skill::{SkillId, Skills, VALUE_SCALE};

use super::icons::icon_path;
use super::layout::{Placed, layout};
use super::name::OwnName;
use super::spawn::{Canvas, FORM, Origin};
use crate::character::CharacterSheet;
use crate::common::SkillsPageOpen;
use crate::widgets::{HoverTip, INK, TextSize, UiId, leaves_presses};

/// How wide a node is at each level: the character, then the top levels and down.
const SIZES: [f32; 5] = [64.0, 52.0, 40.0, 32.0, 26.0];
const LINE_THICKNESS: f32 = 2.0;
/// The room under a node for its name.
const LABEL_WIDTH: f32 = 140.0;
const LINE_COLOUR: Color = Color::srgba(0.25, 0.17, 0.1, 0.55);
/// Each level's fill, lighter further out.
const FILLS: [Color; 5] = [
    Color::srgb(0.55, 0.42, 0.26),
    Color::srgb(0.75, 0.62, 0.42),
    Color::srgb(0.82, 0.71, 0.52),
    Color::srgb(0.88, 0.79, 0.62),
    Color::srgb(0.92, 0.85, 0.7),
];
pub(super) const CHOSEN_EDGE: Color = Color::srgb(0.85, 0.55, 0.1);

/// A known skill's node, or the character's in the middle with their name (`id`
/// none), placed at zoom 1.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct SkillNode {
    pub id: Option<SkillId>,
    pub at: Vec2,
    pub size: f32,
}

/// A line from a node to its parent, both at zoom 1.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct SkillLine {
    pub from: Vec2,
    pub to: Vec2,
}

/// A node's name, hidden when zoomed out past `hidden_below`.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct NodeLabel {
    pub hidden_below: f32,
}

/// What the tree was last drawn from: what is known, and the name in the middle.
#[derive(Default)]
pub(super) struct Drawn(Option<(Skills, String)>);

/// Draws the tree again when the page is open and what is known has changed.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_tree(
    open: Res<SkillsPageOpen>,
    sheet: Res<CharacterSheet>,
    name: Res<OwnName>,
    origin: Single<Entity, With<Origin>>,
    canvas: Single<Entity, With<Canvas>>,
    assets: Res<AssetServer>,
    mut drawn: Local<Drawn>,
    mut commands: Commands,
) {
    let skills = &sheet.0.skills;
    let now = (skills.clone(), name.0.clone());
    if !open.0 || drawn.0.as_ref() == Some(&now) {
        return;
    }
    drawn.0 = Some(now);
    let placed = layout(skills);
    let canvas = *canvas;
    let mut origin = commands.entity(*origin);
    origin.despawn_children();
    origin.with_children(|origin| {
        for one in &placed {
            spawn_line(origin, one, canvas);
        }
        spawn_node(
            origin,
            None,
            Vec2::ZERO,
            &name.0,
            String::new(),
            None,
            canvas,
        );
        for one in &placed {
            let name = one.id.name();
            spawn_node(
                origin,
                Some(one.id),
                one.at,
                name,
                tip(skills, one.id),
                icon_path(one.id).map(|path| assets.load(path)),
                canvas,
            );
        }
    });
}

fn spawn_line(origin: &mut ChildSpawnerCommands, one: &Placed, canvas: Entity) {
    origin.spawn((
        SkillLine {
            from: one.from,
            to: one.at,
        },
        leaves_presses(canvas),
        Node {
            position_type: PositionType::Absolute,
            height: px(LINE_THICKNESS),
            ..default()
        },
        UiTransform::default(),
        BackgroundColor(LINE_COLOUR),
    ));
}

fn spawn_node(
    origin: &mut ChildSpawnerCommands,
    id: Option<SkillId>,
    at: Vec2,
    name: &str,
    tip: String,
    icon: Option<Handle<Image>>,
    canvas: Entity,
) {
    let depth = id.map_or(0, |id| usize::from(id.depth()));
    let size = SIZES[depth.min(SIZES.len() - 1)];
    let ui_id = match id {
        Some(id) => UiId::new(FORM, "node").at(usize::from(id.number())),
        None => UiId::new(FORM, "you"),
    };
    let mut node = origin.spawn((
        SkillNode { id, at, size },
        ui_id,
        HoverTip(tip),
        leaves_presses(canvas),
        Node {
            position_type: PositionType::Absolute,
            width: px(size),
            height: px(size),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(FILLS[depth.min(FILLS.len() - 1)]),
        BorderColor::all(INK),
    ));
    if let Some(icon) = icon {
        // The icons are white, so tinting them draws them in ink.
        node.with_child((
            ImageNode::new(icon).with_color(INK),
            leaves_presses(canvas),
            Node {
                position_type: PositionType::Absolute,
                left: percent(16),
                top: percent(16),
                width: percent(68),
                height: percent(68),
                ..default()
            },
        ));
    }
    node.with_child((
        NodeLabel {
            // Deeper names go first, so the page doesn't fill with words.
            hidden_below: [0.0, 0.0, 0.3, 0.4, 0.5][depth.min(4)],
        },
        leaves_presses(canvas),
        Node {
            position_type: PositionType::Absolute,
            top: px(size + 2.0),
            left: px(size / 2.0 - LABEL_WIDTH / 2.0),
            width: px(LABEL_WIDTH),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Text::new(name),
            if depth <= 1 {
                TextSize::Medium.font()
            } else {
                TextSize::Small.font()
            },
            TextLayout::justify(Justify::Center),
            TextColor(INK),
        )],
    ));
}

/// What hovering over a skill says: its name and what a check with it counts.
pub(super) fn tip(skills: &Skills, id: SkillId) -> String {
    format!("{} {}", id.name(), tenths(skills.total(id)))
}

/// A value in hundredths as words, to a tenth: 2581 is "25.8".
pub(super) fn tenths(hundredths: u32) -> String {
    let tenths = hundredths * 10 / VALUE_SCALE;
    format!("{}.{}", tenths / 10, tenths % 10)
}
