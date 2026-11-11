use super::*;
use crate::common::LoginProvider;

fn provider(name: &str, label: &str) -> LoginProvider {
    LoginProvider {
        name: name.into(),
        label: label.into(),
    }
}

#[test]
fn the_screen_is_good_markup() {
    LOGIN.assert_good();
}

#[test]
fn choosing_offers_each_provider_then_guest_and_cancel() {
    let choosing = LoggingIn::Choosing {
        providers: vec![provider("google", "Google")],
        guests: true,
        failed: None,
    };
    let (_, failed, providers, shown) = asked(&choosing).unwrap();
    assert!(!failed);
    assert_eq!(providers, ["Log in with Google"]);
    assert_eq!(shown, ["guest", "cancel"]);
    let first = UiId::new("login", "provider").at(0);
    let chosen = LoginChoice::Provider("google".into());
    assert_eq!(choice_of(&first, &choosing), Some(Ok(chosen)));
    // No such provider, and a button that isn't the screen's.
    assert_eq!(
        choice_of(&UiId::new("login", "provider").at(1), &choosing),
        None
    );
    assert_eq!(choice_of(&UiId::new("menu", "guest"), &choosing), None);
}

#[test]
fn a_failure_is_said_and_the_browser_step_offers_the_page_again() {
    let failed = LoggingIn::Choosing {
        providers: Vec::new(),
        guests: false,
        failed: Some("the page was closed".into()),
    };
    let (words, said_failed, _, shown) = asked(&failed).unwrap();
    assert!(said_failed && words.contains("the page was closed"));
    assert_eq!(shown, ["cancel"]);

    let waiting = LoggingIn::InBrowser {
        provider: provider("google", "Google"),
        url: None,
    };
    assert_eq!(asked(&waiting).unwrap().3, ["back"]);
    let open = LoggingIn::InBrowser {
        provider: provider("google", "Google"),
        url: Some("https://example.com/login".into()),
    };
    assert_eq!(asked(&open).unwrap().3, ["open_again", "back"]);
    let again = UiId::new("login", "open_again");
    let url = "https://example.com/login".to_owned();
    assert_eq!(choice_of(&again, &open), Some(Err(url)));
    assert_eq!(choice_of(&again, &waiting), None);
    assert_eq!(asked(&LoggingIn::Not), None);
}
