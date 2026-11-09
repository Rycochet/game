//! The Accounts panel's requests, and role changes reaching those playing.

use shared::protocol::{ClientLink, RoleAnswer, RoleAsk, ServerMessage, local_pair};
use shared::role::{ADMIN, BANNED, DEVELOPER, MODERATED, MODERATOR, PLAYER, RoleId};
use shared::world::WorldSeed;

use super::role_ask::{answer_role, refresh_roles};
use super::*;
use crate::accounts::{AccountId, Accounts, LoginKey};
use crate::roles::{Actor, Change};
use crate::storage::now;
use crate::{ServerWorld, ServerWorlds};

/// Accounts with a player, Brin, and a client logged in to an account holding `role`.
fn staff(role: &str) -> (Accounts, AccountId, Client, ClientLink) {
    let mut accounts = Accounts::default();
    let player = accounts.log_in(&LoginKey::Test("player")).unwrap();
    accounts.create_character(&player, "Brin").unwrap();
    let staff = accounts.log_in(&LoginKey::Test("staff")).unwrap();
    give(&mut accounts, staff, role);
    let (end, link) = local_pair();
    let mut client = Client::new(link, None);
    client.account = Some(staff);
    let held = accounts.roles.held(staff, false, now());
    client.take_role(held, &accounts.roles);
    (accounts, player, client, end)
}

fn give(accounts: &mut Accounts, to: AccountId, role: &str) {
    let role = RoleId::new(role);
    let change = Change {
        to: &role,
        until: None,
        reason: "",
    };
    accounts
        .roles
        .set_role(Actor::Server, to, false, change, now())
        .unwrap();
}

fn ask(client: &mut Client, accounts: &mut Accounts, end: &ClientLink, ask: RoleAsk) -> RoleAnswer {
    answer_role(client, ask, accounts);
    match end.receive().unwrap() {
        Some(ServerMessage::Role(answer)) => answer,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_moderator_finds_a_player_and_bans_them_for_a_while() {
    let (mut accounts, _, mut client, end) = staff(MODERATOR);
    let RoleAnswer::Found(found) = ask(
        &mut client,
        &mut accounts,
        &end,
        RoleAsk::LookUp("BRIN".into()),
    ) else {
        panic!("not found");
    };
    assert_eq!(
        (found.name.as_str(), found.role.id.0.as_str()),
        ("Brin", PLAYER)
    );
    let may: Vec<_> = found.may_give.iter().map(|b| b.id.0.as_str()).collect();
    assert_eq!(may, [BANNED, MODERATED, PLAYER]);

    let banning = RoleAsk::Give {
        character: found.character,
        role: RoleId::new(BANNED),
        minutes: Some(60),
        reason: "spam".into(),
    };
    let RoleAnswer::Found(after) = ask(&mut client, &mut accounts, &end, banning) else {
        panic!("not banned");
    };
    assert!(
        after.role.id.is(BANNED) && after.until.is_some(),
        "{after:?}"
    );
    assert_eq!(after.reason, "spam");
    // Not a role a moderator can give.
    let promoting = RoleAsk::Give {
        character: found.character,
        role: RoleId::new(ADMIN),
        minutes: None,
        reason: String::new(),
    };
    assert!(matches!(
        ask(&mut client, &mut accounts, &end, promoting),
        RoleAnswer::Refused(_)
    ));
    let RoleAnswer::Audit { lines, more } =
        ask(&mut client, &mut accounts, &end, RoleAsk::Audit { skip: 0 })
    else {
        panic!("no audit");
    };
    assert!(
        lines[0].contains("banned") && lines[0].contains("spam") && !more,
        "{lines:?}"
    );
}

#[test]
fn what_a_role_may_not_do_is_refused() {
    let (mut accounts, _, mut client, end) = staff(DEVELOPER);
    for refused in [
        RoleAsk::LookUp("Brin".into()),
        RoleAsk::Audit { skip: 0 },
        RoleAsk::Roles,
    ] {
        let answer = ask(&mut client, &mut accounts, &end, refused);
        assert!(matches!(answer, RoleAnswer::Refused(_)), "{answer:?}");
    }
    let unknown = ask(
        &mut client,
        &mut accounts,
        &end,
        RoleAsk::LookUp("Nobody".into()),
    );
    assert!(matches!(unknown, RoleAnswer::Refused(_)), "{unknown:?}");
}

#[test]
fn an_admin_renames_a_role() {
    let (mut accounts, _, mut client, end) = staff(ADMIN);
    let rename = RoleAsk::Rename {
        role: RoleId::new(MODERATOR),
        name: "Warden".into(),
        title: Some("Keeper of the Peace".into()),
    };
    let RoleAnswer::Roles(roles) = ask(&mut client, &mut accounts, &end, rename) else {
        panic!("not renamed");
    };
    let moderator = roles.iter().find(|b| b.id.is(MODERATOR)).unwrap();
    assert_eq!(moderator.shown(), "Keeper of the Peace");
    let too_long = RoleAsk::Rename {
        role: RoleId::new(MODERATOR),
        name: "x".repeat(25),
        title: None,
    };
    assert!(matches!(
        ask(&mut client, &mut accounts, &end, too_long),
        RoleAnswer::Refused(_)
    ));
}

#[test]
fn a_change_reaches_the_player_and_a_ban_sends_them_away() {
    let mut accounts = Accounts::default();
    let player = accounts.log_in(&LoginKey::Test("player")).unwrap();
    let mut worlds = ServerWorlds::with_home(ServerWorld::new(WorldSeed::new(1)));
    let (end, link) = local_pair();
    let mut clients = vec![Client::new(link, None)];
    clients[0].host = false;
    clients[0].account = Some(player);

    give(&mut accounts, player, MODERATOR);
    refresh_roles(&mut clients, &mut accounts, &mut worlds);
    let told = end.receive().unwrap();
    assert!(
        matches!(&told, Some(ServerMessage::Role(RoleAnswer::Changed { role, .. })) if role.id.is(MODERATOR)),
        "{told:?}"
    );
    assert!(clients[0].may(shared::role::Permission::LookUp));

    give(&mut accounts, player, BANNED);
    refresh_roles(&mut clients, &mut accounts, &mut worlds);
    assert!(matches!(
        end.receive().unwrap(),
        Some(ServerMessage::Refused(_))
    ));
    assert!(clients[0].is_gone());
}
