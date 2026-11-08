use super::store::{read_from, write_to};
use super::*;

#[test]
fn a_login_is_kept_and_forgotten_per_server() {
    let folder =
        std::env::temp_dir().join(format!("hall-of-worlds-login-test-{}", std::process::id()));
    let file = folder.join("logins.ron");
    let login = SavedLogin {
        session: SessionToken([5; 32]),
        email: "someone@example.com".into(),
    };
    write_to(&file, "a:1", Some(Kept::from(&login))).unwrap();
    write_to(&file, "b:1", Some(Kept::from(&login))).unwrap();
    let all = read_from(&file).unwrap();
    assert!(all["a:1"].clone().login() == Some(login.clone()));
    write_to(&file, "a:1", None).unwrap();
    let all = read_from(&file).unwrap();
    assert!(!all.contains_key("a:1") && all.contains_key("b:1"));

    // A file that can't be read isn't overwritten.
    std::fs::write(&file, "not logins").unwrap();
    assert!(write_to(&file, "a:1", Some(Kept::from(&login))).is_err());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "not logins");
    let _ = std::fs::remove_dir_all(&folder);
}
