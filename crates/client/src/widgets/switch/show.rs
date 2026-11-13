//! Showing switches: where the thumb is along its slide, and the track's colour
//! by where the thumb is, the pointer and whether it can be used. A disabled
//! one is plainly so: a flat grey track whichever way it is set, a dim thumb
//! with no shadow, faint marks and no change on hovering.

use bevy::prelude::*;

use super::super::feedback::Pointed;
use super::super::theme::{DISABLED_BACKGROUND, DISABLED_EDGE};
use super::super::{ButtonState, UiId};
use super::{
    INSET, SLIDE_SECONDS, SlideKey, Slides, Switch, SwitchMark, SwitchThumb, SwitchTrack, THUMB,
    TRACK,
};
use crate::common::Preferences;

/// The track when on: a green that sits well on parchment.
const ON: Color = Color::srgb(0.36, 0.56, 0.28);
/// The track when off: a warm grey.
const OFF: Color = Color::srgb(0.6, 0.55, 0.46);
pub(super) const THUMB_COLOUR: Color = Color::srgb(0.98, 0.95, 0.88);
pub(super) const THUMB_SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
/// The tick and cross, light on the track.
pub(super) const MARK_COLOUR: Color = THUMB_COLOUR;
/// A disabled thumb: the thumb dimmed towards the disabled grey.
const DISABLED_THUMB: Color = Color::srgb(0.8, 0.77, 0.71);
/// How much of a mark shows on a disabled track.
const DISABLED_MARK_ALPHA: f32 = 0.35;
const LIGHTER_WHEN_HOVERED: f32 = 0.06;
const DARKER_WHEN_PRESSED: f32 = 0.08;

/// What the pointer is doing to a switch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Hover {
    Away,
    Over,
    Down,
}

/// How much of a slide started at `started` is done at `now`, `0..=1`.
pub(super) fn share(started: f32, now: f32) -> f32 {
    ((now - started) / SLIDE_SECONDS).clamp(0.0, 1.0)
}

/// Where the thumb is, from off (0) to on (1), eased in and out.
pub(super) fn place(on: bool, done: f32) -> f32 {
    let eased = done * done * (3.0 - 2.0 * done);
    if on { eased } else { 1.0 - eased }
}

/// The thumb's left edge in the track, with it `place` of the way to on.
pub(super) fn thumb_left(place: f32) -> f32 {
    INSET + place * (TRACK.x - THUMB - 2.0 * INSET)
}

/// The track's colour, with the thumb `place` of the way to on.
pub(super) fn track_colour(place: f32, disabled: bool, hover: Hover) -> Color {
    if disabled {
        return DISABLED_EDGE.mix(&DISABLED_BACKGROUND, 0.5);
    }
    let colour: Color = Srgba::from(OFF).mix(&Srgba::from(ON), place).into();
    match hover {
        Hover::Away => colour,
        Hover::Over => colour.lighter(LIGHTER_WHEN_HOVERED),
        Hover::Down => colour.darker(DARKER_WHEN_PRESSED),
    }
}

/// The thumb's colour, dimmed when disabled.
pub(super) fn thumb_colour(disabled: bool) -> Color {
    if disabled {
        DISABLED_THUMB
    } else {
        THUMB_COLOUR
    }
}

/// The thumb's shadow, gone when disabled: it lies flat.
pub(super) fn shadow_colour(disabled: bool) -> Color {
    if disabled { Color::NONE } else { THUMB_SHADOW }
}

/// The tick's and the cross's colour, faint when disabled.
pub(super) fn mark_colour(disabled: bool) -> Color {
    match disabled {
        true => MARK_COLOUR.with_alpha(DISABLED_MARK_ALPHA),
        false => MARK_COLOUR,
    }
}

/// Moves thumbs along their slides and colours tracks. With Reduce Motion on,
/// thumbs jump.
#[allow(clippy::too_many_arguments)]
pub(in crate::widgets) fn show_switches(
    time: Res<Time<Real>>,
    preferences: Res<Preferences>,
    pointed: Res<Pointed>,
    slides: Res<Slides>,
    switches: Query<(&Switch, &ButtonState, Option<&UiId>)>,
    mut tracks: Query<(&SwitchTrack, &mut BackgroundColor)>,
    mut thumbs: Query<
        (
            &SwitchThumb,
            &mut Node,
            &mut BackgroundColor,
            &mut BoxShadow,
        ),
        Without<SwitchTrack>,
    >,
    marks: Query<(&SwitchMark, &Children)>,
    mut strokes: Query<&mut BackgroundColor, (Without<SwitchTrack>, Without<SwitchThumb>)>,
) {
    let now = time.elapsed_secs();
    let place_of = |entity: Entity| {
        let (switch, state, id) = switches.get(entity).ok()?;
        let key = id.map_or(SlideKey::Entity(entity), |&id| SlideKey::Id(id));
        let done = match (preferences.graphics.reduce_motion, slides.0.get(&key)) {
            (false, Some(&started)) => share(started, now),
            _ => 1.0,
        };
        Some((place(switch.0, done), state.is_disabled()))
    };
    for (track, mut background) in &mut tracks {
        let Some((place, disabled)) = place_of(track.0) else {
            continue;
        };
        let hover = match (
            pointed.hovered == Some(track.0),
            pointed.pressed == Some(track.0),
        ) {
            (true, true) => Hover::Down,
            (true, false) => Hover::Over,
            _ => Hover::Away,
        };
        background.set_if_neq(BackgroundColor(track_colour(place, disabled, hover)));
    }
    for (thumb, mut node, mut background, mut shadow) in &mut thumbs {
        let Some((place, disabled)) = place_of(thumb.0) else {
            continue;
        };
        let left = px(thumb_left(place));
        if node.left != left {
            node.left = left;
        }
        background.set_if_neq(BackgroundColor(thumb_colour(disabled)));
        let shade = shadow_colour(disabled);
        if let Some(style) = shadow.0.first_mut()
            && style.color != shade
        {
            style.color = shade;
        }
    }
    for (mark, children) in &marks {
        let Some((_, disabled)) = place_of(mark.0) else {
            continue;
        };
        let colour = BackgroundColor(mark_colour(disabled));
        for &child in children {
            if let Ok(mut stroke) = strokes.get_mut(child) {
                stroke.set_if_neq(colour);
            }
        }
    }
}
