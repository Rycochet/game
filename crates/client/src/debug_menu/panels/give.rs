//! Give Item's panel, laid out to read like what it makes ("Give 3 Large Size
//! Identical / Quality Rare Set Blessed Magical Longsword").

use bevy::prelude::*;
use shared::creature::Size;
use shared::item::Quality;

use super::{
    CHECK_ROW, FIELDS, FONT, GiveButton, GiveChoice, GiveCount, GiveCountStep, GiveNoSpace,
    GiveOption, GiveOptionsLine, GivePreview, GiveRandomSeed, GiveSeed, GiveToWords, SEED_WIDTH,
    SetChoice, any, give_kinds, row, ui_name, words,
};
use crate::widgets::{
    BUTTON_PADDING, ButtonState, ButtonStyle, Dropdown, INK, Pressable, TextBox, UiId,
    WARNING_TEXT, spawn_button, spawn_dropdown, spawn_new_seed_button, spawn_text_box,
};

const COUNT_WIDTH: f32 = 64.0;
const ARROW_ROOM: f32 = 24.0;
const BLANK_WIDTH: f32 = 56.0;

/// Give Item's panel, as `GiveForm::default` has it; `start_give_panel` fills in a seed.
pub(super) fn spawn_give_panel(content: &mut ChildSpawnerCommands) {
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Give"));
        spawn_count(line);
        let sizes = std::iter::once("<target>").chain(Size::ALL.iter().map(|size| size.name()));
        dropdown(line, (GiveChoice::Size, id("size")), sizes, 0, false);
        line.spawn(words("Size"));
        dropdown(
            line,
            (GiveChoice::Identical, id("identical")),
            ["Different", "Identical"],
            1,
            false,
        )
        .insert(ButtonState::Disabled);
    });
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Quality"));
        let common = Quality::ALL.iter().position(|&q| q == Quality::default());
        let qualities = Quality::ALL.map(|quality| format!("{quality:?}"));
        dropdown(
            line,
            (GiveChoice::Quality, id("quality")),
            qualities,
            common.unwrap_or(0),
            false,
        );
        dropdown(
            line,
            (GiveChoice::Set, id("set")),
            SetChoice::ALL.map(SetChoice::name),
            0,
            false,
        );
        let favours = ["Unaligned", "Blessed", "Cursed"];
        dropdown(line, (GiveChoice::Favour, id("favour")), favours, 0, false);
        dropdown(
            line,
            (GiveChoice::Magic, id("magic")),
            ["Mundane", "Magical"],
            0,
            false,
        );
        let kinds = give_kinds().into_iter().map(|kind| kind.def().name);
        dropdown(line, (GiveChoice::Kind, id("kind")), kinds, 0, true);
    });
    // Shown by `fill_give_options` for kinds whose form can be asked for.
    let hidden = Node {
        display: Display::None,
        ..row()
    };
    content
        .spawn((hidden, GiveOptionsLine))
        .with_children(|line| {
            line.spawn(words("Form"));
            for field in FIELDS {
                let option = GiveOption {
                    field,
                    values: Vec::new(),
                };
                dropdown(line, (option, id(ui_name(field))), [any(field)], 0, false);
            }
        });
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Seed"));
        let node = Node {
            width: px(SEED_WIDTH),
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_text_box(line, digits(), FONT, node).insert((GiveSeed, id("seed")));
        spawn_new_seed_button(line, CHECK_ROW).insert((GiveRandomSeed, id("new_seed")));
    });
    content.spawn((
        GivePreview::default(),
        Node {
            margin: UiRect::vertical(px(4)),
            ..default()
        },
    ));
    let bottom = Node {
        justify_content: JustifyContent::End,
        ..row()
    };
    content.spawn(bottom).with_children(|line| {
        let no_space = (Text::new("No space"), FONT.font(), TextColor(WARNING_TEXT));
        line.spawn((no_space, Visibility::Hidden, GiveNoSpace));
        line.spawn((
            Text::new("to you"),
            FONT.font(),
            TextColor(INK),
            GiveToWords,
        ));
        button(line, "Give").insert((ButtonStyle::Main, GiveButton, id("give")));
    });
}

/// How many: a box between - and +.
fn spawn_count(line: &mut ChildSpawnerCommands) {
    button(line, "-").insert((GiveCountStep(-1), id("less")));
    let count = TextBox {
        text: "1".to_owned(),
        ..digits()
    };
    let node = Node {
        width: px(COUNT_WIDTH),
        height: px(CHECK_ROW),
        ..default()
    };
    spawn_text_box(line, count, FONT, node).insert((GiveCount, id("count")));
    button(line, "+").insert((GiveCountStep(1), id("more")));
}

/// A widget of Give Item's form.
fn id(name: &'static str) -> UiId {
    UiId::new("debug.give_item", name)
}

fn digits() -> TextBox {
    TextBox {
        digits_only: true,
        takes_keys: true,
        ..default()
    }
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

fn dropdown<'a, S: Into<String>>(
    line: &'a mut ChildSpawnerCommands,
    choice: impl Bundle,
    options: impl IntoIterator<Item = S>,
    selected: usize,
    searchable: bool,
) -> EntityCommands<'a> {
    let dropdown = Dropdown {
        options: options.into_iter().map(Into::into).collect(),
        selected,
        searchable,
        ..default()
    };
    // Room on the right for the arrow, and some width for a blank choice.
    let node = Node {
        padding: UiRect {
            right: px(ARROW_ROOM),
            ..BUTTON_PADDING
        },
        min_width: px(BLANK_WIDTH),
        ..default()
    };
    let mut spawned = spawn_dropdown(line, dropdown, FONT, node);
    spawned.insert(choice);
    spawned
}
