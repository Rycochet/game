//! Answering `ClientMessage::Role`: what the character is shown as, and the
//! Accounts panel's finding, giving, renaming and reading the audit log, each
//! checked against the asker's role. Then (`refresh_roles`) anyone playing on
//! an account whose role changed is told, or sent away if banned.

use bevy::log::info;
use shared::protocol::{CharacterId, FoundAccount, RoleAnswer, RoleAsk, ServerMessage};
use shared::role::{BANNED, Permission, RoleId};

use super::Client;
use crate::ServerWorlds;
use crate::accounts::{AccountId, Accounts};
use crate::roles::{Actor, Change};
use crate::storage::now;

/// Answers one; `false` if the client has gone.
pub(super) fn answer_role(client: &mut Client, ask: RoleAsk, accounts: &mut Accounts) -> bool {
    let answer = match ask {
        RoleAsk::ShowAs(id) => {
            if !client.show_as(id) {
                bevy::log::warn!("A client asked to be shown as a role it may not appear as");
            }
            Ok(RoleAnswer::Shown(client.shown()))
        }
        RoleAsk::LookUp(name) if client.may(Permission::LookUp) => look_up(client, &name, accounts),
        RoleAsk::Give {
            character,
            role,
            minutes,
            reason,
        } => give(client, character, role, minutes, reason, accounts),
        RoleAsk::Audit { skip } if client.may(Permission::ReadAudit) => accounts
            .roles
            .audit_page(skip as usize)
            .map(|(lines, more)| RoleAnswer::Audit { lines, more }),
        RoleAsk::Roles if client.may(Permission::RenameRoles) => {
            Ok(RoleAnswer::Roles(accounts.roles.badges()))
        }
        RoleAsk::Rename { role, name, title } if client.may(Permission::RenameRoles) => {
            let by = client
                .account
                .unwrap_or(AccountId(shared::id::Id(uuid::Uuid::nil())));
            let renamed = accounts
                .roles
                .rename(by, &role, &name, title.as_deref(), now());
            renamed.map(|()| RoleAnswer::Roles(accounts.roles.badges()))
        }
        _ => Err("your role doesn't allow that".into()),
    };
    let answer = answer.unwrap_or_else(RoleAnswer::Refused);
    client.link.send(&ServerMessage::Role(answer))
}

fn look_up(client: &Client, name: &str, accounts: &mut Accounts) -> Result<RoleAnswer, String> {
    let Some((character, account)) = accounts.find_character(name)? else {
        return Err(format!("nobody here is called {}", name.trim()));
    };
    found(client, character, account, accounts).map(RoleAnswer::Found)
}

/// What the panel shows of a character's account.
fn found(
    client: &Client,
    character: CharacterId,
    account: AccountId,
    accounts: &mut Accounts,
) -> Result<FoundAccount, String> {
    let save = accounts.account(&account)?.clone();
    let yours = client.account == Some(account);
    let mut held = accounts.roles.held(account, save.guest, now());
    // The embedded server's own player is its Owner, whatever the file says.
    if yours {
        held.badge = client.badge();
    }
    let powers = client.powers();
    let may_change =
        !save.guest && client.account != Some(account) && powers.grants.contains(&held.badge.id);
    let may_give = match may_change {
        true => powers
            .grants
            .iter()
            .filter_map(|id| accounts.roles.badge(id))
            .collect(),
        false => Vec::new(),
    };
    let name = |c: &shared::protocol::CharacterSummary| c.name.clone();
    Ok(FoundAccount {
        character,
        name: save
            .characters
            .iter()
            .find(|c| c.id == character)
            .map(name)
            .unwrap_or_default(),
        others: save
            .characters
            .iter()
            .filter(|c| c.id != character)
            .map(name)
            .collect(),
        guest: save.guest,
        yours,
        role: held.badge,
        until: held.until.map(crate::roles::log_stamp),
        reason: held.reason,
        may_give,
    })
}

fn give(
    client: &Client,
    character: CharacterId,
    role: RoleId,
    minutes: Option<u32>,
    reason: String,
    accounts: &mut Accounts,
) -> Result<RoleAnswer, String> {
    let Some(actor) = client.account else {
        return Err("not logged in".into());
    };
    let Some(target) = accounts.owner_of(&character)? else {
        return Err("that character has gone".into());
    };
    let guest = accounts.account(&target)?.guest;
    let powers = client.powers();
    let time = now();
    let change = Change {
        to: &role,
        until: minutes.map(|minutes| time + u64::from(minutes) * 60),
        reason: reason.trim(),
    };
    accounts
        .roles
        .set_role(Actor::Account(actor, &powers), target, guest, change, time)?;
    found(client, character, target, accounts).map(RoleAnswer::Found)
}

/// After every client's messages: anyone playing on an account whose role
/// changed takes the new one, keeping what they are shown as if they still
/// may, or is sent away if now banned.
pub(super) fn refresh_roles(
    clients: &mut [Client],
    accounts: &mut Accounts,
    worlds: &mut ServerWorlds,
) {
    for account in accounts.roles.take_changed() {
        for client in clients
            .iter_mut()
            .filter(|c| c.account == Some(account) && !c.gone)
        {
            let guest = accounts.account(&account).is_ok_and(|save| save.guest);
            let held = accounts.roles.held(account, guest, now());
            if held.badge.id.is(BANNED) {
                info!("Account {} was sent away: banned", account.0);
                client
                    .link
                    .send(&ServerMessage::Refused(super::login::banned(&held)));
                client.disconnect(worlds.of(client.world));
                continue;
            }
            let shown = client.shown().current;
            client.take_role(held, &accounts.roles);
            client.show_saved(Some(shown));
            client.unsaved = true;
            let changed = RoleAnswer::Changed {
                role: client.badge(),
                powers: client.powers(),
                shown: client.shown(),
            };
            client.link.send(&ServerMessage::Role(changed));
        }
    }
}
