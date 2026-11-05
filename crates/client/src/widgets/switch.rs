//! A switch: a rounded track, about the size of a phone's, with a tick in its
//! left half and a cross in its right, and a round thumb covering one of them.
//! A press anywhere on it flips it, and the thumb slides across (`switch/show`).
//! For settings that apply at once.

mod show;

use std::collections::HashMap;

use bevy::prelude::*;

pub(super) use show::show_switches;

use super::{ButtonPressed, ButtonState, Pressable, Stroke, UiId};

/// The track, across and high.
const TRACK: Vec2 = Vec2::new(52.0, 32.0);
/// Between the track's edge and the thumb.
const INSET: f32 = 3.0;
const THUMB: f32 = TRACK.y - 2.0 * INSET;
/// The tick's and the cross's box, across, centred in each half of the track.
const MARK_BOX: f32 = 14.0;
/// How long the thumb takes to slide across, in seconds.
const SLIDE_SECONDS: f32 = 0.15;

/// The tick: as a check button's, in a box `MARK_BOX` across.
const TICK: [Stroke; 2] = [(6.0, 1.0, 7.0, 45.0), (10.0, 4.0, 6.0, -45.0)];
/// The cross: two strokes 12 long, crossing in the middle of the box.
const CROSS: [Stroke; 2] = [(12.0, 1.0, 6.0, 45.0), (12.0, 1.0, 6.0, -45.0)];

/// Whether a switch is on. The switch flips it itself when pressed; set it to
/// change what it shows (the thumb jumps there).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Switch(pub bool);

/// The player flipped this switch, which is now `.1`. Setting `Switch` doesn't
/// send this.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchFlipped(pub Entity, pub bool);

/// What a slide is kept by: a switch's `UiId`, so a page built again while its
/// thumb is moving carries on sliding, or the switch itself if it has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum SlideKey {
    Id(UiId),
    Entity(Entity),
}

/// When each switch flipped by the player last started sliding, by the real
/// clock (Settings pauses single player, which stops `Time`). Old ones are
/// dropped once over.
#[derive(Resource, Default)]
pub(super) struct Slides(pub(super) HashMap<SlideKey, f32>);

/// The parts of a switch, for showing it.
#[derive(Component)]
pub(super) struct SwitchTrack(pub(super) Entity);
#[derive(Component)]
pub(super) struct SwitchThumb(pub(super) Entity);

/// A switch, `on` or off, with `node` for where it goes (its size is its own).
/// Disable it with `ButtonState::Disabled`. Insert its `UiId` on what this returns.
pub fn spawn_switch<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    on: bool,
    node: Node,
) -> EntityCommands<'a> {
    let mut switch = parent.spawn((
        Switch(on),
        state,
        Pressable,
        Node {
            width: px(TRACK.x),
            height: px(TRACK.y),
            flex_shrink: 0.0,
            ..node
        },
    ));
    let id = switch.id();
    switch.with_children(|inside| {
        inside
            .spawn((
                SwitchTrack(id),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(show::track_colour(
                    f32::from(u8::from(on)),
                    state.is_disabled(),
                    show::Hover::Away,
                )),
            ))
            .with_children(|track| {
                spawn_mark(track, 0.0, &TICK);
                spawn_mark(track, TRACK.x / 2.0, &CROSS);
            });
        inside.spawn((
            SwitchThumb(id),
            Node {
                position_type: PositionType::Absolute,
                left: px(show::thumb_left(f32::from(u8::from(on)))),
                top: px(INSET),
                width: px(THUMB),
                height: px(THUMB),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(show::THUMB_COLOUR),
            BoxShadow::new(show::THUMB_SHADOW, px(0), px(1), px(0), px(3)),
        ));
    });
    switch
}

/// A tick or cross in the half of the track starting `left`.
fn spawn_mark(track: &mut ChildSpawnerCommands, left: f32, strokes: &[Stroke]) {
    track
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: px(left + (TRACK.x / 2.0 - MARK_BOX) / 2.0),
            top: px((TRACK.y - MARK_BOX) / 2.0),
            width: px(MARK_BOX),
            height: px(MARK_BOX),
            ..default()
        })
        .with_children(|mark| super::spawn_strokes(mark, strokes, show::MARK_COLOUR));
}

/// A press flips a switch, starts its slide and says so.
pub(super) fn flip_switches(
    time: Res<Time<Real>>,
    mut pressed: MessageReader<ButtonPressed>,
    mut switches: Query<(&mut Switch, &ButtonState, Option<&UiId>)>,
    mut slides: ResMut<Slides>,
    mut flipped: MessageWriter<SwitchFlipped>,
) {
    let now = time.elapsed_secs();
    slides
        .0
        .retain(|_, &mut started| now - started < SLIDE_SECONDS);
    for &ButtonPressed(entity) in pressed.read() {
        let Ok((mut switch, state, id)) = switches.get_mut(entity) else {
            continue;
        };
        if state.is_disabled() {
            continue;
        }
        switch.0 = !switch.0;
        let key = id.map_or(SlideKey::Entity(entity), |&id| SlideKey::Id(id));
        // Flipped back mid-slide, it turns round where it is.
        let done = slides.0.get(&key).map_or(1.0, |&at| show::share(at, now));
        slides.0.insert(key, now - (1.0 - done) * SLIDE_SECONDS);
        flipped.write(SwitchFlipped(entity, switch.0));
    }
}

#[cfg(test)]
mod tests;
