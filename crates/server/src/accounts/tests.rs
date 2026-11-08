use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use super::*;
use crate::save_queue::tests::MemoryBackend;
use crate::storage::tests::TestFolder;
use shared::protocol::SessionToken;

/// Every file under a folder, as text: its path and contents.
fn everything_in(folder: &Path) -> String {
    let mut text = String::new();
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        text += &path.display().to_string();
        if path.is_dir() {
            text += &everything_in(&path);
        } else {
            text += &fs::read_to_string(&path).unwrap();
        }
    }
    text
}

#[test]
fn a_login_finds_its_account_again_after_a_restart() {
    let folder = TestFolder::new();
    let token = LoginKey::Guest(GuestToken([3; 32]));
    let mut accounts = Accounts::new(SaveQueue::in_folder(&folder.0));
    let id = accounts.log_in(&token).unwrap();
    let character = accounts.guest_character(&id).unwrap();
    assert!(accounts.account(&id).unwrap().guest);
    // Dropping it finishes writing, as stopping does.
    drop(accounts);

    let mut restarted = Accounts::new(SaveQueue::in_folder(&folder.0));
    assert_eq!(restarted.log_in(&token).unwrap(), id);
    assert_eq!(restarted.guest_character(&id).unwrap(), character);
    assert_eq!(restarted.account(&id).unwrap().characters.len(), 1);
    // Someone else is someone else.
    let other = LoginKey::Guest(GuestToken([4; 32]));
    assert_ne!(restarted.log_in(&other).unwrap(), id);
}

#[test]
fn a_guest_token_is_never_saved() {
    let folder = TestFolder::new();
    let token = GuestToken([0xcd; 32]);
    let mut accounts = Accounts::new(SaveQueue::in_folder(&folder.0));
    let id = accounts.log_in(&LoginKey::Guest(token)).unwrap();
    accounts.guest_character(&id).unwrap();
    drop(accounts);
    assert!(!everything_in(&folder.0).contains(&token.to_hex()));
}

#[test]
fn the_single_player_keeps_their_character_after_a_restart() {
    let folder = TestFolder::new();
    let mut accounts = Accounts::new(SaveQueue::in_folder(&folder.0));
    let character = accounts.single_player().unwrap();
    drop(accounts);
    let mut restarted = Accounts::new(SaveQueue::in_folder(&folder.0));
    assert_eq!(restarted.single_player().unwrap(), character);
    assert_eq!(character.0.0.get_version_num(), 6);
}

#[test]
fn the_same_key_from_another_method_is_another_login() {
    let guest = LoginKey::Guest(GuestToken([0; 32])).hash();
    let test = LoginKey::Test("").hash();
    assert_ne!(guest, test);
}

#[test]
fn names_are_checked_and_characters_limited() {
    let mut accounts = Accounts::default();
    let id = accounts.log_in(&LoginKey::Test("someone")).unwrap();
    assert!(!accounts.account(&id).unwrap().guest);
    for bad in ["", "   ", "a\nb", &"x".repeat(MAX_NAME + 1)] {
        assert!(accounts.create_character(&id, bad).is_err(), "{bad}");
    }
    for _ in 0..MAX_CHARACTERS {
        accounts
            .create_character(&id, " Long Name Ünder 24 ")
            .unwrap();
    }
    assert_eq!(
        accounts.account(&id).unwrap().characters[0].name,
        "Long Name Ünder 24"
    );
    assert!(accounts.create_character(&id, "One More").is_err());
}

#[test]
fn a_character_is_found_by_name_for_others_to_see() {
    let mut accounts = Accounts::default();
    let id = accounts.log_in(&LoginKey::Test("someone")).unwrap();
    let character = accounts.create_character(&id, "Brin").unwrap();
    assert_eq!(accounts.character_name(&character).as_deref(), Some("Brin"));
    accounts.rename_character(&character, "Bo").unwrap();
    assert_eq!(accounts.character_name(&character).as_deref(), Some("Bo"));
}

#[test]
fn saves_that_cant_be_reached_dont_make_a_new_account() {
    let backend = Arc::new(MemoryBackend::default());
    let queue = SaveQueue::new(backend.clone());
    let token = LoginKey::Guest(GuestToken([3; 32]));
    let id = Accounts::new(queue.clone()).log_in(&token).unwrap();
    assert!(queue.flush(Duration::from_secs(5)));

    backend.cant_read.store(true, Ordering::Relaxed);
    let mut restarted = Accounts::new(queue.clone());
    assert!(restarted.log_in(&token).is_err());
    assert!(restarted.account(&id).is_err());

    backend.cant_read.store(false, Ordering::Relaxed);
    assert_eq!(restarted.log_in(&token).unwrap(), id);
}

#[test]
fn an_email_login_has_the_same_account_wherever_the_secret_is_shared() {
    let key = LoginKey::Email(EmailKey([7; 32]));
    let id = Accounts::default().log_in(&key).unwrap();
    assert_eq!(Accounts::default().log_in(&key).unwrap(), id);
    let other = Accounts::default().log_in(&LoginKey::Email(EmailKey([8; 32])));
    assert_ne!(other.unwrap(), id);
    let mut accounts = Accounts::default();
    accounts.log_in(&key).unwrap();
    assert!(!accounts.is_guest(&id).unwrap());
}

#[test]
fn a_session_stands_for_its_account_until_it_runs_out_and_is_never_saved() {
    let folder = TestFolder::new();
    let queue = SaveQueue::in_folder(&folder.0);
    let mut accounts = Accounts::new(queue.clone());
    let id = accounts.log_in(&LoginKey::Test("someone")).unwrap();
    let token = accounts.new_session(id).unwrap();
    assert_eq!(accounts.session(&token).unwrap(), Some(id));
    assert_eq!(accounts.session(&SessionToken([1; 32])).unwrap(), None);
    assert!(queue.flush(Duration::from_secs(5)));
    assert!(!everything_in(&folder.0).contains(&token.to_hex()));

    // After a restart, until it runs out.
    let mut restarted = Accounts::new(queue.clone());
    assert_eq!(restarted.session(&token).unwrap(), Some(id));
    let hash = session_hash(&token);
    let ran_out = LoginSave {
        account: id,
        expires: Some(1),
    };
    queue.save(SaveKey::Login(hash.clone()), Save::Login(ran_out));
    assert!(queue.flush(Duration::from_secs(5)));
    let mut restarted = Accounts::new(queue.clone());
    assert_eq!(restarted.session(&token).unwrap(), None);
    assert!(queue.flush(Duration::from_secs(5)));
    assert!(queue.load(&SaveKey::Login(hash)).unwrap().is_none());
}
