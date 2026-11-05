use super::*;

fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        headers.append(*name, HeaderValue::from_str(value).unwrap());
    }
    headers
}

#[test]
fn dcz_is_taken_only_when_offered() {
    assert!(takes_dcz(&headers(&[(
        "accept-encoding",
        "gzip, br, zstd, dcb, dcz"
    )])));
    assert!(takes_dcz(&headers(&[("accept-encoding", "DCZ;q=0.5")])));
    assert!(!takes_dcz(&headers(&[("accept-encoding", "gzip, br")])));
    assert!(!takes_dcz(&headers(&[("accept-encoding", "dcz;q=0")])));
    assert!(!takes_dcz(&HeaderMap::new()));
}

#[test]
fn the_available_dictionary_is_read_from_its_hash() {
    let hash = web_delta::sha256(b"an old build");
    let value = format!(":{}:", STANDARD.encode(hash));
    assert_eq!(
        available(&headers(&[(AVAILABLE_DICTIONARY, &value)])),
        Some(hash)
    );
    let short = format!(":{}:", STANDARD.encode([1, 2, 3]));
    assert_eq!(available(&headers(&[(AVAILABLE_DICTIONARY, &short)])), None);
    assert_eq!(
        available(&headers(&[(AVAILABLE_DICTIONARY, "no colons")])),
        None
    );
    assert_eq!(available(&HeaderMap::new()), None);
}
