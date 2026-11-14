//! Drawing the tree: a round node for each known skill, coloured by the attributes
//! it leans on and showing its icon, joined to its parent (or to the plain circle
//! in the middle, for the character) by a line. Drawn again only when a skill is
//! learned (or forgotten), not on every bit of practice: `values` keeps the rings
//! up to date in place, and `place` moves and sizes them as the page is moved and
//! zoomed.

use bevy::prelude::*;
use shared::skill::{SkillId, Skills, VALUE_SCALE};

use super::icons::icon_path;
use super::layout::{Placed, SIZES, layout, size_of};
use super::spawn::{FORM, Origin};
use super::view::SkillsView;
use crate::character::CharacterSheet;
use crate::widgets::{
    INK, Shade, SkillRing, UiId, attribute_colour, skill_ring, spawn_outlined_icon,
};

const LINE_THICKNESS: f32 = 2.0;
const LINE_COLOUR: Color = Color::srgba(0.25, 0.17, 0.1, 0.55);
/// The character's circle in the middle.
const MIDDLE_FILL: Color = Color::srgb(0.55, 0.42, 0.26);
/// How much of a node its icon fills, as a share of its width.
const ICON_SHARE: f32 = 0.68;

/// A known skill's node, or the character's in the middle (`id` none), placed and
/// sized at zoom 1.
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

/// A skill's ring, kept up to date by `values::show_values`.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct RingOf(pub SkillId);

/// Which skills the tree was last drawn with.
#[derive(Default)]
pub(super) struct Drawn(Option<Vec<SkillId>>);

impl Drawn {
    /// Remembers which skills `skills` knows, saying whether that differs from
    /// what was drawn: practice alone doesn't need the tree drawn again (and doing
    /// so on every blow made the rings flicker, and could crash).
    pub(super) fn changed_to(&mut self, skills: &Skills) -> bool {
        let known: Vec<_> = skills.known().map(|(id, _)| id).collect();
        if self.0.as_ref() == Some(&known) {
            return false;
        }
        self.0 = Some(known);
        true
    }
}

/// Draws the tree again when the page is on show and which skills are known has
/// changed, or the page is new (the Character panel opened again).
#[allow(clippy::too_many_arguments)]
pub(super) fn build_tree(
    view: Res<SkillsView>,
    sheet: Res<CharacterSheet>,
    origin: Option<Single<(Entity, Ref<Origin>)>>,
    assets: Res<AssetServer>,
    mut drawn: Local<Drawn>,
    mut commands: Commands,
) {
    let Some(origin) = origin else {
        return;
    };
    let (origin, mark) = *origin;
    if mark.is_added() {
        *drawn = Drawn::default();
    }
    let skills = &sheet.0.skills;
    if !view.shown || !drawn.changed_to(skills) {
        return;
    }
    let placed = layout(skills);
    let mut origin = commands.entity(origin);
    origin.despawn_children();
    origin.with_children(|origin| {
        for one in &placed {
            spawn_line(origin, one);
        }
        spawn_middle(origin);
        for one in &placed {
            let icon = icon_path(one.id).map(|path| assets.load(path));
            spawn_skill(origin, one, ring_value(skills, one.id), icon);
        }
    });
}

fn spawn_line(origin: &mut ChildSpawnerCommands, one: &Placed) {
    origin.spawn((
        SkillLine {
            from: one.from,
            to: one.at,
        },
        Node {
            position_type: PositionType::Absolute,
            height: px(LINE_THICKNESS),
            ..default()
        },
        UiTransform::default(),
        BackgroundColor(LINE_COLOUR),
    ));
}

/// A round node: its edge in ink, an outline (shown on hovering or choosing it)
/// kept apart from the edge, which the skill's ring will use.
fn round(size: f32, fill: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: px(size),
            height: px(size),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(fill),
        BorderColor::all(INK),
        Outline::new(px(3), px(2), Color::NONE),
    )
}

/// The character in the middle: a plain circle (owner).
fn spawn_middle(origin: &mut ChildSpawnerCommands) {
    let size = SIZES[0];
    origin.spawn((
        SkillNode {
            id: None,
            at: Vec2::ZERO,
            size,
        },
        UiId::new(FORM, "you"),
        round(size, MIDDLE_FILL),
    ));
}

/// A known skill: coloured by what it leans on, its icon in white with a black
/// outline, as abilities' are, and a ring round its edge for how good it is (its own
/// value; specialising isn't built yet).
fn spawn_skill(
    origin: &mut ChildSpawnerCommands,
    one: &Placed,
    value: f32,
    icon: Option<Handle<Image>>,
) {
    let size = size_of(one.id);
    let fill = attribute_colour(one.id.leans(), Shade::Skill);
    let mut node = origin.spawn((
        SkillNode {
            id: Some(one.id),
            at: one.at,
            size,
        },
        UiId::new(FORM, "node").at(usize::from(one.id.number())),
        round(size, fill),
    ));
    if let Some(icon) = icon {
        let inset = percent((1.0 - ICON_SHARE) / 2.0 * 100.0);
        node.with_children(|node| {
            node.spawn((Node {
                position_type: PositionType::Absolute,
                left: inset,
                top: inset,
                width: percent(ICON_SHARE * 100.0),
                height: percent(ICON_SHARE * 100.0),
                ..default()
            },))
                .with_children(|holder| spawn_outlined_icon(holder, icon, Color::WHITE));
        });
    }
    node.with_child((skill_ring(SkillRing::of(value, false)), RingOf(one.id)));
}

/// A skill's own value as its ring shows it.
pub(super) fn ring_value(skills: &Skills, id: SkillId) -> f32 {
    skills.value(id) as f32 / VALUE_SCALE as f32
}

/// A value in hundredths as words, to a tenth: 2581 is "25.8".
pub(super) fn tenths(hundredths: u32) -> String {
    let tenths = hundredths * 10 / VALUE_SCALE;
    format!("{}.{}", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests;
