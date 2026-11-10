use std::fs;

use shared::id::Id;
use shared::role::*;
use uuid::Uuid;

use super::set_role::{Actor, Change};
use super::*;
use crate::storage::tests::TestFolder;

fn account(n: u128) -> AccountId {
    AccountId(Id(Uuid::from_u128(n)))
}

fn give(roles: &mut Roles, by: Actor, to: AccountId, role: &str) -> Result<(), String> {
    let role = RoleId::new(role);
    let change = Change {
        to: &role,
        until: None,
        reason: "",
    };
    roles.set_role(by, to, false, change, 1000)
}

#[test]
fn everyone_starts_a_player_and_guests_are_guests() {
    let mut roles = Roles::default();
    assert!(roles.held(account(1), false, 0).badge.id.is(PLAYER));
    assert!(roles.held(account(1), true, 0).badge.id.is(GUEST));
    assert_eq!(roles.held(account(1), false, 0).powers, Powers::default());
}

#[test]
fn the_owner_may_do_everything_and_give_any_role_but_owner() {
    let owner = Roles::default().owner();
    assert!(Permission::all().all(|p| owner.powers.may(p)));
    assert!(owner.powers.grants.contains(&RoleId::new(ADMIN)));
    assert!(!owner.powers.grants.contains(&RoleId::new(OWNER)));
    assert!(!owner.powers.grants.contains(&RoleId::new(GUEST)));
}

#[test]
fn developers_and_moderators_dont_overlap() {
    let mut roles = Roles::default();
    let owner = roles.owner().powers;
    give(
        &mut roles,
        Actor::Account(account(9), &owner),
        account(1),
        DEVELOPER,
    )
    .unwrap();
    give(
        &mut roles,
        Actor::Account(account(9), &owner),
        account(2),
        MODERATOR,
    )
    .unwrap();
    let developer = roles.held(account(1), false, 0).powers;
    let moderator = roles.held(account(2), false, 0).powers;
    let tool = |tool| Permission::Debug(tool);
    assert!(developer.may(tool(DebugTool::GiveItem)) && !developer.may(Permission::LookUp));
    // World Time and Freeze Mobs are for Admins.
    assert!(!developer.may(tool(DebugTool::WorldTime)));
    assert!(moderator.may(Permission::LookUp) && !moderator.may(tool(DebugTool::GiveItem)));
    assert!(moderator.may(tool(DebugTool::ExamineMob)) && moderator.may(tool(DebugTool::GodMode)));
}

#[test]
fn only_roles_in_grants_can_be_given_or_changed() {
    let mut roles = Roles::default();
    let owner = roles.owner().powers;
    let admin = roles.role(&RoleId::new(ADMIN), None, String::new()).powers;
    let moderator = roles
        .role(&RoleId::new(MODERATOR), None, String::new())
        .powers;
    give(
        &mut roles,
        Actor::Account(account(9), &owner),
        account(1),
        ADMIN,
    )
    .unwrap();
    // A moderator can moderate a player, but not an admin, nor make one.
    let by = |powers| Actor::Account(account(5), powers);
    assert!(give(&mut roles, by(&moderator), account(2), MODERATED).is_ok());
    assert!(give(&mut roles, by(&moderator), account(1), MODERATED).is_err());
    assert!(give(&mut roles, by(&moderator), account(3), ADMIN).is_err());
    // An admin can't make admins, nor change their own role.
    assert!(give(&mut roles, by(&admin), account(3), ADMIN).is_err());
    let own = Actor::Account(account(1), &admin);
    assert!(give(&mut roles, own, account(1), PLAYER).is_err());
    // Taking a role away is giving Player.
    assert!(give(&mut roles, by(&admin), account(2), PLAYER).is_ok());
    assert!(roles.held(account(2), false, 0).badge.id.is(PLAYER));
}

#[test]
fn guests_hold_no_roles_and_nobody_is_made_a_guest() {
    let mut roles = Roles::default();
    let owner = roles.owner().powers;
    let guest = RoleId::new(MODERATOR);
    let change = Change {
        to: &guest,
        until: None,
        reason: "",
    };
    let by = Actor::Account(account(9), &owner);
    assert!(roles.set_role(by, account(1), true, change, 0).is_err());
    let by = Actor::Account(account(9), &owner);
    assert!(give(&mut roles, by, account(1), GUEST).is_err());
}

#[test]
fn a_role_for_a_time_runs_out() {
    let mut roles = Roles::default();
    let banned = RoleId::new(BANNED);
    let change = Change {
        to: &banned,
        until: Some(500),
        reason: "spamming",
    };
    let owner = roles.owner().powers;
    let by = Actor::Account(account(9), &owner);
    roles.set_role(by, account(1), false, change, 100).unwrap();
    let held = roles.held(account(1), false, 499);
    assert!(held.badge.id.is(BANNED));
    assert_eq!((held.until, held.reason.as_str()), (Some(500), "spamming"));
    assert!(roles.held(account(1), false, 500).badge.id.is(PLAYER));
    let lines = roles.audit_lines();
    assert!(
        lines[0].contains("gave") && lines[0].contains("spamming"),
        "{lines:?}"
    );
    assert!(lines[1].contains("ran out"), "{lines:?}");
}

