use super::*;

#[test]
fn it_survives_a_message() {
    let id = Id::for_test(0x1234);
    let bytes = postcard::to_allocvec(&id).unwrap();
    // 16 bytes and their length.
    assert_eq!(bytes.len(), 17);
    assert_eq!(postcard::from_bytes::<Id>(&bytes).unwrap(), id);
}

#[test]
fn it_is_read_back_from_its_own_text_only() {
    let id = Id::for_test(0x1234);
    assert_eq!(Id::parse(&id.to_string()), Some(id));
    assert_eq!(Id::parse(&id.0.simple().to_string()), None);
    assert_eq!(Id::parse("not an id"), None);
    assert_eq!(Id::parse(""), None);
}
