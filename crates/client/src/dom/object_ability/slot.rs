//! Drawing a slot for an ability: its ability or an empty frame, its key, and
//! the edges saying how it is used.

use bevy::prelude::*;

use super::{AbilityCell, AbilitySlotShown, SlotPart};
use crate::widgets::{
    ABILITY, ALLOWED, Floating, GRID_LINE, REFUSED, TextSize, queued_stripe, set_edge,
    spawn_ability, spawn_outlined_text,
};

const EMPTY_EDGE: f32 = 2.0;
const EMPTY_BACKGROUND: Color = Color::srgba(0.1, 0.08, 0.06, 0.6);
/// How thick the edge of a slot used by itself is, and of one that is on or
/// aimed at.
const EDGE: f32 = 3.0;
/// How thick the stripes of a queued slot are, inside the edge.
const QUEUED_STRIPE: f32 = 3.0;
/// How dark the shade is.
const SHADE: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);

/// Draws each slot again when what it shows changes.
pub(in crate::dom) fn draw_ability_slots(
    mut commands: Commands,
    changed: Query<(Entity, &AbilitySlotShown), Changed<AbilitySlotShown>>,
) {
    for (slot, shown) in &changed {
        let node = Node {
            width: px(ABILITY),
            height: px(ABILITY),
            ..default()
        };
        let mut entity = commands.entity(slot);
        entity.insert(node).despawn_related::<Children>();
        entity.with_children(|inside| draw(inside, slot, shown));
    }
}

/// An edge over the whole slot.
fn edge(colour: Color) -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100.0),
        height: percent(100.0),
        border: UiRect::all(px(EDGE)),
        border_radius: BorderRadius::all(px(4.0)),
        ..default()
    };
    (Floating, node, BorderColor::from(colour))
}

fn draw(inside: &mut ChildSpawnerCommands, slot: Entity, shown: &AbilitySlotShown) {
    match &shown.thing {
        Some(thing) => {
            let frame = spawn_ability(inside, thing.look.clone());
            let cell = AbilityCell {
                of: slot,
                ability: thing.ability,
            };
            inside.commands().entity(frame).insert(cell);
        }
        None => {
            let node = Node {
                width: percent(100.0),
                height: percent(100.0),
                border: UiRect::all(px(EMPTY_EDGE)),
                border_radius: BorderRadius::all(px(4.0)),
                ..default()
            };
            let colours = (
                BackgroundColor(EMPTY_BACKGROUND),
                BorderColor::from(GRID_LINE),
            );
            inside.spawn((node, colours));
        }
    }
    if shown.shade {
        // Empty until its owner gives it a height.
        let node = Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(0),
            height: percent(0),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        };
        let shade = (SlotPart::Shade(slot), Floating, node);
        inside.spawn((shade, BackgroundColor(SHADE)));
    }
    if shown.auto {
        inside.spawn(set_edge(None, UiRect::all(px(EDGE)), 0.0));
    }
    if shown.on {
        inside.spawn((edge(ALLOWED), SlotPart::On(slot)));
    }
    if shown.queued {
        let stripes = queued_stripe(QUEUED_STRIPE, EDGE);
        inside.spawn((stripes, SlotPart::Queued(slot)));
    }
    if let Some(allowed) = shown.aim {
        inside.spawn(edge(if allowed { ALLOWED } else { REFUSED }));
    }
    if !shown.key.is_empty() {
        let corner = Node {
            position_type: PositionType::Absolute,
            right: px(3.0),
            bottom: px(1.0),
            ..default()
        };
        inside.spawn((Floating, corner)).with_children(|corner| {
            spawn_outlined_text(corner, &shown.key, TextSize::Small);
        });
    }
}
