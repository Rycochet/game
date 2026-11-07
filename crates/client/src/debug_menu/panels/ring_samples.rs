//! Show Widgets' row of skill rings: one at each fifth's colour, a sliver, a full
//! ring, and a specialised one with its second ring half way.

use bevy::prelude::*;
use shared::sheet::Attribute;

use crate::widgets::{INK, Shade, SkillRing, attribute_colour, skill_ring};

const SIDE: f32 = 48.0;

pub(super) fn spawn_ring_samples(content: &mut ChildSpawnerCommands) {
    content
        .spawn(Node {
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            let fill = attribute_colour(&[Attribute::Agility], Shade::Skill);
            for (value, specialised) in [
                (1.0, false),
                (15.0, false),
                (30.0, false),
                (50.0, false),
                (70.0, false),
                (90.0, false),
                (100.0, false),
                (110.0, true),
            ] {
                row.spawn((
                    Node {
                        width: px(SIDE),
                        height: px(SIDE),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(fill),
                    BorderColor::all(INK),
                ))
                .with_child(skill_ring(SkillRing::of(value, specialised)));
            }
        });
}
