//! Showing the login screen as it would be at each step, without logging in.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params};

use super::done;
use crate::common::{LoggingIn, LoginProvider};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LoginShown {
    Closed,
    /// Two providers and guests to choose from.
    Choosing,
    /// The same, after a try that failed.
    Failed,
    /// Waiting for the provider's page to be sent.
    Opening,
    /// Waiting while the player logs in on the provider's page.
    Browser,
}

#[derive(Deserialize)]
pub(super) struct LoginParams {
    show: LoginShown,
}

/// `client.login_screen { "show": "choosing" }`: `closed`, `choosing`,
/// `failed`, `opening` or `browser`. Only what the screen shows: nothing is
/// asked of a server, and its buttons' choices go nowhere.
pub(super) fn login_screen(
    In(request): In<Option<Value>>,
    mut logging_in: ResMut<LoggingIn>,
) -> BrpResult {
    let LoginParams { show } = params(request)?;
    let provider = |name: &str, label: &str| LoginProvider {
        name: name.to_owned(),
        label: label.to_owned(),
    };
    let choosing = |failed: Option<&str>| LoggingIn::Choosing {
        providers: vec![provider("first", "First"), provider("second", "Second")],
        guests: true,
        failed: failed.map(str::to_owned),
    };
    let in_browser = |url: Option<&str>| LoggingIn::InBrowser {
        provider: provider("first", "First"),
        url: url.map(str::to_owned),
    };
    *logging_in = match show {
        LoginShown::Closed => LoggingIn::Not,
        LoginShown::Choosing => choosing(None),
        LoginShown::Failed => choosing(Some("the page was closed")),
        LoginShown::Opening => in_browser(None),
        LoginShown::Browser => in_browser(Some("https://example.invalid/login")),
    };
    done()
}
