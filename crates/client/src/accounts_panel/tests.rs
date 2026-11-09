use shared::role::{Permission, Powers};

use super::state::{Tab, tabs_for};

#[test]
fn each_role_sees_the_tabs_it_may_use() {
    let with = |permissions: &[Permission]| Powers {
        permissions: permissions.iter().copied().collect(),
        ..Powers::default()
    };
    assert_eq!(tabs_for(&with(&[Permission::DebugTools])), []);
    assert_eq!(
        tabs_for(&with(&[Permission::LookUp, Permission::ReadAudit])),
        [Tab::Find, Tab::Audit]
    );
    assert_eq!(
        tabs_for(&with(&Permission::ALL)),
        [Tab::Find, Tab::Roles, Tab::Audit]
    );
}
