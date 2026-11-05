use super::*;
use std::time::{Duration, SystemTime};

use crate::storage::tests::TestFolder;

/// Writes a build named by its hash, as last changed `days_ago`.
fn old_build(history: &Path, number: u8, days_ago: u64) -> Hash {
    let data = [number; 10];
    let hash = web_delta::sha256(&data);
    let path = history.join(format!("{}.wasm", hex(&hash)));
    fs::write(&path, data).unwrap();
    let when = SystemTime::now() - Duration::from_secs(days_ago * 24 * 60 * 60);
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(when)
        .unwrap();
    hash
}

#[test]
fn keeps_the_current_build_and_the_newest_before_it() {
    let folder = TestFolder::new();
    let history = &folder.0;
    fs::create_dir_all(history).unwrap();
    let builds: Vec<_> = (1..=5).map(|n| old_build(history, n, n as u64)).collect();
    let current = b"the current build";
    let hash = web_delta::sha256(current);
    // A delta still wanted, one to another build, and one from a build dropped.
    fs::write(delta_path(history, &builds[0], &hash), "kept").unwrap();
    fs::write(delta_path(history, &builds[0], &builds[1]), "to another").unwrap();
    fs::write(delta_path(history, &builds[4], &hash), "from a dropped one").unwrap();

    let kept = keep(history, current, &hash).unwrap();

    let kept_hashes: Vec<_> = kept.iter().map(|(hash, _)| *hash).collect();
    assert_eq!(kept_hashes, builds[..KEPT]);
    assert_eq!(fs::read(&kept[0].1).unwrap(), [1; 10]);
    assert_eq!(
        fs::read(history.join(format!("{}.wasm", hex(&hash)))).unwrap(),
        current
    );
    let mut left: Vec<_> = fs::read_dir(history)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    left.sort();
    assert_eq!(left.len(), KEPT + 2, "{left:?}");
    assert!(delta_path(history, &builds[0], &hash).exists());
    assert!(!delta_path(history, &builds[4], &hash).exists());
}

#[test]
fn a_build_already_kept_stays_as_it_is() {
    let folder = TestFolder::new();
    let history = &folder.0;
    fs::create_dir_all(history).unwrap();
    let hash = old_build(history, 9, 30);
    let path = history.join(format!("{}.wasm", hex(&hash)));
    let before = fs::metadata(&path).unwrap().modified().unwrap();

    assert!(keep(history, &[9; 10], &hash).unwrap().is_empty());
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), before);
}
