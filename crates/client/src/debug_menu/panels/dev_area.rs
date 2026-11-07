//! The Dev Area panel (owner's design): size, layout, who is in it and how many,
//! and a seed of any words, then Generate, which goes straight in.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::site::{DEV_AREA_SIZES, DevArea, DevLayout, DevMobs, MOST_DEV_MOBS};

use super::{CHECK_ROW, FONT, SEED_WIDTH, row, words};
use crate::widgets::{
    BUTTON_PADDING, ButtonState, ButtonStyle, Dropdown, Pressable, TextBox, UiId, spawn_button,
    spawn_dropdown, spawn_new_seed_button, spawn_text_box,
};

const COUNT_WIDTH: f32 = 64.0;
const ARROW_ROOM: f32 = 24.0;

/// What the Dev Area panel is set to, besides its seed box. Kept while the
/// game runs, so the panel opens as it was left.
#[derive(Resource, Clone, Debug)]
pub(in crate::debug_menu) struct DevAreaForm {
    pub(super) size: u16,
    pub(super) layout: DevLayout,
    /// `None` for Default, `Some(None)` for None, else the kind.
    pub(super) mobs: Option<Option<CreatureKind>>,
    pub(super) count: u8,
    /// The seed box's words, kept for the next time the panel opens.
    pub(super) seed: String,
    pub(super) seeds_made: i64,
}

impl Default for DevAreaForm {
    fn default() -> Self {
        Self {
            size: DEV_AREA_SIZES[1],
            layout: DevLayout::Empty,
            mobs: None,
            count: 3,
            seed: String::new(),
            seeds_made: 0,
        }
    }
}

impl DevAreaForm {
    /// What to ask the server to make, as the panel says.
    pub(super) fn area(&self) -> DevArea {
        let mobs = match self.mobs {
            None => DevMobs::Default,
            Some(None) => DevMobs::None,
            Some(Some(kind)) => DevMobs::Kind {
                kind,
                count: self.count,
            },
        };
        DevArea {
            size: self.size,
            layout: self.layout,
            mobs,
            seed: DevArea::seed_from_text(&self.seed),
        }
        .checked()
    }

    /// Whether a count goes with the mobs chosen.
    pub(super) fn has_count(&self) -> bool {
        matches!(self.mobs, Some(Some(_)))
    }

    /// The mobs dropdown's choice: Default, None, then every kind.
    pub(super) fn mobs_index(&self) -> usize {
        match self.mobs {
            None => 0,
            Some(None) => 1,
            Some(Some(kind)) => {
                2 + CreatureKind::ALL
                    .iter()
                    .position(|&k| k == kind)
                    .unwrap_or(0)
            }
        }
    }

    pub(super) fn choose_mobs(&mut self, index: usize) {
        self.mobs = match index {
            0 => None,
            1 => Some(None),
            _ => Some(CreatureKind::ALL.get(index - 2).copied()),
        };
    }

    /// One more or fewer, kept to what can be made.
    pub(super) fn step_count(&mut self, step: i8) {
        self.count = self
            .count
            .saturating_add_signed(step)
            .clamp(1, MOST_DEV_MOBS);
    }
}

/// Which of the panel's dropdowns it is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum DevAreaChoice {
    Size,
    Layout,
    Mobs,
}

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaCount;

#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct DevAreaCountStep(pub(super) i8);

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaSeed;

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaNewSeed;

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaGenerate;

/// The panel, as `DevAreaForm::default` has it; `start_dev_area_panel` then
/// sets it to the form kept, with a seed if there is none.
pub(super) fn spawn_dev_area_panel(content: &mut ChildSpawnerCommands) {
    let form = DevAreaForm::default();
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Size"));
        let sizes = DEV_AREA_SIZES.map(|size| format!("{size} × {size}"));
        let size = DEV_AREA_SIZES.iter().position(|&s| s == form.size);
        dropdown(line, DevAreaChoice::Size, sizes, size.unwrap_or(0), false);
        line.spawn(words("Generator"));
        let layouts = DevLayout::ALL.map(DevLayout::name);
        let layout = DevLayout::ALL.iter().position(|&l| l == form.layout);
        dropdown(
            line,
            DevAreaChoice::Layout,
            layouts,
            layout.unwrap_or(0),
            true,
        );
    });
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Mobs"));
        let kinds = CreatureKind::ALL
            .iter()
            .map(|kind| kind.def().name.to_owned());
        let mobs = ["Default".to_owned(), "None".to_owned()]
            .into_iter()
            .chain(kinds);
        dropdown(line, DevAreaChoice::Mobs, mobs, form.mobs_index(), true);
        button(line, "-").insert((DevAreaCountStep(-1), id("less")));
        let count = TextBox {
            text: form.count.to_string(),
            digits_only: true,
            takes_keys: true,
            ..default()
        };
        let node = Node {
            width: px(COUNT_WIDTH),
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_text_box(line, count, FONT, node).insert((DevAreaCount, id("count")));
        button(line, "+").insert((DevAreaCountStep(1), id("more")));
    });
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Seed"));
        let seed = TextBox {
            text: form.seed.clone(),
            takes_keys: true,
            ..default()
        };
        let node = Node {
            width: px(SEED_WIDTH),
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_text_box(line, seed, FONT, node).insert((DevAreaSeed, id("seed")));
        spawn_new_seed_button(line, CHECK_ROW).insert((DevAreaNewSeed, id("new_seed")));
    });
    let bottom = Node {
        justify_content: JustifyContent::End,
        ..row()
    };
    content.spawn(bottom).with_children(|line| {
        button(line, "Generate").insert((ButtonStyle::Main, DevAreaGenerate, id("generate")));
    });
}

/// A widget of the Dev Area panel.
fn id(name: &'static str) -> UiId {
    UiId::new("debug.dev_area", name)
}

fn button<'a>(line: &'a mut ChildSpawnerCommands, label: &str) -> EntityCommands<'a> {
    let node = Node {
        padding: BUTTON_PADDING,
        ..default()
    };
    let mut button = spawn_button(line, ButtonState::Normal, label, FONT, node);
    button.insert(Pressable);
    button
}

fn dropdown<S: Into<String>>(
    line: &mut ChildSpawnerCommands,
    choice: DevAreaChoice,
    options: impl IntoIterator<Item = S>,
    selected: usize,
    searchable: bool,
) {
    let name = match choice {
        DevAreaChoice::Size => "size",
        DevAreaChoice::Layout => "generator",
        DevAreaChoice::Mobs => "mobs",
    };
    let dropdown = Dropdown {
        options: options.into_iter().map(Into::into).collect(),
        selected,
        searchable,
        ..default()
    };
    let node = Node {
        padding: UiRect {
            right: px(ARROW_ROOM),
            ..BUTTON_PADDING
        },
        ..default()
    };
    spawn_dropdown(line, dropdown, FONT, node).insert((choice, id(name)));
}
