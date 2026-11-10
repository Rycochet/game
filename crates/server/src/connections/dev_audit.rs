//! Staff's use of debug tools goes in the audit log (owner, 2026-11-09), named
//! by account and character id only. Not the single player's own, and not
//! what only asks (`ClientMessage::audited`).

use shared::protocol::ClientMessage;

use super::Client;
use crate::ServerOptions;
use crate::accounts::Accounts;
use crate::storage::now;

/// Notes a debug tool this client may use, before it is used.
pub(super) fn note_use(
    client: &Client,
    message: &ClientMessage,
    accounts: &mut Accounts,
    options: &ServerOptions,
) {
    let Some(tool) = message.dev_tool() else {
        return;
    };
    if client.host || !message.audited() || !client.may_debug(tool, options) {
        return;
    }
    let account = client.account.map(|account| account.0);
    let what = format!(
        "Debug tool {tool:?} used by account {account:?}, character {:?}: {message:?}",
        client.character_id
    );
    accounts.roles.record(now(), &what);
}
