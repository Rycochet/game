//! The page providers send players back to (`CALLBACK_PATH`): the login is
//! finished here, on the network thread, and handed to the game
//! (`Logins::finished`), and the player is told to go back to it.

use axum::extract::RawQuery;
use axum::http::StatusCode;
use axum::response::Html;

use super::{Finished, Logins};

/// Answers a player coming back from a provider.
pub(super) async fn came_back(logins: Logins, query: RawQuery) -> (StatusCode, Html<String>) {
    let mut state = None;
    let mut code = None;
    let mut refused = None;
    for (key, value) in url::form_urlencoded::parse(query.0.unwrap_or_default().as_bytes()) {
        match &*key {
            "state" => state = Some(value.into_owned()),
            "code" => code = Some(value.into_owned()),
            "error" => refused = Some(value.into_owned()),
            _ => {}
        }
    }
    let Some(waiting) = state.as_deref().and_then(|state| logins.take(state)) else {
        return page(
            StatusCode::BAD_REQUEST,
            "This login has run out or was already used. Go back to the game to try again.",
        );
    };
    let provider = &logins.0.providers[waiting.provider];
    let email = match (code, refused) {
        (Some(code), None) => {
            let callback = &logins.0.callback;
            provider
                .email(&logins.0.http, callback, &code, &waiting.verifier)
                .await
        }
        (_, Some(refused)) => Err(format!("the provider said {refused:?}")),
        (None, None) => Err("the provider sent no code".into()),
    };
    let answer = match &email {
        Ok(_) => page(
            StatusCode::OK,
            "You're logged in. You can close this tab and go back to the game.",
        ),
        Err(reason) => page(
            StatusCode::BAD_REQUEST,
            &format!("Logging in didn't work: {reason}. Go back to the game to try again."),
        ),
    };
    logins.finish(Finished {
        state: state.expect("found by it"),
        provider: provider.settings.name.clone(),
        email,
    });
    answer
}

/// A plain page saying what happened.
fn page(status: StatusCode, said: &str) -> (StatusCode, Html<String>) {
    let said = said
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    (
        status,
        Html(format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta name=\"viewport\" content=\"width=device-width\">\
             <title>Hall of Worlds</title></head>\
             <body style=\"font-family: sans-serif; margin: 3em auto; max-width: 30em\">\
             <h1>Hall of Worlds</h1><p>{said}</p></body></html>"
        )),
    )
}
