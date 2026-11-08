//! Logins waiting for their players to come back from a provider's website, by
//! the `state` sent there and back, each with its PKCE verifier.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ring::digest::{SHA256, digest};
use ring::rand::{SecureRandom, SystemRandom};

/// How long a player has to log in on the provider's website.
pub(super) const LOGIN_WITHIN: Duration = Duration::from_secs(10 * 60);

/// A login waiting for its player to come back.
pub(super) struct Waiting {
    /// Which provider (by its place in the list).
    pub provider: usize,
    /// The PKCE secret its challenge was made from, which only this server has.
    pub verifier: String,
    began: Instant,
}

/// The logins waiting, by state.
#[derive(Default)]
pub(super) struct Pending(HashMap<String, Waiting>);

impl Pending {
    /// Starts a login at `provider`: its state and PKCE challenge. Ones that
    /// ran out are forgotten. An error if secure randomness fails.
    pub(super) fn begin(&mut self, provider: usize) -> Result<(String, String), String> {
        let now = Instant::now();
        self.0
            .retain(|_, waiting| now.duration_since(waiting.began) < LOGIN_WITHIN);
        let state = random_text()?;
        let verifier = random_text()?;
        let challenge = URL_SAFE_NO_PAD.encode(digest(&SHA256, verifier.as_bytes()));
        self.0.insert(
            state.clone(),
            Waiting {
                provider,
                verifier,
                began: now,
            },
        );
        Ok((state, challenge))
    }

    /// The login with this state, if it is waiting and hasn't run out: only
    /// once, so a state can't be used again.
    pub(super) fn take(&mut self, state: &str) -> Option<Waiting> {
        self.0
            .remove(state)
            .filter(|waiting| waiting.began.elapsed() < LOGIN_WITHIN)
    }

    /// Forgets a login its player has given up on.
    pub(super) fn forget(&mut self, state: &str) {
        self.0.remove(state);
    }
}

/// 32 secure random bytes, as text fit for an address (43 characters).
fn random_text() -> Result<String, String> {
    let mut bytes = [0; 32];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| "the system can't provide secure random numbers".to_owned())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}
