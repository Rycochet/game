use shared::role::{Permission, Powers};

use super::markup::ACCOUNTS;
use super::show::capitalised;
use super::state::{FOR, Tab, tabs_for};

#[test]
fn the_panel_is_good_markup_with_every_tab_and_time() {
    let source = ACCOUNTS.source;
    ACCOUNTS.assert_good();
    for tab in Tab::ALL {
        let option = format!("<option>{}</option>", tab.label());
        assert!(source.contains(&option));
        let (_, page) = tab.page().split_once('/').unwrap();
        assert!(source.contains(&format!("<div id=\"{page}\"")));
    }
    // The times a role is given for, in order.
    let mut at = 0;
    for (words, _) in FOR {
        let option = format!("<option>{words}</option>");
        at += source[at..].find(&option).expect("in order");
    }
}

#[test]
fn a_reason_starts_with_a_capital() {
    assert_eq!(capitalised("no such character"), "No such character");
    assert_eq!(capitalised(""), "");
}

#[test]
fn each_role_sees_the_tabs_it_may_use() {
    let with = |permissions: &[Permission]| Powers {
        permissions: permissions.iter().copied().collect(),
        ..Powers::default()
    };
    assert_eq!(
        tabs_for(&with(&[Permission::Debug(
            shared::role::DebugTool::GiveItem
        )])),
        []
    );
    assert_eq!(
        tabs_for(&with(&[Permission::LookUp, Permission::ReadAudit])),
        [Tab::Find, Tab::Audit]
    );
    assert_eq!(
        tabs_for(&with(&Permission::all().collect::<Vec<_>>())),
        [Tab::Find, Tab::Roles, Tab::Audit]
    );
}
