//! A quest boss as everyone sees it (`EntityLook::boss`): named on hover by
//! its own name and epithet ("Grukk the Foul"), and glowing in its tier's
//! quality colour (owner).

use bevy::prelude::*;
use shared::mob::BossLook;

use crate::common::{HoverName, TokenGlow, Wording, quality_colour};

/// What is known of a quest boss among the others, until named.
#[derive(Component)]
pub(super) struct SeenBoss(pub(super) BossLook);

/// Names each newly seen boss in the player's language and sets its glow.
pub(super) fn show_bosses(
    seen: Query<(Entity, &SeenBoss), Added<SeenBoss>>,
    wording: Res<Wording>,
    mut commands: Commands,
) {
    for (entity, SeenBoss(look)) in &seen {
        let colour = quality_colour(look.tier).to_srgba();
        let rgb = [colour.red, colour.green, colour.blue].map(|c| (c * 255.0).round() as u8);
        let mut marks = commands.entity(entity);
        marks.insert(TokenGlow(rgb));
        if let Ok(name) = look.boss.full_name(&wording.0) {
            marks.insert(HoverName(name));
        }
    }
}
