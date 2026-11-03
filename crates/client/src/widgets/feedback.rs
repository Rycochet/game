//! How widgets answer the pointer: looks for being hovered and pressed, a ripple
//! from the point pressed, and the names of buttons after a short wait.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};

use super::item::{Floating, ItemTip, ItemTooltip, logical_rect, shown_at};
use super::moveable::PanelPart;
use super::slider::Slider;
use super::text_box::{TextBox, TextFocus};
use super::theme::{ERROR_EDGE, HOVER_EDGE, MAIN_BACKGROUND, PARCHMENT, RIPPLE};
use super::{BUTTON_BORDER, ButtonState, INK, Pressable, raised_border, sunken_border};
use crate::common::UiPointer;

const LIGHTER_WHEN_HOVERED: f32 = 0.06;
const DARKER_WHEN_PRESSED: f32 = 0.08;
/// How long a ripple takes to spread and fade, in seconds.
const RIPPLE_SECONDS: f32 = 0.35;
/// How long the pointer stays on a button before its name shows, in seconds.
const TOOLTIP_DELAY: f32 = 0.5;
const TOOLTIP_OFFSET: Vec2 = Vec2::new(16.0, 16.0);

/// How much a button stands out; leave it off for an ordinary one.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    #[default]
    Plain,
    /// The main thing to do here: one to an area.
    Main,
    /// Something less often wanted: no box until the pointer is on it.
    Quiet,
    /// A tab: when chosen, of a piece with the panel below it.
    Folder,
}

/// The name of a button that only shows a picture, for hovering over it.
#[derive(Component, Clone, Debug)]
pub struct ButtonTooltip(pub String);

/// What the pointer is doing to a button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Touch {
    Away,
    Hovered,
    Pressed,
}

/// What the pointer is on.
#[derive(Resource, Default)]
pub(super) struct Pointed {
    /// The front-most widget under it, unless disabled.
    hovered: Option<Entity>,
    /// When `hovered` last changed, in seconds.
    since: f32,
    /// The widget the button being held went down on.
    pressed: Option<Entity>,
}

/// A spreading circle in a button, in a holder that cuts it to the button.
#[derive(Component)]
pub(super) struct Ripple {
    age: f32,
    centre: Vec2,
    reach: f32,
}

/// A button's background, border and text colours.
fn look(state: ButtonState, style: ButtonStyle, touch: Touch) -> (Color, BorderColor, Color) {
    let (mut background, mut border, mut text) = state.colours();
    if state == ButtonState::Selected && style == ButtonStyle::Folder {
        let [red, green, blue] = PARCHMENT;
        background = Color::srgb_u8(red, green, blue);
        border = BorderColor {
            bottom: background,
            ..raised_border()
        };
        text = INK;
    }
    if state == ButtonState::Normal {
        match style {
            ButtonStyle::Plain | ButtonStyle::Folder => {}
            ButtonStyle::Main => background = MAIN_BACKGROUND,
            ButtonStyle::Quiet if touch == Touch::Away => {
                background = Color::NONE;
                border = BorderColor::from(Color::NONE);
            }
            ButtonStyle::Quiet => {}
        }
    }
    if state.is_disabled() {
        return (background, border, text);
    }
    match touch {
        Touch::Away => {}
        Touch::Hovered => background = background.lighter(LIGHTER_WHEN_HOVERED),
        Touch::Pressed => {
            background = background.darker(DARKER_WHEN_PRESSED);
            // Pushed in: the light now falls on its other edges.
            if state == ButtonState::Normal {
                border = sunken_border();
            }
        }
    }
    (background, border, text)
}

/// How far across a ripple is and how strongly it shows, `age` seconds on, out of
/// `reach` across at its widest.
fn ripple_shape(age: f32, reach: f32) -> (f32, f32) {
    let done = (age / RIPPLE_SECONDS).clamp(0.0, 1.0);
    (reach * done, (1.0 - done) * RIPPLE.alpha())
}

