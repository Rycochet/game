//! Keeping what is drawn in step with each element: its `Node`, background,
//! tooltip, picture and bar, whenever it is new or its attributes change.

use bevy::prelude::*;

use super::layout::{Around, node_for};
use super::parts::{Bar, Element, InTable, Inert, TextStyle};
use super::pictures::{Pictures, picture_for};
use super::tag::Tag;
use super::theme::Theme;
use crate::widgets::{HoverTip, Progress};

type Parts = (
    &'static Element,
    &'static TextStyle,
    Option<&'static InTable>,
    Option<&'static Bar>,
);

#[allow(clippy::too_many_arguments)]
pub(super) fn restyle(
    mut commands: Commands,
    theme: Res<Theme>,
    pictures: Res<Pictures>,
    assets: Res<AssetServer>,
    changed: Query<(Entity, &Element), Changed<Element>>,
    elements: Query<Parts>,
    cells: Query<(Entity, &InTable)>,
    mut nodes: Query<&mut Node>,
    mut bars: Query<&mut Progress>,
) {
    // A row hides its cells with it, so they are looked at again too.
    let mut stale: Vec<Entity> = Vec::new();
    for (entity, element) in &changed {
        stale.push(entity);
        if element.tag == Tag::Tr {
            let of_row = |(cell, spot): (Entity, &InTable)| match spot {
                InTable::Cell { of, .. } if *of == entity => Some(cell),
                _ => None,
            };
            stale.extend(cells.iter().filter_map(of_row));
        }
    }
    for entity in stale {
        let Ok((element, style, spot, bar)) = elements.get(entity) else {
            continue;
        };
        let (tag, attrs) = (element.tag, &element.attrs);
        let class = attrs.class.as_deref();
        if let Some(class) = class
            && !theme.has(tag, class)
        {
            warn!("the theme has no `{class}` look for `<{}>`", tag.name());
        }
        let look = theme.look(tag, class);
        let row = match spot {
            Some(InTable::Cell { of, .. }) => elements.get(*of).ok(),
            _ => None,
        };
        let around = Around {
            look: &look,
            spot,
            row_hidden: row.is_some_and(|(row, ..)| row.attrs.hidden),
            line: style.size.pixels() * theme.line,
        };
        let node = node_for(tag, attrs, &around);
        if let Ok(mut shown) = nodes.get_mut(entity)
            && *shown != node
        {
            *shown = node;
        }

        let mut entity = commands.entity(entity);
        let tinted = matches!(spot, Some(InTable::Row { tinted: true, .. }));
        match look.background.or(tinted.then_some(theme.row_tint)) {
            Some(colour) => entity.try_insert(BackgroundColor(colour)),
            None => entity.try_remove::<BackgroundColor>(),
        };
        match &attrs.title {
            Some(title) => entity.try_insert(HoverTip(title.to_string())),
            None => entity.try_remove::<HoverTip>(),
        };
        match attrs.inert {
            true => entity.try_insert(Inert),
            false => entity.try_remove::<Inert>(),
        };
        let src = attrs.text("src").filter(|_| tag == Tag::Img);
        if let Some((picture, as_writing)) =
            src.and_then(|src| picture_for(src, &pictures, &assets))
        {
            let tint = if as_writing {
                style.colour
            } else {
                Color::WHITE
            };
            entity.try_insert(ImageNode::new(picture).with_color(tint));
        }
        if let Some(Bar(bar)) = bar
            && let Ok(mut progress) = bars.get_mut(*bar)
        {
            let most = attrs.number("max").unwrap_or(1.0).max(f32::EPSILON);
            let full = attrs.number("value").unwrap_or(0.0) / most;
            progress.0 = full.clamp(0.0, 1.0);
        }
    }
}
