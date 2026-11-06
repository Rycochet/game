use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use super::*;
use crate::save_queue::tests::MemoryBackend;
use crate::storage::tests::TestFolder;

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
