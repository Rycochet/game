//! The Accounts panel, from the system menu's Accounts for roles with
//! something on it: finding a character by name and giving their account a
//! role (for good or a while, with a reason), renaming roles, and reading the
//! audit log. Also keeps the account's role (`common::AccountRole`). The
//! server checks everything asked; see "Roles" in
//! `docs/context/architecture.md`.

use bevy::prelude::*;

mod answers;
mod audit;
mod find;
mod input;
mod markup;
mod open;
mod roles;
mod show;
mod state;

pub struct AccountsPanelPlugin;

impl Plugin for AccountsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<state::Panel>().add_systems(
            Update,
            (
                answers::read_answers,
                open::notice_closed,
                open::open_panel,
                open::bind_panel,
                input::read_events,
                show::show_panel,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests;
