//! The Roles page: a role's name and the title shown in its place, if any.

use super::show::numbered;
use super::state::Panel;
use crate::dom::Dom;

pub(super) fn show_roles(panel: &mut Panel, dom: &mut Dom) {
    let known = !panel.roles.is_empty();
    dom.set_flag("accounts/asking", "hidden", known);
    dom.set_flag("accounts/roles", "hidden", !known);
    if known && panel.new_roles {
        let ids = panel.roles.iter().map(|badge| badge.id.0.as_str());
        dom.set_options("accounts/role", numbered(ids));
        dom.set("accounts/role", "value", &panel.role.to_string());
    }
    // Only when a role's words were taken: what is typed since is the boxes'
    // own.
    if panel.new_words {
        dom.set("accounts/role_name", "value", &panel.role_name);
        dom.set("accounts/role_title", "value", &panel.role_title);
    }
    panel.new_roles = false;
    panel.new_words = false;
}
