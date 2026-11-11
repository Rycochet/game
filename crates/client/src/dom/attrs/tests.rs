use super::*;

fn set(attrs: &mut Attrs, tag: Tag, name: &str, value: &'static str) -> Result<(), String> {
    attrs.set(tag, name, Some(value.into()))
}

#[test]
fn lengths_are_pixels_or_percent() {
    let mut attrs = Attrs::default();
    set(&mut attrs, Tag::Div, "width", "120").unwrap();
    set(&mut attrs, Tag::Div, "left", "50%").unwrap();
    assert_eq!(attrs.width, Some(Length::Pixels(120.0)));
    assert_eq!(attrs.left, Some(Length::Percent(50.0)));
    assert!(attrs.is_placed());
    assert!(set(&mut attrs, Tag::Div, "height", "-4").is_err());
    assert!(set(&mut attrs, Tag::Div, "height", "tall").is_err());
}

#[test]
fn flags_stand_alone() {
    let mut attrs = Attrs::default();
    attrs.set(Tag::P, "hidden", None).unwrap();
    assert!(attrs.hidden);
    set(&mut attrs, Tag::P, "inert", "inert").unwrap();
    assert!(attrs.inert);
    assert!(set(&mut attrs, Tag::P, "disabled", "no").is_err());
    assert!(attrs.set(Tag::P, "width", None).is_err());
}

#[test]
fn margins_are_multiples_of_four() {
    let mut attrs = Attrs::default();
    set(&mut attrs, Tag::P, "margin", "8").unwrap();
    set(&mut attrs, Tag::P, "margin-right", "0").unwrap();
    assert_eq!(attrs.margin, [Some(8.0), Some(0.0), Some(8.0), Some(8.0)]);
    let error = set(&mut attrs, Tag::P, "margin-top", "6").unwrap_err();
    assert_eq!(error, "`margin-top` must be a multiple of 4, not `6`");
}

#[test]
fn a_side_told_three_ways_is_refused() {
    let mut attrs = Attrs::default();
    set(&mut attrs, Tag::Div, "left", "0").unwrap();
    set(&mut attrs, Tag::Div, "right", "0").unwrap();
    let error = set(&mut attrs, Tag::Div, "width", "10").unwrap_err();
    assert_eq!(error, "`left` and `right` already give the width");
}

#[test]
fn only_known_attributes_are_taken() {
    let mut attrs = Attrs::default();
    assert_eq!(
        set(&mut attrs, Tag::Div, "src", "icon:x").unwrap_err(),
        "`<div>` has no `src`"
    );
    set(&mut attrs, Tag::Img, "src", "icon:x").unwrap();
    assert!(set(&mut attrs, Tag::Div, "style", "color: red").is_err());
    assert!(set(&mut attrs, Tag::Div, "onclick", "go()").is_err());
}

#[test]
fn attributes_are_written_back_as_markup() {
    let mut attrs = Attrs::default();
    set(&mut attrs, Tag::P, "class", "warning").unwrap();
    set(&mut attrs, Tag::P, "title", "A \"tip\" & more").unwrap();
    set(&mut attrs, Tag::P, "width", "50%").unwrap();
    set(&mut attrs, Tag::P, "margin-left", "8").unwrap();
    attrs.set(Tag::P, "hidden", None).unwrap();
    let written = attrs.written();
    let expected = r#" class="warning" title="A &quot;tip&quot; &amp; more" width="50%" margin-left="8" hidden"#;
    assert_eq!(written, expected);
}

#[test]
fn clearing_puts_one_attribute_back() {
    let mut attrs = Attrs::default();
    attrs.set(Tag::P, "hidden", None).unwrap();
    set(&mut attrs, Tag::P, "margin", "8").unwrap();
    set(&mut attrs, Tag::P, "width", "40").unwrap();
    attrs.clear(Tag::P, "hidden").unwrap();
    attrs.clear(Tag::P, "margin-left").unwrap();
    assert!(!attrs.hidden);
    assert_eq!(attrs.margin, [Some(8.0), Some(8.0), Some(8.0), None]);
    assert_eq!(attrs.width, Some(Length::Pixels(40.0)));
    assert!(attrs.clear(Tag::P, "src").is_err());
}
