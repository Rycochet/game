use super::*;

/// Bytes that don't compress by themselves, from a simple shuffle.
fn noise(length: usize, seed: u32) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect()
}

#[test]
fn a_small_change_makes_a_small_delta() {
    let old = noise(300_000, 1);
    let mut new = old.clone();
    new[1000..1100].copy_from_slice(&noise(100, 2));
    new.extend(noise(500, 3));

    let delta = encode(&old, &new).unwrap();
    assert!(delta.len() < 2_000, "{} bytes", delta.len());
    assert_eq!(decode(&old, &delta).unwrap(), new);
}

#[test]
fn starts_with_the_header_and_the_dictionarys_hash() {
    let old = noise(1000, 4);
    let delta = encode(&old, b"new").unwrap();
    assert_eq!(delta[..8], MAGIC);
    assert_eq!(delta[8..40], sha256(&old));
}

#[test]
fn another_dictionary_is_refused() {
    let delta = encode(&noise(1000, 5), b"new").unwrap();
    assert!(decode(&noise(1000, 6), &delta).is_err());
    assert!(decode(b"", b"short").is_err());
}

#[test]
fn hex_goes_both_ways() {
    let hash = sha256(b"abc");
    assert_eq!(
        hex(&hash),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(from_hex(&hex(&hash)), Some(hash));
    assert_eq!(from_hex("ba78"), None);
    assert_eq!(from_hex(&"g".repeat(64)), None);
}

#[test]
fn windows_stay_within_what_browsers_take() {
    assert_eq!(window_log(0), 23);
    assert_eq!(window_log(58_000_000), 26);
    assert_eq!(window_log(1 << 30), 27);
}