#[test]
fn the_claim_code_makes_an_account_the_owner_once() {
    let mut roles = Roles::default();
    let code = roles.start_claim().unwrap().unwrap();
    assert_eq!(code.len(), 24);
    assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(roles.try_claim(account(1), false, "Brin", 0), None);
    assert!(
        roles
            .try_claim(account(1), true, &code, 0)
            .unwrap()
            .is_err()
    );
    assert_eq!(roles.try_claim(account(1), false, &code, 0), Some(Ok(())));
    assert!(roles.held(account(1), false, 0).badge.id.is(OWNER));
    assert_eq!(roles.try_claim(account(2), false, &code, 0), None);
    assert_eq!(roles.start_claim(), Ok(None));
}

#[test]
fn the_file_is_made_kept_and_backed_up() {
    let folder = TestFolder::new();
    fs::create_dir_all(&folder.0).unwrap();
    let mut roles = Roles::open(&folder.0, 0).unwrap();
    assert!(folder.0.join("roles.ron").exists());
    let code = roles.start_claim().unwrap().unwrap();
    let claim_file = fs::read_to_string(folder.0.join("owner-claim.txt")).unwrap();
    assert!(claim_file.starts_with(&code));
    roles
        .try_claim(account(1), false, &code, 100)
        .unwrap()
        .unwrap();
    assert!(!folder.0.join("owner-claim.txt").exists());
    let audit = fs::read_to_string(folder.0.join("audit.log")).unwrap();
    assert!(audit.contains(&account(1).0.to_string()), "{audit}");

    let mut again = Roles::open(&folder.0, 200).unwrap();
    assert!(again.held(account(1), false, 0).badge.id.is(OWNER));
    let owner = again.owner().powers;
    for n in 2..20 {
        let by = Actor::Account(account(1), &owner);
        give(&mut again, by, account(n), MODERATOR).unwrap();
    }
    let backups = fs::read_dir(folder.0.join("roles-backups"))
        .unwrap()
        .count();
    assert_eq!(backups, disk::BACKUPS);
}

#[test]
fn a_hand_edited_file_is_filled_in_and_checked() {
    let folder = TestFolder::new();
    fs::create_dir_all(&folder.0).unwrap();
    let written = r#"(
        format: 1,
        roles: [
            (id: "developer", name: "Coder", title: Some("Wizard"), permissions: {DebugTools: false}),
            (id: "game-master", name: "Game Master", permissions: {LookUp: true}, appears_as: ["player"]),
        ],
        accounts: {
            "00000000-0000-0000-0000-000000000007": (role: "game-master", since: 0),
        },
    )"#;
    fs::write(folder.0.join("roles.ron"), written).unwrap();
    let mut roles = Roles::open(&folder.0, 0).unwrap();
    let held = roles.held(account(7), false, 0);
    assert_eq!(held.badge.shown(), "Game Master");
    assert_eq!(held.powers.permissions, [Permission::LookUp].into());
    // What the file says wins over the defaults; what it leaves out is filled in.
    let developer = roles.role(&RoleId::new(DEVELOPER), None, String::new());
    assert_eq!(developer.badge.shown(), "Wizard");
    // The old single debug tools permission, off where it was on: none of them.
    assert!(!developer.powers.may(Permission::Debug(DebugTool::GiveItem)));
    assert!(!developer.powers.may(Permission::DebugTools));
    assert!(roles.file.role(&RoleId::new(ADMIN)).is_some());
    let rewritten = fs::read_to_string(folder.0.join("roles.ron")).unwrap();
    assert!(rewritten.contains("game-master") && rewritten.contains("admin"));
}

#[test]
fn a_broken_file_stops_the_server_and_is_left_alone() {
    let folder = TestFolder::new();
    fs::create_dir_all(&folder.0).unwrap();
    let broken = [
        "not a roles file",
        r#"(format: 1, roles: [(id: "Bad Id", name: "x")])"#,
        r#"(format: 1, roles: [], accounts: {"nobody": (role: "admin", since: 0)})"#,
        r#"(format: 1, roles: [], accounts: {"00000000-0000-0000-0000-000000000007": (role: "player", since: 0)})"#,
        r#"(format: 1, roles: [(id: "x", name: "X", grants: ["owner"])])"#,
    ];
    for text in broken {
        fs::write(folder.0.join("roles.ron"), text).unwrap();
        assert!(Roles::open(&folder.0, 0).is_err(), "{text}");
        assert_eq!(
            fs::read_to_string(folder.0.join("roles.ron")).unwrap(),
            text
        );
    }
}

#[test]
fn old_debug_tools_permissions_become_each_tool() {
    let folder = TestFolder::new();
    fs::create_dir_all(&folder.0).unwrap();
    let written = r#"(
        format: 1,
        roles: [
            (id: "moderator", name: "Moderator", permissions: {DebugTools: false, LookUp: true}),
            (id: "builder", name: "Builder", permissions: {DebugTools: true}),
        ],
        accounts: {},
    )"#;
    fs::write(folder.0.join("roles.ron"), written).unwrap();
    let roles = Roles::open(&folder.0, 0).unwrap();
    let tool = |tool| Permission::Debug(tool);
    // As it had by default: the new defaults.
    let moderator = roles
        .role(&RoleId::new(MODERATOR), None, String::new())
        .powers;
    assert!(moderator.may(tool(DebugTool::ExamineMob)));
    // A server's own role had them all: every one but the Admins'.
    let builder = roles
        .role(&RoleId::new("builder"), None, String::new())
        .powers;
    assert!(builder.may(tool(DebugTool::GiveItem)) && !builder.may(tool(DebugTool::WorldTime)));
    let rewritten = fs::read_to_string(folder.0.join("roles.ron")).unwrap();
    assert!(!rewritten.contains("DebugTools") && rewritten.contains("Debug(GiveItem)"));
}
