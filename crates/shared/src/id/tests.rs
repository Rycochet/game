use super::*;

#[test]
fn it_survives_a_message() {
    let id = Id::for_test(0x1234);
    let bytes = postcard::to_allocvec(&id).unwrap();
    // 16 bytes and their length.
    assert_eq!(bytes.len(), 17);
    assert_eq!(postcard::from_bytes::<Id>(&bytes).unwrap(), id);
}
