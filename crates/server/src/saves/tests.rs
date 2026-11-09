use super::*;
use crate::storage::tests::TestFolder;
use crate::storage::{OLDEST_SAVE_FORMAT, SAVE_FORMAT};

/// A data folder as an older server left it, with some certificates.
fn old_folder() -> TestFolder {
    let folder = TestFolder::new();
    for file in [
        "accounts/a.ron",
        "characters/player.ron",
        "logins/l.ron",
        "worlds/00112233/world.ron",
        "tls/key.pem",
    ] {
        let file = folder.0.join(file);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, "kept").unwrap();
    }
    folder
}

fn set_aside_folders(data: &Path) -> Vec<PathBuf> {
    fs::read_dir(data)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.starts_with("old-saves-")
        })
        .collect()
}

#[test]
fn old_saves_are_moved_aside_with_nothing_lost_and_certificates_left() {
    let folder = old_folder();
    set_aside_old_saves(&folder.0).unwrap();
    let aside = set_aside_folders(&folder.0);
    assert_eq!(aside.len(), 1);
    for file in [
        "accounts/a.ron",
        "characters/player.ron",
        "logins/l.ron",
        "worlds/00112233/world.ron",
    ] {
        assert_eq!(fs::read_to_string(aside[0].join(file)).unwrap(), "kept");
        assert!(!folder.0.join(file).exists(), "{file}");
    }
    assert!(folder.0.join("tls/key.pem").exists());
    assert!(!aside[0].join("tls").exists());
}

#[test]
fn saves_in_the_current_format_stay() {
    let folder = old_folder();
    set_aside_old_saves(&folder.0).unwrap();
    fs::create_dir_all(folder.0.join("characters")).unwrap();
    set_aside_old_saves(&folder.0).unwrap();
    assert_eq!(set_aside_folders(&folder.0).len(), 1);
    assert!(folder.0.join("characters").exists());
}

#[test]
fn an_older_format_is_moved_aside_too_and_each_wipe_gets_its_own_folder() {
    let folder = old_folder();
    set_aside_old_saves(&folder.0).unwrap();
    fs::create_dir_all(folder.0.join("worlds")).unwrap();
    let older = format!("(format: {})", OLDEST_SAVE_FORMAT - 1);
    fs::write(folder.0.join(FORMAT_FILE), older).unwrap();
    set_aside_old_saves(&folder.0).unwrap();
    assert_eq!(set_aside_folders(&folder.0).len(), 2);
    assert!(!folder.0.join("worlds").exists());
}

#[test]
fn a_newer_format_is_left_for_the_world_to_refuse() {
    let folder = old_folder();
    let newer = format!("(format: {})", SAVE_FORMAT + 1);
    fs::write(folder.0.join(FORMAT_FILE), &newer).unwrap();
    set_aside_old_saves(&folder.0).unwrap();
    assert!(set_aside_folders(&folder.0).is_empty());
    assert_eq!(
        fs::read_to_string(folder.0.join(FORMAT_FILE)).unwrap(),
        newer
    );
}

#[test]
fn a_new_folder_just_notes_its_format() {
    let folder = TestFolder::new();
    fs::create_dir_all(&folder.0).unwrap();
    set_aside_old_saves(&folder.0).unwrap();
    assert!(set_aside_folders(&folder.0).is_empty());
    assert!(folder.0.join(FORMAT_FILE).exists());
}

#[test]
fn a_sheet_saved_with_the_four_old_stats_still_loads() {
    use shared::creature::CreatureKind;
    use shared::sheet::Sheet;

    let old = "(stats: (strength: 10, dexterity: 10, intelligence: 10, constitution: 10), \
               skills: {}, appearance: ({}))";
    let sheet: Sheet = ron::from_str(old).unwrap();
    // Joining works its attributes out from the centre (`Sheet::settled`).
    let human = CreatureKind::Human;
    assert_eq!(
        sheet.settled(human).attributes,
        Sheet::new(human).attributes
    );
}
