//! Which providers the environment sets up: `HOW_OIDC_<NAME>_CLIENT_ID` and
//! `_CLIENT_SECRET` (or `_CLIENT_SECRET_FILE`) for each, with `_ISSUER` and
//! `_LABEL` (optional for the providers in `KNOWN`), and where they send players
//! back to (`HOW_LOGIN_URL`).

use url::Url;

use super::CALLBACK_PATH;
use crate::environment::secret_value;

/// Providers that need only a client id and secret: their name (as in the
/// variables, lower case), issuer and label.
const KNOWN: [(&str, &str, &str); 1] = [("google", "https://accounts.google.com", "Google")];

const PREFIX: &str = "HOW_OIDC_";
const CLIENT_ID: &str = "_CLIENT_ID";

/// A provider as the environment sets it up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProviderSettings {
    /// Lower case: what `LoginMethod::Provider` calls it.
    pub name: String,
    /// What players see it called.
    pub label: String,
    /// Where its OIDC discovery document is (`<issuer>/.well-known/...`).
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
}

/// The providers set up in `variables` (the environment's), by name, and a
/// warning for each left out and why.
pub(super) fn providers_from(
    variables: &[(String, String)],
) -> (Vec<ProviderSettings>, Vec<String>) {
    let value = |name: &str| {
        let found = variables.iter().find(|(key, _)| key == name);
        found.map(|(_, value)| value.clone())
    };
    let get = |name: &str| {
        value(name)
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    };
    let mut names: Vec<&str> = variables
        .iter()
        .filter_map(|(key, _)| key.strip_prefix(PREFIX)?.strip_suffix(CLIENT_ID))
        .filter(|name| !name.is_empty())
        .collect();
    names.sort_unstable();
    names.dedup();
    let mut providers = Vec::new();
    let mut warnings = Vec::new();
    for name in names {
        let variable = |part: &str| format!("{PREFIX}{name}_{part}");
        let lower = name.to_lowercase();
        let known = KNOWN.iter().find(|(known, ..)| *known == lower);
        let client_secret = match secret_value(&variable("CLIENT_SECRET"), value) {
            Ok(secret) => secret,
            Err(problem) => {
                warnings.push(format!("{problem}, so {lower} logins are off"));
                continue;
            }
        };
        let (Some(client_id), Some(client_secret)) = (get(&variable("CLIENT_ID")), client_secret)
        else {
            warnings.push(format!(
                "{} or {} (or {}) isn't set, so {lower} logins are off",
                variable("CLIENT_ID"),
                variable("CLIENT_SECRET"),
                variable("CLIENT_SECRET_FILE")
            ));
            continue;
        };
        let Some(issuer) =
            get(&variable("ISSUER")).or(known.map(|(_, issuer, _)| issuer.to_string()))
        else {
            warnings.push(format!(
                "{} isn't set, so {lower} logins are off",
                variable("ISSUER")
            ));
            continue;
        };
        if let Err(problem) = secure_enough(&issuer) {
            warnings.push(format!(
                "{}: {problem}, so {lower} logins are off",
                variable("ISSUER")
            ));
            continue;
        }
        let label = get(&variable("LABEL"))
            .or(known.map(|(.., label)| label.to_string()))
            .unwrap_or_else(|| capitalised(&lower));
        providers.push(ProviderSettings {
            name: lower,
            label,
            issuer,
            client_id,
            client_secret,
        });
    }
    (providers, warnings)
}

/// Where providers send players back to: `HOW_LOGIN_URL` (`login_url`), or the
/// Let's Encrypt domain's address, with `CALLBACK_PATH` on the end.
pub(super) fn callback_from(
    login_url: Option<&str>,
    lets_encrypt_domain: Option<&str>,
) -> Result<String, String> {
    let base = match (login_url, lets_encrypt_domain) {
        (Some(url), _) => url.trim().to_owned(),
        (None, Some(domain)) => format!("https://{}", domain.trim()),
        (None, None) => {
            return Err("HOW_LOGIN_URL isn't set, so providers can't send players back".into());
        }
    };
    secure_enough(&base).map_err(|problem| format!("HOW_LOGIN_URL: {problem}"))?;
    Ok(format!("{}{CALLBACK_PATH}", base.trim_end_matches('/')))
}

/// `Ok` for an `https` address, or a plain `http` one on this machine (for
/// trying a provider out locally); otherwise what is wrong with it.
pub(super) fn secure_enough(address: &str) -> Result<(), String> {
    let url =
        Url::parse(address).map_err(|error| format!("{address:?} isn't an address ({error})"))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    match url.scheme() {
        "https" => Ok(()),
        "http" if local => Ok(()),
        _ => Err(format!("{address:?} isn't https")),
    }
}

fn capitalised(name: &str) -> String {
    let mut letters = name.chars();
    match letters.next() {
        Some(first) => first.to_uppercase().chain(letters).collect(),
        None => String::new(),
    }
}