/// Notes what the pointer is on, and starts a ripple on a press. Runs before
/// anything takes the press.
#[allow(clippy::type_complexity)]
pub(super) fn track_pointer(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    widgets: Query<
        (
            Entity,
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            Option<&ButtonState>,
        ),
        Or<(With<Pressable>, With<PanelPart>, With<ButtonState>)>,
    >,
    mut pointed: ResMut<Pointed>,
) {
    let front = pointer.0.and_then(|at| {
        widgets
            .iter()
            .filter(|(_, _, node, transform, visible, clip, _)| {
                visible.get() && shown_at(at, node, transform, *clip)
            })
            .max_by_key(|(_, stack, ..)| stack.0)
    });
    let usable = front.filter(|(.., state)| !state.is_some_and(|state| state.is_disabled()));
    let under = usable.map(|(entity, ..)| entity);
    let pressing = mouse.just_pressed(MouseButton::Left);
    if pressing {
        pointed.pressed = under;
    } else if !mouse.pressed(MouseButton::Left) {
        pointed.pressed = None;
    }
    // Nothing else lights up while something is being dragged about.
    let held_elsewhere = mouse.pressed(MouseButton::Left) && pointed.pressed != under;
    let hovered = under.filter(|_| !held_elsewhere);
    if pointed.hovered != hovered {
        pointed.hovered = hovered;
        pointed.since = time.elapsed_secs();
    }
    if !pressing {
        return;
    }
    let (Some(at), Some((button, _, node, transform, _, _, Some(_)))) = (pointer.0, usable) else {
        return;
    };
    let (corner, size) = logical_rect(node, transform);
    commands.entity(button).with_children(|inside| {
        inside
            .spawn((
                Floating,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ))
            .with_child((
                Floating,
                Ripple {
                    age: 0.0,
                    centre: at - corner - Vec2::splat(BUTTON_BORDER),
                    // From a corner, it still reaches the far one.
                    reach: 2.0 * size.length(),
                },
                Node {
                    position_type: PositionType::Absolute,
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
    });
}

/// Spreads ripples, and removes them when they have faded.
pub(super) fn spread_ripples(
    mut commands: Commands,
    time: Res<Time>,
    mut ripples: Query<(&mut Ripple, &mut Node, &mut BackgroundColor, &ChildOf)>,
) {
    for (mut ripple, mut node, mut colour, holder) in &mut ripples {
        ripple.age += time.delta_secs();
        if ripple.age >= RIPPLE_SECONDS {
            commands.entity(holder.parent()).despawn();
            continue;
        }
        let (across, strength) = ripple_shape(ripple.age, ripple.reach);
        node.left = px(ripple.centre.x - across / 2.0);
        node.top = px(ripple.centre.y - across / 2.0);
        node.width = px(across);
        node.height = px(across);
        colour.0 = RIPPLE.with_alpha(strength);
    }
}

/// Makes buttons look as their state and the pointer say. After `Update`, so a
/// button spawned then is right from the first frame it is drawn.
#[allow(clippy::type_complexity)]
pub(super) fn show_button_looks(
    pointed: Res<Pointed>,
    mut buttons: Query<(
        Entity,
        &ButtonState,
        Option<&ButtonStyle>,
        &mut BackgroundColor,
        &mut BorderColor,
        &Children,
    )>,
    mut labels: Query<&mut TextColor>,
) {
    for (entity, state, style, mut background, mut border, children) in &mut buttons {
        let touch = if pointed.pressed == Some(entity) && pointed.hovered == Some(entity) {
            Touch::Pressed
        } else if pointed.hovered == Some(entity) {
            Touch::Hovered
        } else {
            Touch::Away
        };
        let style = style.copied().unwrap_or_default();
        let (new_background, new_border, new_text) = look(*state, style, touch);
        background.set_if_neq(BackgroundColor(new_background));
        border.set_if_neq(new_border);
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                text.set_if_neq(TextColor(new_text));
            }
        }
    }
}

/// The edge of a text box: what is wrong, then who has it, then the pointer.
fn text_box_edge(error: bool, focused: bool, hovered: bool) -> BorderColor {
    if error {
        BorderColor::from(ERROR_EDGE)
    } else if focused {
        BorderColor::from(INK)
    } else if hovered {
        BorderColor::from(HOVER_EDGE)
    } else {
        sunken_border()
    }
}

/// Text boxes and sliders have no state of their own to show: their edges change.
#[allow(clippy::type_complexity)]
pub(super) fn show_edges(
    pointed: Res<Pointed>,
    focus: Res<TextFocus>,
    mut boxes: Query<(Entity, &TextBox, &mut BorderColor)>,
    mut sliders: Query<(Entity, &mut BorderColor), (With<Slider>, Without<TextBox>)>,
) {
    for (entity, text_box, mut border) in &mut boxes {
        border.set_if_neq(text_box_edge(
            text_box.error,
            focus.0 == Some(entity),
            pointed.hovered == Some(entity),
        ));
    }
    for (entity, mut border) in &mut sliders {
        let colour = if pointed.hovered == Some(entity) {
            HOVER_EDGE
        } else {
            INK
        };
        border.set_if_neq(BorderColor::from(colour));
    }
}

/// Asks for the name of the button the pointer has stayed on, if it has one.
pub(super) fn show_button_tooltips(
    time: Res<Time>,
    pointer: Res<UiPointer>,
    pointed: Res<Pointed>,
    names: Query<&ButtonTooltip>,
    mut tooltip: ResMut<ItemTooltip>,
) {
    let (Some(at), Some(hovered), None) = (pointer.0, pointed.hovered, pointed.pressed) else {
        return;
    };
    if time.elapsed_secs() - pointed.since < TOOLTIP_DELAY {
        return;
    }
    if let Ok(name) = names.get(hovered) {
        tooltip.0 = Some(ItemTip {
            words: name.0.clone(),
            quality: None,
            set: false,
            at: at + TOOLTIP_OFFSET,
        });
    }
}

#[cfg(test)]
mod tests;
