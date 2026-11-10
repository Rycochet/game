//! The roles every server starts with, and what each may do until the file
//! says otherwise.

use std::collections::BTreeSet;

use shared::role::{
    ADMIN, BANNED, DEVELOPER, DebugTool, GUEST, MODERATED, MODERATOR, OWNER, PLAYER, Permission,
    RoleId,
};

use super::file::RoleDef;

pub(super) fn default_roles() -> Vec<RoleDef> {
    use Permission::*;
    let all: Vec<Permission> = Permission::all().collect();
    let all = all.as_slice();
    // Developers have every debug tool but those for Admins; Moderators those
    // that help look after players (`DebugTool::for_moderators`).
    let developer = debug_tools(|tool| !tool.admin_only());
    let mut moderator = debug_tools(DebugTool::for_moderators);
    moderator.extend([LookUp, ReadAudit, PastPlayerLimit]);
    vec![
        // `fill_in` gives the Owner everything.
        role(OWNER, "Owner", all, &[], &[]),
        role(
            ADMIN,
            "Admin",
            all,
            &[DEVELOPER, MODERATOR, PLAYER, MODERATED, BANNED],
            &[DEVELOPER, MODERATOR, PLAYER],
        ),
        role(DEVELOPER, "Developer", &developer, &[], &[PLAYER]),
        role(
            MODERATOR,
            "Moderator",
            &moderator,
            &[PLAYER, MODERATED, BANNED],
            &[PLAYER],
        ),
        role(PLAYER, "Player", &[], &[], &[]),
        role(MODERATED, "Moderated", &[], &[], &[]),
        role(BANNED, "Banned", &[], &[], &[]),
        role(GUEST, "Guest", &[], &[], &[]),
    ]
}

/// The debug tools `which` picks, as permissions.
fn debug_tools(which: impl Fn(DebugTool) -> bool) -> Vec<Permission> {
    DebugTool::ALL
        .into_iter()
        .filter(|&tool| which(tool))
        .map(Permission::Debug)
        .collect()
}

fn role(
    id: &str,
    name: &str,
    permissions: &[Permission],
    grants: &[&str],
    appears_as: &[&str],
) -> RoleDef {
    let ids = |ids: &[&str]| {
        ids.iter()
            .map(|id| RoleId::new(id))
            .collect::<BTreeSet<_>>()
    };
    RoleDef {
        id: RoleId::new(id),
        name: name.into(),
        title: None,
        permissions: Permission::all()
            .map(|p| (p, permissions.contains(&p)))
            .collect(),
        grants: ids(grants),
        appears_as: ids(appears_as),
    }
}
