//! The Learn Skills panel (owner's design, 2026-11-09): every skill of the tree
//! as a checkbox (learn or forget it) with its value in a box beside it, for
//! the character or another player targeted, and buttons for six skills at
//! random, forgetting them all, or setting them all to one value. It takes the
//! place of Learn Some Skills and Forget Skills.

use bevy::prelude::*;
use shared::id::Id;
use shared::skill::{SkillId, Skills};

use super::{CHECK_ROW, FONT, row, words};
use crate::widgets::{
    BUTTON_PADDING, ButtonState, Pressable, TextBox, TextSize, UiId, spawn_button,
    spawn_check_button, spawn_scroll_area, spawn_text_box,
};

/// The panel's form, for the debug API.
pub(super) const FORM: &str = "debug.learn_skills";
/// How tall the list of skills may be before it scrolls, as a share of the
/// window's height.
const LIST_HEIGHT: f32 = 60.0;
/// How far each level of the tree is set in, in pixels.
const INDENT: f32 = 18.0;
const BOX_WIDTH: f32 = 56.0;
const SKILL_WIDTH: f32 = 240.0;

/// Whose skills the panel shows and changes, and what it was last told of them.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct LearnSkillsForm {
    /// The targeted player's, rather than the character's.
    pub(super) for_target: bool,
    /// Whose skills were last asked for (`None`: the character's), while open.
    pub(super) asked: Option<Option<Id>>,
    /// What the server said they are.
    pub(super) skills: Option<Skills>,
}

/// The panel's buttons, other than the skills'.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum LearnButton {
    You,
    Target,
    Random,
    Clear,
    SetAll,
}

/// A skill's checkbox: learns or forgets it.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct SkillCheck(pub(super) SkillId);

/// A skill's value, as a whole percentage, set when the box is left.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct SkillBox(pub(super) SkillId);

/// The value Set All gives every skill.
#[derive(Component)]
pub(in crate::debug_menu) struct SetAllBox;

pub(super) fn spawn_learn_skills(content: &mut ChildSpawnerCommands) {
    let button = |content: &mut ChildSpawnerCommands, label: &str, which, name| {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, label, FONT, node).insert((
            which,
            Pressable,
            UiId::new(FORM, name),
        ));
    };
    content.spawn(row()).with_children(|row| {
        row.spawn(words("For"));
        button(row, "You", LearnButton::You, "you");
        button(row, "Target", LearnButton::Target, "target");
    });
    content.spawn(row()).with_children(|row| {
        button(row, "Random", LearnButton::Random, "random");
        button(row, "Clear Skills", LearnButton::Clear, "clear");
        spawn_percent_box(row).insert((SetAllBox, UiId::new(FORM, "all")));
        row.spawn(words("%"));
        button(row, "Set All", LearnButton::SetAll, "set_all");
    });
    let area = spawn_scroll_area(content, Val::Vh(LIST_HEIGHT));
    content
        .commands()
        .entity(area)
        .insert(UiId::new(FORM, "scroll"));
    content.commands().entity(area).with_children(|area| {
        area.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            ..default()
        })
        .with_children(|list| {
            for (at, skill) in SkillId::all().enumerate() {
                spawn_skill_row(list, at, skill);
            }
        });
    });
}

/// A box taking a whole percentage.
fn spawn_percent_box<'a>(row: &'a mut ChildSpawnerCommands) -> EntityCommands<'a> {
    let text_box = TextBox {
        digits_only: true,
        takes_keys: true,
        ..default()
    };
    let node = Node {
        width: px(BOX_WIDTH),
        height: px(CHECK_ROW),
        ..default()
    };
    spawn_text_box(row, text_box, FONT, node)
}

/// One skill: its value's box, then its checkbox, set in by its depth.
fn spawn_skill_row(list: &mut ChildSpawnerCommands, at: usize, skill: SkillId) {
    let indent = INDENT * f32::from(skill.depth().saturating_sub(1));
    list.spawn(Node {
        padding: UiRect::left(px(indent)),
        ..row()
    })
    .with_children(|row| {
        spawn_percent_box(row).insert((SkillBox(skill), UiId::new(FORM, "value").at(at)));
        let node = Node {
            width: px(SKILL_WIDTH),
            height: px(CHECK_ROW),
            ..default()
        };
        let name = skill.name();
        spawn_check_button(
            row,
            ButtonState::Disabled,
            Some(false),
            name,
            TextSize::Small,
            node,
        )
        .insert((
            SkillCheck(skill),
            Pressable,
            UiId::new(FORM, "skill").at(at),
        ));
    });
}
