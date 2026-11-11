//! Change User, at the bottom left: only while logged in to an account on the
//! server (`common::AccountEmail`; never as a guest or in single player). It
//! forgets that server's login and connects again, so the server asks how to
//! log in (`common::LoginChoice::ChangeUser`, `connection`).

use bevy::prelude::*;

use crate::common::AccountEmail;
use crate::dom::Dom;

/// Shows its row, under the grid of characters, while logged in to an account.
pub(super) fn show_change_user(
    email: Res<AccountEmail>,
    mut dom: Dom,
    mut shown: Local<Option<bool>>,
) {
    let logged_in = email.0.is_some();
    if *shown != Some(logged_in) {
        *shown = Some(logged_in);
        dom.set_flag("character_select/change_user_row", "hidden", !logged_in);
    }
}
