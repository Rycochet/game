//! Abilities: the Character panel's Abilities page, every ability the character
//! knows, and the action bar either side of the Health and Mana circle. See
//! `docs/context/client/abilities.md`.

use bevy::prelude::*;

use crate::common::PointerSystems;
use crate::dom::{DrawObjects, HoldObjects};
use crate::player::PlayerSystems;
use crate::widgets::WidgetSystems;

mod bar;
mod cancel;
mod drag;
mod held;
mod icons;
mod manual;
mod mark;
mod own_bar;
mod page;
mod sneak;
mod spots;
mod use_ability;
mod vitals_now;
mod wait;

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<own_bar::OwnBar>()
            .init_resource::<drag::Drag>()
            .init_resource::<drag::Aim>()
            .init_resource::<manual::Manual>()
            .add_message::<use_ability::UseSlot>()
            .add_systems(Startup, bar::spawn_bar)
            // Before widgets take presses on panels, and the map clicks.
            // Before the system menu sees Esc.
            .add_systems(
                PreUpdate,
                cancel::cancel_by_escape.before(WidgetSystems::Front),
            )
            .add_systems(
                PreUpdate,
                // Either side of `dom` moving what is dragged.
                (
                    drag::drag_abilities
                        .after(PointerSystems)
                        .before(WidgetSystems::Presses)
                        .before(HoldObjects),
                    drag::drop_abilities.after(HoldObjects),
                ),
            )
            // Once the page and the slots have drawn themselves.
            .add_systems(PostUpdate, mark::mark_abilities.after(DrawObjects))
            .add_systems(
                Update,
                (
                    page::fill_pages,
                    own_bar::take_bar,
                    held::use_by_key,
                    use_ability::take_timing,
                    sneak::take_sneaking,
                    // Moving first, then what was chosen by hand, then
                    // automatic abilities if nothing else is waiting.
                    cancel::cancel_by_moving,
                    manual::use_slots,
                    manual::ask_wanted,
                    use_ability::use_automatic,
                    (bar::place_bar, bar::show_bar),
                    wait::show_wait,
                )
                    .chain()
                    // Knowing whether the player is moving this frame.
                    .after(PlayerSystems::Travel),
            );
    }
}

#[cfg(test)]
mod tests;
