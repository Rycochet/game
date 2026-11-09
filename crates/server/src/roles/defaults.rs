//! The roles every server starts with, and what each may do until the file
//! says otherwise.

use std::collections::BTreeSet;

use shared::role::{
    ADMIN, BANNED, DEVELOPER, GUEST, MODERATED, MODERATOR, OWNER, PLAYER, Permission, RoleId,
};

use super::file::RoleDef;

pub(super) fn default_roles() -> Vec<RoleDef> {
    use Permission::*;
    let all = Permission::ALL.as_slice();
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
        role(DEVELOPER, "Developer", &[DebugTools], &[], &[PLAYER]),
        role(
            MODERATOR,
            "Moderator",
            &[LookUp, ReadAudit, PastPlayerLimit],
            &[PLAYER, MODERATED, BANNED],
            &[PLAYER],
        ),
        role(PLAYER, "Player", &[], &[], &[]),
        role(MODERATED, "Moderated", &[], &[], &[]),
        role(BANNED, "Banned", &[], &[], &[]),
        role(GUEST, "Guest", &[], &[], &[]),
    ]
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
        permissions: Permission::ALL
            .into_iter()
            .map(|p| (p, permissions.contains(&p)))
            .collect(),
        grants: ids(grants),
        appears_as: ids(appears_as),
    }
}
