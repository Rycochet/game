//! A small round "?" after the label of every labelled row (`span.labelled`)
//! that has a tip, from markup's `title` or set from code (`HoverTip`), so a
//! player can see which settings have more to say. Pointing at it shows the
//! row's tip, as pointing anywhere on the row does; a click or a tap on it
//! pins the tip beside it until the next press or key (`help_mark/pin`), as
//! fingers can't hover. A row built by code gets one the same way, naming its
//! label with `HelpAfter`.

mod pin;

use bevy::prelude::*;

pub(super) use pin::pin_tips;

use super::input::Click;
use super::parts::{Element, Solid};
use super::tag::Tag;
use crate::widgets::{HoverTip, INK};

/// The mark's picture: a question mark in a ring, from game-icons.net.
const ICON: &str = "icons/sbed/help.png";
/// How big the mark is, and how far it stands from the label's words.
const SIZE: f32 = 16.0;
const GAP: f32 = 6.0;
/// How strongly the mark shows: quieter than the words.
const ALPHA: f32 = 0.6;

/// On a "?": the row whose tip it shows.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct HelpMark(pub(super) Entity);

/// On a row with a "?": the mark.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct HelpShown(Entity);

/// On a row built by code (not markup's `span.labelled`) with a `HoverTip`:
/// its label, for the "?" to go right after, as for a labelled row.
#[derive(Component, Clone, Copy, Debug)]
pub struct HelpAfter(pub Entity);

/// Where a row's "?" goes: last in its label's element, or right after a
/// label built by code.
#[derive(Clone, Copy, Debug)]
enum Place {
    In(Entity),
    After(Entity),
}

/// Whether this element is a labelled row.
fn is_labelled(element: &Element) -> bool {
    element.tag == Tag::Span && element.attrs.class.as_deref() == Some("labelled")
}

type Row = (
    Entity,
    Option<&'static Element>,
    Option<&'static HelpAfter>,
    Option<&'static HoverTip>,
    Option<&'static HelpShown>,
    Option<&'static Children>,
);

/// What may have a "?": elements (labelled rows) and rows built by code.
type MayHaveOne = Or<(With<Element>, With<HelpAfter>)>;

/// Gives each labelled row with something to say a "?" after its label, and
/// takes it away once the row's tip is gone or empty.
pub(super) fn place_help_marks(
    mut commands: Commands,
    assets: Res<AssetServer>,
    rows: Query<Row, MayHaveOne>,
    elements: Query<&Element>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
) {
    for (row, element, after, tip, shown, held) in &rows {
        let place = match (after, element) {
            (Some(&HelpAfter(label)), _) => Some(Place::After(label)),
            (None, Some(element)) if is_labelled(element) => {
                let is_label = |child: &Entity| {
                    let element = elements.get(*child);
                    element.is_ok_and(|element| element.tag == Tag::Label)
                };
                let label = held.and_then(|held| held.iter().find(is_label));
                label.map(Place::In)
            }
            _ => continue,
        };
        let wanted = tip.is_some_and(|tip| !tip.0.is_empty());
        match (wanted, shown, place) {
            (true, None, Some(Place::In(label))) => {
                let mark = spawn_mark(&mut commands, &assets, row);
                commands.entity(mark).insert(ChildOf(label));
                commands.entity(row).insert(HelpShown(mark));
            }
            (true, None, Some(Place::After(label))) => {
                let Ok(parent) = parents.get(label).map(ChildOf::parent) else {
                    continue;
                };
                let at = children
                    .get(parent)
                    .ok()
                    .and_then(|held| held.iter().position(|child| child == label))
                    .map_or(0, |index| index + 1);
                let mark = spawn_mark(&mut commands, &assets, row);
                commands.entity(parent).insert_children(at, &[mark]);
                commands.entity(row).insert(HelpShown(mark));
            }
            (false, Some(&HelpShown(mark)), _) => {
                commands.entity(mark).try_despawn();
                commands.entity(row).remove::<HelpShown>();
            }
            _ => {}
        }
    }
}

/// A "?" for `row`, to be put in its place.
fn spawn_mark(commands: &mut Commands, assets: &AssetServer, row: Entity) -> Entity {
    let picture = ImageNode::new(assets.load(ICON)).with_color(INK.with_alpha(ALPHA));
    commands
        .spawn((
            HelpMark(row),
            Solid,
            picture,
            Node {
                width: px(SIZE),
                height: px(SIZE),
                margin: UiRect::left(px(GAP)),
                align_self: AlignSelf::Center,
                flex_shrink: 0.0,
                ..default()
            },
        ))
        // A click on the "?" is the mark's, not the label's control's.
        .observe(|mut click: On<Click>| click.propagate(false))
        .id()
}

#[cfg(test)]
mod tests;
