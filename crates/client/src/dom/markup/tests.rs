use std::borrow::Cow;

use super::*;

fn element(node: &RawNode) -> &RawElement {
    match node {
        RawNode::Element(element) => element,
        RawNode::Text(..) => panic!("expected an element"),
    }
}

fn words(node: &RawNode) -> &str {
    match node {
        RawNode::Text(words, _) => words,
        RawNode::Element(_) => panic!("expected words"),
    }
}

fn error(source: &'static str) -> (u32, u32, String) {
    let error = parse(source).unwrap_err();
    (error.at.line, error.at.column, error.what)
}

#[test]
fn elements_nest_and_close_themselves() {
    let nodes = parse("<div><br/><p>Words</p></div>").unwrap();
    let div = element(&nodes[0]);
    assert_eq!(div.name, "div");
    assert_eq!(element(&div.children[0]).name, "br");
    let p = element(&div.children[1]);
    let at = Place {
        line: 1,
        column: 14,
    };
    assert_eq!(p.children[0], RawNode::Text("Words".into(), at));
}

#[test]
fn spaces_are_tidied_and_empty_words_dropped() {
    let nodes = parse("<p>\n   Some\n   words   here\n</p>\n  ").unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(words(&element(&nodes[0]).children[0]), "Some words here");
}

#[test]
fn attributes_may_stand_alone_or_have_values() {
    let nodes = parse(r#"<p hidden title="A &amp; B" class='x'/>"#).unwrap();
    let attrs = &element(&nodes[0]).attrs;
    assert_eq!(attrs[0].name, "hidden");
    assert_eq!(attrs[0].value, None);
    assert_eq!(attrs[1].value.as_deref(), Some("A & B"));
    assert_eq!(attrs[2].value.as_deref(), Some("x"));
}

#[test]
fn plain_values_are_borrowed_from_the_source() {
    let nodes = parse(r#"<p id="status"/>"#).unwrap();
    let value = element(&nodes[0]).attrs[0].value.clone().unwrap();
    assert!(matches!(value, Cow::Borrowed("status")));
}

#[test]
fn entities_are_read() {
    let nodes = parse("<p>1 &lt; 2 &#65;&#x42;</p>").unwrap();
    assert_eq!(words(&element(&nodes[0]).children[0]), "1 < 2 AB");
}

#[test]
fn comments_are_left_out() {
    let nodes = parse("<!-- a note --><br/>").unwrap();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn errors_say_where() {
    let closes = "`</div>` closes `<p>`".to_owned();
    assert_eq!(error("<div>\n  <p>\n</div>"), (3, 1, closes));
    assert_eq!(error("<br>"), (1, 5, "`<br>` is never closed".into()));
    assert_eq!(error("</p>"), (1, 1, "`</p>` closes nothing".into()));
}

#[test]
fn errors_say_what() {
    assert_eq!(error("<Div/>").2, "`Div` must be in lower case");
    assert_eq!(error("<p a=b/>").2, "the value of `a` must be in quotes");
    assert_eq!(error("<p a='1' a='2'/>").2, "`a` is given twice");
    assert_eq!(error("<p>a & b</p>").2, "`&` must be written `&amp;`");
    assert_eq!(error("<p>&what;</p>").2, "`&what;` is not known");
    assert_eq!(error("<!-- never").2, "this comment is never closed");
}
