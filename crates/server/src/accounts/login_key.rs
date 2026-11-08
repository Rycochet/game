//! A way of logging in, once proved: what `Accounts::log_in` finds an account
//! by.

use std::fmt;

use ring::digest::{SHA256, digest};
use shared::id::Id;
use shared::protocol::GuestToken;
use uuid::Uuid;

use super::{AccountId, hex};

/// Under which an email login's account id is made from its key, so that every
/// server sharing the login secret gives the same person the same id.
const ACCOUNT_ID_NAMESPACE: Uuid = Uuid::from_u128(0x3c8e_51d2_7a49_4f06_9b1e_d5a7_02c4_e18b);

/// An email address a provider vouched for, keyed with the login secret
/// (`oidc::LoginSecret::email_key`): it stands for the address without being
/// it, and can't be worked back without the secret.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmailKey(pub [u8; 32]);

/// By hand, so a key never reaches a log.
impl fmt::Debug for EmailKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EmailKey(hidden)")
    }
}

/// A way of logging in, proved by whoever makes one.
pub(crate) enum LoginKey {
    Guest(GuestToken),
    /// Through a provider's website (OIDC), by the address it vouched for.
    Email(EmailKey),
    /// The embedded server's one player, who has nothing to log in with.
    SinglePlayer,
    /// A way to an account that isn't a guest's, without a provider.
    #[cfg(test)]
    Test(&'static str),
}

impl LoginKey {
    pub(super) fn is_guest(&self) -> bool {
        matches!(self, Self::Guest(_) | Self::SinglePlayer)
    }

    /// The account it leads to, if that is decided by the key rather than made
    /// new: an email login's, the same on every server sharing the secret.
    pub(super) fn account_id(&self) -> Option<AccountId> {
        match self {
            Self::Email(key) => Some(AccountId(Id(Uuid::new_v5(&ACCOUNT_ID_NAMESPACE, &key.0)))),
            _ => None,
        }
    }

    /// Its file name. A method's name comes first, so the same key from another
    /// method is another login.
    pub(super) fn hash(&self) -> String {
        let mut text = match self {
            Self::Guest(_) => b"guest:".to_vec(),
            Self::Email(_) => b"email:".to_vec(),
            Self::SinglePlayer => b"single player:".to_vec(),
            #[cfg(test)]
            Self::Test(_) => b"test:".to_vec(),
        };
        match self {
            Self::Guest(token) => text.extend(token.0),
            Self::Email(key) => text.extend(key.0),
            Self::SinglePlayer => {}
            #[cfg(test)]
            Self::Test(key) => text.extend(key.as_bytes()),
        }
        hex(digest(&SHA256, &text).as_ref())
    }
}
