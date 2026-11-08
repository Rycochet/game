//! The Abilities page: every ability the character knows, flowing in rows, Attack
//! first. Drawn again when the body they are in changes.

use bevy::prelude::*;
use shared::ability::{Ability, known};
use shared::creature::CreatureKind;

use super::icons::icon_path;
use crate::common::AbilitiesPage;
use crate::inventory::Carried;
use crate::widgets::{ABILITY, AbilityLook, Shade, UiId, attribute_colour, spawn_ability};

/// How many abilities a row holds before the next row starts.
const PER_ROW: f32 = 8.0;
const GAP: f32 = 6.0;

/// An ability on the page, which can be dragged to the action bar.
#[derive(Component, Clone, Copy)]
pub(super) struct PageAbility(pub Ability);

/// The body a page was drawn for.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) struct DrawnFor(CreatureKind);

/// Fills each Abilities page that is new, or was drawn for another body.
pub(super) fn fill_pages(
    pages: Query<(Entity, Option<&DrawnFor>), With<AbilitiesPage>>,
    carried: Res<Carried>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let kind = carried.0.creature;
    for (page, drawn) in &pages {
        if drawn == Some(&DrawnFor(kind)) {
            continue;
        }
        let mut page = commands.entity(page);
        page.despawn_children().insert(DrawnFor(kind));
        page.with_children(|page| spawn_contents(page, kind, &assets));
    }
}

fn spawn_contents(page: &mut ChildSpawnerCommands, kind: CreatureKind, assets: &AssetServer) {
    page.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(GAP),
        row_gap: px(GAP),
        width: px(PER_ROW * ABILITY + (PER_ROW - 1.0) * GAP),
        ..default()
    })
    .with_children(|list| {
        for ability in known(kind) {
            let frame = spawn_ability(list, look(ability, kind, assets));
            let id = UiId::new("abilities", ability_id(ability));
            list.commands()
                .entity(frame)
                .insert((id, PageAbility(ability)));
        }
    });
}

/// How an ability shows on the page.
pub(super) fn look(ability: Ability, kind: CreatureKind, assets: &AssetServer) -> AbilityLook {
    AbilityLook {
        icon: assets.load(icon_path(ability)),
        name: format!(
            "{}
{}",
            ability.name(),
            ability.description()
        ),
        passive: !ability.is_active(),
        // A passive ability is never done, so it is never shown as unusable.
        usable: !ability.is_active() || ability.can_use(kind),
        colour: attribute_colour(ability.leans(), Shade::of_ability(!ability.is_active())),
    }
}

/// Its name in widget ids: `abilities/<id>`.
pub(super) fn ability_id(ability: Ability) -> &'static str {
    match ability {
        Ability::Attack => "attack",
        Ability::Heal => "heal",
        Ability::Flutter => "flutter",
        Ability::Kick => "kick",
        Ability::NightVision => "night_vision",
        Ability::Trample => "trample",
        Ability::Sneak => "sneak",
    }
}
