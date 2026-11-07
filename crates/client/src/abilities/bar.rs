//! The action bar: ten slots, five either side of the Health and Mana circle, each
//! with its key's short name over its bottom right. A slot whose ability is used
//! by itself has a sliding metal edge, as set items do.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use shared::ability::{Ability, ActionBar, BAR_SLOTS};
use shared::creature::CreatureKind;

use super::drag::Aim;
use super::own_bar::OwnBar;
use super::page::look;
use super::vitals_now::affords;
use crate::common::{Action, OwnVitals, Preferences, VitalsCircle, interface_area};
use crate::inventory::Carried;
use crate::widgets::{
    ABILITY, ALLOWED, Floating, GRID_LINE, REFUSED, TextSize, UiId, set_edge, spawn_ability,
    spawn_outlined_text, stops_presses,
};

/// Between slots, and between the bar and the circle.
pub(super) const GAP: f32 = 6.0;
/// How many slots stand on each side of the circle.
const PER_SIDE: usize = BAR_SLOTS / 2;
const EMPTY_EDGE: f32 = 2.0;
const EMPTY_BACKGROUND: Color = Color::srgba(0.1, 0.08, 0.06, 0.6);
/// How thick the edge of a slot used by itself is.
const AUTO_EDGE: f32 = 3.0;

/// One side of the bar: 0 left of the circle, 1 right.
#[derive(Component)]
pub(super) struct BarSide(usize);

/// One slot, holding its place on the bar.
#[derive(Component, Clone, Copy)]
pub(super) struct SlotNode(pub u8);

/// The two sides and their slots, empty until `show_bar` fills them.
pub(super) fn spawn_bar(mut commands: Commands) {
    for side in 0..2 {
        let row = commands
            .spawn((
                BarSide(side),
                Node {
                    position_type: PositionType::Absolute,
                    column_gap: px(GAP),
                    ..default()
                },
                Visibility::Hidden,
            ))
            .id();
        commands.entity(row).insert(stops_presses(row));
        commands.entity(row).with_children(|row| {
            for index in 0..PER_SIDE {
                let slot = (side * PER_SIDE + index) as u8;
                row.spawn((
                    SlotNode(slot),
                    UiId::new("bar", "slot").at(slot as usize),
                    Node {
                        width: px(ABILITY),
                        height: px(ABILITY),
                        ..default()
                    },
                ));
            }
        });
    }
}

/// Stands each side by the circle, hidden while it is.
pub(super) fn place_bar(
    circle: Res<VitalsCircle>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut sides: Query<(&BarSide, &mut Node, &mut Visibility)>,
) {
    let room = interface_area(&window, &ui_scale);
    let width = PER_SIDE as f32 * (ABILITY + GAP) - GAP;
    for (side, mut node, mut visibility) in &mut sides {
        if circle.across <= 0.0 {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        let half = circle.across / 2.0 + GAP;
        let left = match side.0 {
            0 => room.x / 2.0 - half - width,
            _ => room.x / 2.0 + half,
        };
        node.left = px(left.round());
        node.bottom = px(circle.bottom);
    }
}

/// What the slots last showed.
#[derive(PartialEq)]
pub(super) struct Shown {
    bar: ActionBar,
    kind: CreatureKind,
    keys: Vec<(String, String)>,
    aim: Option<(u8, bool)>,
    /// Too little Mana for what costs it (all cost the same so far: Heal's).
    low_mana: bool,
}

/// Draws each slot again when what it shows changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_bar(
    bar: Res<OwnBar>,
    carried: Res<Carried>,
    preferences: Res<Preferences>,
    aim: Res<Aim>,
    (vitals, time): (Res<OwnVitals>, Res<Time>),
    assets: Res<AssetServer>,
    slots: Query<(Entity, &SlotNode)>,
    mut shown: Local<Option<Shown>>,
    mut commands: Commands,
) {
    let keys = (0..BAR_SLOTS as u8)
        .map(|slot| {
            let [first, _] = preferences.controls.keys(Action::UseSlot(slot));
            first.map_or_else(Default::default, |key| (key.short_label(), key.label()))
        })
        .collect();
    let now = Shown {
        bar: bar.0.clone(),
        kind: carried.0.creature,
        keys,
        aim: aim.0,
        low_mana: !affords(Ability::Heal, &vitals, time.elapsed_secs_f64()),
    };
    if shown.as_ref() == Some(&now) {
        return;
    }
    for (entity, &SlotNode(slot)) in &slots {
        let mut node = commands.entity(entity);
        node.despawn_children();
        node.with_children(|inside| spawn_slot(inside, slot, &now, &assets));
    }
    *shown = Some(now);
}

/// What is in one slot: its ability or an empty frame, its key, and its edge.
fn spawn_slot(inside: &mut ChildSpawnerCommands, slot: u8, now: &Shown, assets: &AssetServer) {
    let held = now.bar.slots[slot as usize];
    let (short, full) = &now.keys[slot as usize];
    match held {
        Some(held) => {
            let mut look = look(held.ability, now.kind, assets);
            let ability = held.ability;
            if !full.is_empty() {
                look.name = format!(
                    "{} ({full})
{}",
                    ability.name(),
                    ability.description()
                );
            }
            // Greyed while there isn't the Mana for it.
            look.usable &= !now.low_mana || ability.mana_cost() == 0;
            spawn_ability(inside, look);
            if held.ability.is_active() {
                super::wait::spawn_shade(inside);
            }
        }
        None => {
            inside.spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    border: UiRect::all(px(EMPTY_EDGE)),
                    border_radius: BorderRadius::all(px(4.0)),
                    ..default()
                },
                BackgroundColor(EMPTY_BACKGROUND),
                BorderColor::from(GRID_LINE),
            ));
        }
    }
    if held.is_some_and(|held| held.auto) {
        inside.spawn(set_edge(None, UiRect::all(px(AUTO_EDGE)), 0.0));
    }
    if let Some((_, allowed)) = now.aim.filter(|(aimed, _)| *aimed == slot) {
        let colour = if allowed { ALLOWED } else { REFUSED };
        inside.spawn((
            Floating,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                border: UiRect::all(px(AUTO_EDGE)),
                border_radius: BorderRadius::all(px(4.0)),
                ..default()
            },
            BorderColor::from(colour),
        ));
    }
    if !short.is_empty() {
        inside
            .spawn((
                Floating,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(3.0),
                    bottom: px(1.0),
                    ..default()
                },
            ))
            .with_children(|corner| {
                spawn_outlined_text(corner, short, TextSize::Small);
            });
    }
}
