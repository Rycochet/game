//! A provider ready to use: where its pages are (from its discovery document),
//! the address players are sent to, and the email address it vouches for.

use serde_json::Value;
use url::Url;

use super::config::{ProviderSettings, secure_enough};
use super::http::Http;

/// What a login asks the provider for: only who they are (`openid`) and their
/// email address.
const SCOPES: &str = "openid email";

/// A provider whose pages are known.
pub(super) struct Provider {
    pub settings: ProviderSettings,
    authorization: String,
    token: String,
    userinfo: String,
}

impl Provider {
    /// Reads the provider's discovery document, for where its pages are.
    pub(super) async fn discover(http: &Http, settings: ProviderSettings) -> Result<Self, String> {
        let issuer = settings.issuer.trim_end_matches('/');
        let found = http
            .get_json(&format!("{issuer}/.well-known/openid-configuration"), None)
            .await?;
        if found["issuer"]
            .as_str()
            .map(|said| said.trim_end_matches('/'))
            != Some(issuer)
        {
            return Err(format!(
                "its discovery document is for another issuer than {issuer}"
            ));
        }
        let page = |name: &str| {
            let page = found[name]
                .as_str()
                .ok_or_else(|| format!("its discovery document has no {name}"))?;
            secure_enough(page)?;
            Ok::<String, String>(page.to_owned())
        };
        Ok(Self {
            authorization: page("authorization_endpoint")?,
            token: page("token_endpoint")?,
            userinfo: page("userinfo_endpoint")?,
            settings,
        })
    }

    /// Where to send the player to log in: back to `callback` afterwards with
    /// `state`, proving it is this login with the PKCE `challenge`.
    pub(super) fn login_page(&self, callback: &str, state: &str, challenge: &str) -> String {
        let mut url = Url::parse(&self.authorization).expect("checked by `discover`");
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.settings.client_id)
            .append_pair("redirect_uri", callback)
            .append_pair("scope", SCOPES)
            .append_pair("state", state)
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256");
        url.into()
    }

    /// Swaps the code the player came back with for a token, then asks who it
    /// is for: the email address, if the provider has verified it. Nothing else
    /// is asked for or kept, and the token is forgotten straight away.
    pub(super) async fn email(
        &self,
        http: &Http,
        callback: &str,
        code: &str,
        verifier: &str,
    ) -> Result<String, String> {
        let tokens = http
            .post_form(
                &self.token,
                &[
                    ("grant_type", "authorization_code"),
                    ("code", code),
                    ("redirect_uri", callback),
                    ("client_id", &self.settings.client_id),
                    ("client_secret", &self.settings.client_secret),
                    ("code_verifier", verifier),
                ],
            )
            .await?;
        let token = tokens["access_token"]
            .as_str()
            .ok_or("the provider gave no access token")?;
        verified_email(&http.get_json(&self.userinfo, Some(token)).await?)
    }
}

/// The email address in a provider's user info, if it says it has verified it
/// (some say so as text). Anyone could claim an address that isn't verified.
pub(super) fn verified_email(info: &Value) -> Result<String, String> {
    let email = info["email"]
        .as_str()
        .filter(|email| email.contains('@'))
        .ok_or("the provider gave no email address")?;
    let verified = match &info["email_verified"] {
        Value::Bool(verified) => *verified,
        Value::String(verified) => verified == "true",
        _ => false,
    };
    if !verified {
        return Err("the provider hasn't verified this email address".into());
    }
    Ok(email.to_owned())
}
