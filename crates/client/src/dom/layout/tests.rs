use super::*;
use crate::dom::theme::Theme;

fn attrs(tag: Tag, pairs: &[(&str, &'static str)]) -> Attrs {
    let mut attrs = Attrs::default();
    for (name, value) in pairs {
        attrs.set(tag, name, Some((*value).into())).unwrap();
    }
    attrs
}

fn node(tag: Tag, pairs: &[(&str, &'static str)], spot: Option<InTable>) -> Node {
    let look = Theme::parchment().look(tag, None);
    let around = Around {
        look: &look,
        spot: spot.as_ref(),
        row_hidden: false,
        line: 20.0,
    };
    node_for(tag, &attrs(tag, pairs), &around)
}

#[test]
fn blocks_stack_and_inline_flows() {
    let div = node(Tag::Div, &[], None);
    assert_eq!(div.flex_direction, FlexDirection::Column);
    let span = node(Tag::Span, &[], None);
    assert_eq!(span.flex_direction, FlexDirection::Row);
    assert_eq!(span.flex_wrap, FlexWrap::Wrap);
    assert_eq!(span.column_gap, px(4));
}

#[test]
fn sides_place_an_element_out_of_the_flow() {
    let placed = node(Tag::Div, &[("right", "16"), ("bottom", "25%")], None);
    assert_eq!(placed.position_type, PositionType::Absolute);
    assert_eq!(placed.right, px(16));
    assert_eq!(placed.bottom, percent(25));
    assert_eq!(placed.left, Val::Auto);
    let flowing = node(Tag::Div, &[("width", "40")], None);
    assert_eq!(flowing.position_type, PositionType::Relative);
    assert_eq!(flowing.width, px(40));
}

#[test]
fn spacing_is_what_holds_things_unless_one_gives_its_own() {
    // Things are spaced by what they are in: a stack's gap, not margins.
    let plain = node(Tag::P, &[], None);
    assert_eq!(plain.margin.bottom, px(0));
    assert_eq!(node(Tag::Div, &[], None).row_gap, px(8));
    let tight = node(
        Tag::P,
        &[("margin-bottom", "0"), ("margin-left", "4")],
        None,
    );
    assert_eq!(tight.margin.bottom, px(0));
    assert_eq!(tight.margin.left, px(4));
}

#[test]
fn align_lines_up_what_is_inside() {
    let div = node(Tag::Div, &[("align", "right")], None);
    assert_eq!(div.align_items, AlignItems::FlexEnd);
    let span = node(Tag::Span, &[("align", "center"), ("valign", "top")], None);
    assert_eq!(span.justify_content, JustifyContent::Center);
    assert_eq!(span.align_items, AlignItems::FlexStart);
}

#[test]
fn hidden_and_templates_are_not_laid_out() {
    let mut hidden = Attrs::default();
    hidden.set(Tag::P, "hidden", None).unwrap();
    let look = Look::default();
    let around = Around {
        look: &look,
        spot: None,
        row_hidden: false,
        line: 20.0,
    };
    assert_eq!(node_for(Tag::P, &hidden, &around).display, Display::None);
    let template = node_for(Tag::Template, &Attrs::default(), &around);
    assert_eq!(template.display, Display::None);
}

#[test]
fn table_parts_take_their_places_in_the_grid() {
    let table = node(
        Tag::Table,
        &[("cellspacing", "4")],
        Some(InTable::Table {
            columns: 3,
            wide: None,
        }),
    );
    assert_eq!(table.display, Display::Grid);
    let three: Vec<RepeatedGridTrack> = vec![RepeatedGridTrack::auto(1); 3];
    assert_eq!(table.grid_template_columns, three);
    assert_eq!(table.column_gap, px(4));
    let spot = InTable::Cell {
        of: Entity::PLACEHOLDER,
        row: 1,
        column: 1,
        span: 2,
        padding: 4.0,
    };
    let cell = node(Tag::Td, &[], Some(spot));
    assert_eq!(cell.grid_row, GridPlacement::start(2));
    assert_eq!(cell.grid_column, GridPlacement::start_span(2, 2));
    assert_eq!(cell.padding, UiRect::all(px(4)));
}

#[test]
fn pictures_are_a_line_tall_unless_sized() {
    assert_eq!(node(Tag::Img, &[], None).height, px(20));
    assert_eq!(node(Tag::Img, &[("height", "64")], None).height, px(64));
}
