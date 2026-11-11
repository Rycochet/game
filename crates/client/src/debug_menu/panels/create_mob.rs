//! The Create Mob panel (owner's design): what the mob will be, and Create, a
//! toggle that turns Mob Mode on: clicks then put that mob on the cell clicked.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::sheet::Feeling;

use crate::dom::Template;
use crate::html;
use crate::widgets::UiId;

/// The attitudes to choose from, by the dropdown's order: the kind's own first.
pub(super) const ATTITUDES: [(&str, Option<Feeling>); 4] = [
    ("Default", None),
    ("Friendly", Some(Feeling::Friendly)),
    ("Neutral", Some(Feeling::Neutral)),
    ("Unfriendly", Some(Feeling::Unfriendly)),
];

/// Which of the panel's dropdowns it is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum MobChoice {
    Kind,
    Attitude,
    Gear,
}

#[derive(Component)]
pub(in crate::debug_menu) struct MobStill;

#[derive(Component)]
pub(in crate::debug_menu) struct MobSeed;

#[derive(Component)]
pub(in crate::debug_menu) struct MobNewSeed;

/// Create: on while Mob Mode is.
#[derive(Component)]
pub(in crate::debug_menu) struct MobCreate;

/// "Click to place", shown while Mob Mode is on.
#[derive(Component)]
pub(in crate::debug_menu) struct MobClickToPlace;

/// The panel's contents, as markup; `start_create_mob_panel` gives the
/// dropdowns their choices and sets it all to `common::CreateMobOrder`.
pub(super) static CREATE_MOB: Template = html!(
    "debug_menu.create_mob",
    r#"
<form id="debug.create_mob">
  <span nowrap valign="middle">
    <label for="kind">Kind</label>
    <select id="kind" searchable width="150"><option>Human</option></select>
    <label for="attitude">Attitude</label>
    <select id="attitude" width="130"><option>Default</option></select>
    <label for="gear">Gear</label>
    <select id="gear" width="100"><option>Rolled</option></select>
    <input id="still" type="checkbox" label="Still" readonly/>
  </span>
  <span nowrap valign="middle">
    <label for="seed">Seed</label>
    <input id="seed" type="text" inputmode="numeric" width="240"/>
    <button id="new_seed" title="Random seed"><img src="picture:new_seed"/></button>
  </span>
  <span nowrap align="right" valign="middle">
    <p id="click_to_place" class="warning">Click to place</p>
    <button id="create">Create</button>
  </span>
</form>
"#
);

impl MobChoice {
    pub(super) const ALL: [Self; 3] = [Self::Kind, Self::Attitude, Self::Gear];

    /// Its dropdown's name in the markup.
    pub(super) fn ui_name(self) -> &'static str {
        match self {
            Self::Kind => "kind",
            Self::Attitude => "attitude",
            Self::Gear => "gear",
        }
    }

    /// What it offers, in order.
    pub(super) fn options(self) -> Vec<String> {
        match self {
            Self::Kind => CreatureKind::ALL
                .iter()
                .map(|kind| kind.def().name.to_owned())
                .collect(),
            Self::Attitude => ATTITUDES
                .iter()
                .map(|(name, _)| (*name).to_owned())
                .collect(),
            Self::Gear => vec!["Rolled".to_owned(), "None".to_owned()],
        }
    }
}

/// What a built part of the panel is known by, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    let name = id.parts().1;
    if let Some(choice) = MobChoice::ALL.into_iter().find(|it| it.ui_name() == name) {
        bound.insert(choice);
        return;
    }
    match name {
        "still" => bound.insert(MobStill),
        "seed" => bound.insert(MobSeed),
        "new_seed" => bound.insert(MobNewSeed),
        "create" => bound.insert(MobCreate),
        // Shown only while Mob Mode is on.
        "click_to_place" => bound.insert((MobClickToPlace, Visibility::Hidden)),
        _ => bound,
    };
}
