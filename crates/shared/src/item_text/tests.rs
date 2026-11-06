use super::*;

const A: &str = "0189a1b2-c3d4-6e5f-8a9b-0c1d2e3f4a5b";

fn id(text: &str) -> Id {
    Id(Uuid::parse_str(text).unwrap())
}

#[test]
fn an_item_is_written_and_read_back() {
    let item = Id::for_test(0x1234);
    let token = item_token(item);
    assert_eq!(token, "${item:00000000-0000-0000-0000-000000001234}");
    assert_eq!(pieces(&token), vec![Piece::Item(item)]);
}

#[test]
fn text_is_split_round_items() {
    let text = format!("a ${{item:{A}}} and ${{item:{A}}}!");
    assert_eq!(
        pieces(&text),
        vec![
            Piece::Text("a "),
            Piece::Item(id(A)),
            Piece::Text(" and "),
            Piece::Item(id(A)),
            Piece::Text("!"),
        ]
    );
    assert_eq!(items_in(&text), vec![id(A)]);
}

#[test]
fn upper_case_ids_are_the_same_item() {
    let text = format!("${{item:{}}}", A.to_uppercase());
    assert_eq!(pieces(&text), vec![Piece::Item(id(A))]);
}

#[test]
fn anything_not_exactly_an_item_is_plain_text() {
    let simple = A.replace('-', "");
    for text in [
        "${item:}".to_owned(),
        format!("${{item:{A}"),
        format!("${{item:{simple}}}"),
        format!("${{item:{{{A}}}}}"),
        format!("${{item: {A}}}"),
        format!("${{Item:{A}}}"),
        format!("${{item:{}}}", A.replace('a', "g")),
        "${item:café}".to_owned(),
        "plain words".to_owned(),
    ] {
        assert_eq!(pieces(&text), vec![Piece::Text(&text)], "{text}");
    }
}

#[test]
fn a_broken_item_does_not_hide_a_good_one_after_it() {
    let text = format!("${{item:${{item:{A}}}");
    assert_eq!(
        pieces(&text),
        vec![Piece::Text("${item:"), Piece::Item(id(A))]
    );
}

#[test]
fn empty_text_has_no_pieces() {
    assert!(pieces("").is_empty());
}
