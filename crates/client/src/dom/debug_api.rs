//! `client.ui.at` and `client.ui.html` (debug API only): what the pointer
//! would be on at a point, and the documents on show written back as markup.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params};

use super::input::{Finder, Hit};
use super::parts::{Control, ControlOf, Document, Element, InTable, Solid, SurfaceOf, Words};
use crate::common::in_interface;
use crate::widgets::UiId;

pub(super) fn add(app: &mut App) {
    use server::debug_api::AddDebugMethod;
    app.add_debug_method("client.ui.at", at)
        .add_debug_method("client.ui.html", html);
}

#[derive(Deserialize)]
struct AtAsked {
    /// In the window's logical pixels, as `client.click` takes them. Left
    /// out: where the pointer is now.
    x: Option<f32>,
    y: Option<f32>,
}

/// `client.ui.at {x?, y?}`: the thing in front at a point (`target`) and
/// everything it is in, going outwards (`chain`): what would be pointed at
/// and pressed there, and nothing else.
fn at(
    In(request): In<Option<Value>>,
    finder: Finder,
    hit: Res<Hit>,
    scale: Res<UiScale>,
    about: Query<(Option<&UiId>, Option<&Element>, Has<Solid>)>,
) -> BrpResult {
    let asked: AtAsked = params(request)?;
    let chain = match (asked.x, asked.y) {
        (Some(x), Some(y)) => {
            let target = finder.target_at(in_interface(Vec2::new(x, y), &scale));
            target
                .map(|target| finder.chain_of(target))
                .unwrap_or_default()
        }
        _ => hit.chain.clone(),
    };
    let describe = |entity: &Entity| {
        let (id, element, solid) = about.get(*entity).ok()?;
        Some(json!({
            "entity": entity.to_string(),
            "id": id.map(UiId::to_string),
            "element": element.map(|element| element.tag.name()),
            "solid": solid || element.is_some(),
        }))
    };
    let chain: Vec<Value> = chain.iter().filter_map(describe).collect();
    Ok(json!({ "target": chain.first(), "chain": chain }))
}

/// What the dump reads of each node.
type Part = (
    Option<&'static Element>,
    Option<&'static UiId>,
    Option<&'static InTable>,
    Option<&'static Text>,
    Has<Words>,
    Option<&'static Children>,
    Option<&'static Control>,
);

/// `client.ui.html`: every document as markup, as it is now.
fn html(
    In(_): In<Option<Value>>,
    documents: Query<(Entity, &Document, &InheritedVisibility)>,
    surfaces: Query<(&SurfaceOf, &ControlOf)>,
    parts: Query<Part>,
) -> BrpResult {
    let dump = |(root, Document(template), visible): (Entity, &Document, &InheritedVisibility)| {
        let mut markup = String::new();
        write_children(&parts, root, 0, &mut markup);
        // Its panels and dialogs stand apart from it, so they come after.
        for (SurfaceOf(document), ControlOf(element)) in &surfaces {
            if *document == root {
                write_node(&parts, *element, 0, &mut markup);
            }
        }
        json!({ "name": template.name, "shown": visible.get(), "html": markup })
    };
    let documents: Vec<Value> = documents.iter().map(dump).collect();
    Ok(json!({ "documents": documents }))
}

/// Writes what `entity` holds. A table's rows are strips beside their cells
/// (see `InTable`), so each row is written round the cells that follow it.
fn write_children(parts: &Query<Part>, entity: Entity, depth: usize, out: &mut String) {
    let Ok((.., children, _)) = parts.get(entity) else {
        return;
    };
    let indent = "  ".repeat(depth);
    let mut row_open = false;
    for child in children.map(|held| held.to_vec()).unwrap_or_default() {
        let spot = parts.get(child).ok().and_then(|part| part.2);
        match spot {
            Some(InTable::Row { .. }) => {
                if std::mem::replace(&mut row_open, true) {
                    out.push_str(&format!(
                        "{indent}</tr>
"
                    ));
                }
                if let Some((name, inside_tag)) = opening(parts, child) {
                    out.push_str(&format!(
                        "{indent}<{name}{inside_tag}>
"
                    ));
                }
            }
            Some(InTable::Cell { .. }) => write_node(parts, child, depth + 1, out),
            _ => write_node(parts, child, depth, out),
        }
    }
    if row_open {
        out.push_str(&format!(
            "{indent}</tr>
"
        ));
    }
}

/// An element's name and what is written inside its opening tag.
fn opening(parts: &Query<Part>, entity: Entity) -> Option<(&'static str, String)> {
    let (element, id, .., control) = parts.get(entity).ok()?;
    let element = element?;
    // A control's id is on the widget that shows it.
    let on_widget = control.and_then(|control| parts.get(control.0).ok());
    let id = id.or(on_widget.and_then(|widget| widget.1));
    let id = id.map(|id| format!(" id=\"{id}\"")).unwrap_or_default();
    Some((
        element.tag.name(),
        format!("{id}{}", element.attrs.written()),
    ))
}

fn write_node(parts: &Query<Part>, entity: Entity, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    if let Some((name, inside_tag)) = opening(parts, entity) {
        let mut held = String::new();
        write_children(parts, entity, depth + 1, &mut held);
        if held.is_empty() {
            out.push_str(&format!(
                "{indent}<{name}{inside_tag}/>
"
            ));
        } else {
            out.push_str(&format!(
                "{indent}<{name}{inside_tag}>
{held}{indent}</{name}>
"
            ));
        }
        return;
    }
    let Ok((_, _, _, text, words, ..)) = parts.get(entity) else {
        return;
    };
    match text.filter(|_| words) {
        Some(text) => out.push_str(&format!(
            "{indent}{}
",
            text.0
        )),
        // Something an element is made of (a bar): only its words show.
        None => write_children(parts, entity, depth, out),
    }
}
