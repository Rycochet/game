//! The Teleport to World panel (owner's design, 2026-11-09): a world's seed
//! (blank for the one the player is in, or in the Hall the one they came
//! from), a choice of its World Gates, and buttons going there, to the Hall of
//! Worlds beside that gate's way in, or showing that planet. It takes the place
//! of Go to World, Teleport to Gate and Teleport to Hall of Worlds.

use bevy::prelude::*;
use shared::world_gate::WORLD_GATES;

use crate::dom::{Dom, Template};
use crate::html;
use crate::widgets::UiId;

/// The panel's form, for the debug API.
pub(super) const FORM: &str = "debug.teleport_to_world";

/// What the panel has chosen: the gate (none: Any), and a seed to put in its
/// box once it is there (when View Planet opened it).
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct TeleportForm {
    pub(super) gate: Option<u8>,
    pub(super) seed_to_show: Option<String>,
}

/// The box with the world's seed.
#[derive(Component)]
pub(in crate::debug_menu) struct WorldSeedBox;

/// What the panel's other buttons do.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum TeleportButton {
    RandomSeed,
    /// A World Gate (from 0; it says one more), or none for Any.
    Gate(Option<u8>),
    ViewPlanet,
    Hall,
    Go,
}

/// The panel's contents, as markup. Each World Gate's button is a copy of
/// the template, numbered from 0 (it says one more).
pub(super) static TELEPORT_TO_WORLD: Template = html!(
    "debug_menu.teleport_to_world",
    r#"
<form id="debug.teleport_to_world">
  <panel id="panel" moveable>
    <span nowrap valign="middle">
      <label for="seed">Seed</label>
      <input id="seed" type="text" inputmode="numeric" width="240"/>
      <button id="new_seed" title="Random seed"><img src="picture:new_seed"/></button>
      Blank: this world
    </span>
    <span nowrap valign="middle">
      Gate
      <small id="gates" class="joined" nowrap>
        <button id="any">Any</button>
      </small>
    </span>
    <span nowrap>
      <button id="view_planet">View Planet...</button>
      <button id="hall">Hall of Worlds</button>
      <button id="go" class="main">Go</button>
    </span>
    <template id="gate"><button id="gate"/></template>
  </panel>
</form>
"#
);

/// What a built part of the panel is known by, from its id; the gates'
/// place is filled with a button for each.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId, dom: &mut Dom) {
    match (id.parts().1, id.index()) {
        ("seed", _) => bound.insert(WorldSeedBox),
        ("new_seed", _) => bound.insert(TeleportButton::RandomSeed),
        ("any", _) => bound.insert(TeleportButton::Gate(None)),
        ("gate", Some(gate)) => bound.insert(TeleportButton::Gate(Some(gate as u8))),
        ("view_planet", _) => bound.insert(TeleportButton::ViewPlanet),
        ("hall", _) => bound.insert(TeleportButton::Hall),
        ("go", _) => bound.insert(TeleportButton::Go),
        ("gates", _) => {
            for gate in 0..WORLD_GATES {
                dom.copy_template(
                    "debug.teleport_to_world/gate",
                    "debug.teleport_to_world/gates",
                    gate,
                );
                let words = (gate + 1).to_string();
                dom.set_text(&format!("debug.teleport_to_world/gate.{gate}"), words);
            }
            bound
        }
        _ => bound,
    };
}

/// The seed in the box: none when it is blank, and `Err` when it isn't a
/// number.
pub(super) fn seed_in(text: &str) -> Result<Option<u64>, ()> {
    let text = text.trim();
    if text.is_empty() {
        Ok(None)
    } else {
        text.parse().map(Some).map_err(|_| ())
    }
}
