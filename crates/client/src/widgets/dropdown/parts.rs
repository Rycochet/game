//! A dropdown and its open list's parts, and spawning one.

use bevy::prelude::*;

use super::super::text_box::RaisesKeyboard;
use super::super::{
    ButtonState, Pressable, TextSize, spawn_button, spawn_small_icon_button, spawn_strokes,
};
use super::{ARROW_RIGHT, ARROW_STROKES, ARROW_WIDE};

/// A choice among `options`. Set `selected` to change it.
#[derive(Component, Clone, Debug, Default)]
pub struct Dropdown {
    pub options: Vec<String>,
    pub selected: usize,
    pub searchable: bool,
    /// A picture for each option, shown beside it in the list, or none.
    pub icons: Vec<Handle<Image>>,
    /// Options shown but not to be chosen, by index; missing ones are not greyed.
    pub greyed: Vec<bool>,
}

impl Dropdown {
    /// Whether this option is shown greyed, and can't be chosen.
    pub(in crate::widgets) fn is_greyed(&self, at: usize) -> bool {
        self.greyed.get(at).copied().unwrap_or(false)
    }
}

/// The player chose an option of this dropdown.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropdownChosen(pub Entity, pub usize);

/// Clicks outside the list, which close it.
#[derive(Component)]
pub(in crate::widgets) struct ListBacking;

/// Holds the rows on show.
#[derive(Component)]
pub(in crate::widgets) struct ListRows;

/// An option in an open list, by its index in `options`.
#[derive(Component, Clone, Copy)]
pub(in crate::widgets) struct ListOption(pub(super) usize);

/// The list that is open.
#[derive(Resource, Default)]
pub(in crate::widgets) struct OpenList(pub(super) Option<List>);

pub(in crate::widgets) struct List {
    pub(super) dropdown: Entity,
    pub(super) backing: Entity,
    pub(super) search: Option<Entity>,
    pub(super) filter: String,
    /// Which of those on show the arrow keys are on.
    pub(super) highlight: Option<usize>,
    /// The first of those on show that is drawn.
    pub(super) first: usize,
    pub(super) dragged: f32,
    pub(super) drawn: Option<(Vec<usize>, usize, Option<usize>)>,
}

/// A dropdown, with `node` for its size and where it goes.
pub fn spawn_dropdown<'a>(
    parent: &'a mut ChildSpawnerCommands,
    dropdown: Dropdown,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let label = label_of(&dropdown);
    let searchable = dropdown.searchable;
    let mut button = spawn_button(parent, ButtonState::Normal, label, size, node);
    button.insert((dropdown, Pressable));
    button.with_children(|inside| {
        inside
            .spawn(Node {
                position_type: PositionType::Absolute,
                right: px(ARROW_RIGHT + ARROW_WIDE),
                top: percent(50),
                margin: UiRect::top(px(-ARROW_WIDE / 2.0)),
                ..default()
            })
            .with_children(|arrow| spawn_strokes(arrow, &ARROW_STROKES));
    });
    if searchable {
        button.insert(RaisesKeyboard);
    }
    button
}

/// A small icon button showing the chosen option's picture, which opens the list.
/// Give it a `ButtonTooltip` to say what it chooses.
pub fn spawn_icon_dropdown<'a>(
    parent: &'a mut ChildSpawnerCommands,
    dropdown: Dropdown,
    node: Node,
) -> EntityCommands<'a> {
    let icon = dropdown.icons.get(dropdown.selected).cloned();
    let mut button =
        spawn_small_icon_button(parent, ButtonState::Normal, icon.unwrap_or_default(), node);
    button.insert((dropdown, Pressable));
    button
}

pub(super) fn label_of(dropdown: &Dropdown) -> String {
    let chosen = dropdown.options.get(dropdown.selected);
    chosen.cloned().unwrap_or_default()
}
