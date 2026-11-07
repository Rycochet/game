//! Spawning buttons: plain, icon, check and close, with their strokes.

use bevy::prelude::*;

use super::theme::{INK, TextSize};
use super::{
    BUTTON_BORDER, ButtonState, CHECK_GAP, CHECK_PIXELS, CLOSE_FONT, CLOSE_PIXELS, STROKE_THICKNESS,
};

/// The two strokes of a tick: each one's length, where it goes and how it is turned.
pub(super) const TICK_STROKES: [Stroke; 2] = [(6.0, 1.0, 7.0, 45.0), (10.0, 4.0, 6.0, -45.0)];

/// Between the edge of a button and its picture.
pub(super) const ICON_PADDING: f32 = 4.0;
/// A small icon button, across: for controls on the edge of something, such as a grid.
const SMALL_ICON_BUTTON: f32 = 22.0;
/// Between the edge of a small icon button and its picture.
const SMALL_ICON_PADDING: f32 = 2.0;

/// A button. `node` says its size or padding and where it goes; its border and
/// the centring of its words are set here.
pub fn spawn_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    label: impl Into<String>,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, text) = state.colours();
    let mut button = parent.spawn((
        state,
        Node {
            border: UiRect::all(px(BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_child((
        Text::new(label),
        size.font(),
        TextLayout::justify(Justify::Center),
        TextColor(text),
    ));
    button
}

/// A button showing a picture rather than words. Give it a `ButtonTooltip` to say
/// what it is.
pub fn spawn_icon_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    icon: Handle<Image>,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, _) = state.colours();
    let mut button = parent.spawn((
        state,
        Node {
            border: UiRect::all(px(BUTTON_BORDER)),
            padding: UiRect::all(px(ICON_PADDING)),
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_child((
        ImageNode::new(icon),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
    ));
    button
}

/// An icon button `SMALL_ICON_BUTTON` across, with `node` for where it goes.
pub fn spawn_small_icon_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    icon: Handle<Image>,
    node: Node,
) -> EntityCommands<'a> {
    let mut button = spawn_icon_button(parent, state, icon, node);
    button.entry::<Node>().and_modify(|mut node| {
        node.width = px(SMALL_ICON_BUTTON);
        node.height = px(SMALL_ICON_BUTTON);
        node.flex_shrink = 0.0;
        node.padding = UiRect::all(px(SMALL_ICON_PADDING));
    });
    button
}

/// A button with its words on the left, after a checkbox if `checked` says
/// whether it is ticked, or the same space if it has none, so words line up.
pub fn spawn_check_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    checked: Option<bool>,
    label: impl Into<String>,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, text) = state.colours();
    let mut button = parent.spawn((
        state,
        Node {
            border: UiRect::all(px(BUTTON_BORDER)),
            padding: UiRect::horizontal(px(CHECK_GAP)),
            column_gap: px(CHECK_GAP),
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    if let Some(checked) = checked {
        button.insert(super::Checked(checked));
    }
    button.with_children(|inside| {
        let mut slot = inside.spawn(Node {
            width: px(CHECK_PIXELS),
            height: px(CHECK_PIXELS),
            flex_shrink: 0.0,
            border: UiRect::all(px(checked.map_or(0.0, |_| BUTTON_BORDER))),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        });
        slot.insert(BorderColor::from(INK));
        if checked.is_some() {
            // Always there, shown by `Checked`, so changing that is enough.
            slot.with_children(|slot| {
                slot.spawn((
                    CheckTick,
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    visible_if(checked == Some(true)),
                ))
                .with_children(|tick| spawn_strokes(tick, &TICK_STROKES, INK));
            });
        }
        inside.spawn((Text::new(label), size.font(), TextColor(text)));
    });
    button
}

/// A check button's tick, inside its box.
#[derive(Component)]
pub(super) struct CheckTick;

fn visible_if(shown: bool) -> Visibility {
    if shown {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// Shows or hides each check button's tick as its `Checked` changes.
pub(super) fn show_checks(
    buttons: Query<(&super::Checked, &Children), Changed<super::Checked>>,
    boxes: Query<&Children>,
    mut ticks: Query<&mut Visibility, With<CheckTick>>,
) {
    for (checked, inside) in &buttons {
        for tick in inside
            .iter()
            .flat_map(|slot| boxes.get(slot).into_iter().flatten())
        {
            if let Ok(mut visibility) = ticks.get_mut(*tick) {
                visibility.set_if_neq(visible_if(checked.0));
            }
        }
    }
}

/// A straight line drawn on something: its length, where it goes and how it is turned.
pub(super) type Stroke = (f32, f32, f32, f32);

/// Lines making up a small mark, such as a tick, in `colour`.
pub(super) fn spawn_strokes(parent: &mut ChildSpawnerCommands, strokes: &[Stroke], colour: Color) {
    for &(length, left, top, degrees) in strokes {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(length),
                height: px(STROKE_THICKNESS),
                ..default()
            },
            UiTransform::from_rotation(Rot2::degrees(degrees)),
            BackgroundColor(colour),
        ));
    }
}

/// A close button, with `node` for where it goes.
pub fn spawn_close_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, _) = ButtonState::Normal.colours();
    let mut button = parent.spawn((
        ButtonState::Normal,
        Node {
            width: px(CLOSE_PIXELS),
            height: px(CLOSE_PIXELS),
            border: UiRect::all(px(BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_child((Text::new("X"), CLOSE_FONT.font(), TextColor(INK)));
    button
}
