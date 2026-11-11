use super::*;
use crate::dom::markup::parse;

fn checked(source: &'static str) -> Result<Vec<CheckedNode>, MarkupError> {
    check(parse(source)?)
}

fn error(source: &'static str) -> String {
    checked(source).unwrap_err().what
}

fn element(node: &CheckedNode) -> &Checked {
    match node {
        CheckedNode::Element(element) => element,
        CheckedNode::Text(_) => panic!("expected an element"),
    }
}

#[test]
fn ids_take_their_forms_name() {
    let nodes = checked(r#"<form id="connect"><p id="status">Ready</p></form>"#).unwrap();
    let form = element(&nodes[0]);
    assert_eq!(form.id, None);
    assert_eq!(form.form, Some("connect"));
    let status = element(&form.children[0]);
    assert_eq!(status.id.unwrap().to_string(), "connect/status");
}

#[test]
fn a_form_attribute_names_the_form_of_one_element() {
    let nodes = checked(r#"<p id="status" form="connect"/>"#).unwrap();
    assert_eq!(element(&nodes[0]).id.unwrap().to_string(), "connect/status");
    assert_eq!(
        error(r#"<p id="status"/>"#),
        "an `id` needs a `<form>` round it, or a `form`"
    );
}

#[test]
fn unknown_things_are_errors() {
    assert_eq!(error("<blink/>"), "`<blink>` is not known");
    assert_eq!(error("<p style='x'/>"), "`<p>` has no `style`");
    assert_eq!(error("<script>x</script>"), "`<script>` is not known");
    assert_eq!(error("words"), "words must be inside an element");
}

#[test]
fn an_id_used_twice_is_an_error() {
    let source = r#"<form id="a"><p id="x"/><p id="x"/></form>"#;
    assert_eq!(error(source), "`a/x` is used twice");
}

#[test]
fn some_elements_need_ids() {
    assert_eq!(error("<canvas/>"), "`<canvas>` needs an `id`");
    assert_eq!(error("<form><p/></form>"), "`<form>` needs an `id`");
}

#[test]
fn labels_name_something_in_their_form() {
    let good = r#"<form id="a"><label for="x">X</label><p id="x"/></form>"#;
    assert!(checked(good).is_ok());
    let bad = r#"<form id="a"><label for="y">X</label><p id="x"/></form>"#;
    assert_eq!(error(bad), "`for` names `a/y`, which isn't here");
}

#[test]
fn elements_go_where_they_can() {
    assert_eq!(error("<td/>"), "`<td>` goes in `<tr>`");
    assert_eq!(
        error("<option/>"),
        "`<option>` goes in `<select>` or `<toggle-row>` or `<tab-bar>` or `<colour-picker>`"
    );
    assert_eq!(error("<table><p/></table>"), "`<table>` can't hold `<p>`");
    assert_eq!(error("<table>words</table>"), "`<table>` can't hold words");
    assert_eq!(
        error("<br>words</br>"),
        "`<br>` holds nothing: write it `<br/>`"
    );
    assert_eq!(
        error("<progress><p/></progress>"),
        "`<progress>` can't hold `<p>`"
    );
    assert!(checked("<table><tr><td>a</td><th>b</th></tr></table>").is_ok());
}

#[test]
fn a_templates_ids_are_its_own() {
    let source = r#"<form id="a">
        <template id="row"><p id="name"/></template>
        <p id="name"/>
    </form>"#;
    assert!(checked(source).is_ok());
}

#[test]
fn errors_keep_their_place() {
    let error = checked("<div>\n  <p margin=\"3\"/>\n</div>").unwrap_err();
    assert_eq!((error.at.line, error.at.column), (2, 6));
}
