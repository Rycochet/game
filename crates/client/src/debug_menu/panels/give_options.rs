//! Give Item's line of options for the item's form: which show for its kind, what
//! each can be given the others (`item_form::choices`), and choosing them.

use bevy::prelude::*;
use shared::item::{ItemKind, Quality};
use shared::item_form::{Choices, RollRequest, choices, fields};

use super::give_option::{any, asked, asking, values, words};
use super::{GiveForm, GiveOption, GiveOptionsLine};
use crate::widgets::{Dropdown, DropdownChosen};

/// What `choices` last said, and for what: they take a while for armour sets.
type Found = Option<((ItemKind, Quality, RollRequest), Choices)>;

/// Shows the options the kind takes, each listing what it can be given the others,
/// and lets go of any asked for that the kind can't have.
pub(in crate::debug_menu) fn fill_give_options(
    mut form: ResMut<GiveForm>,
    mut found: Local<Found>,
    mut lines: Query<&mut Node, (With<GiveOptionsLine>, Without<GiveOption>)>,
    mut options: Query<(&mut GiveOption, &mut Dropdown, &mut Node), Without<GiveOptionsLine>>,
) {
    // A panel just opened has its dropdowns to fill.
    let opened = options.iter_mut().any(|(option, ..)| option.is_added());
    if !form.is_changed() && !opened {
        return;
    }
    let kind = form.kind;
    let takes = fields(kind);
    for mut node in &mut lines {
        let display = if takes.is_empty() {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != display {
            node.display = display;
        }
    }
    // A kind can't keep what it doesn't take.
    let mut request = form.request;
    for field in request.asked() {
        if !takes.contains(&field) {
            request = request.without(field);
        }
    }
    let key = (kind, form.quality, request);
    if found.as_ref().is_none_or(|(was, _)| *was != key) {
        *found = Some((key, choices(kind, form.quality, &request)));
    }
    let Some((_, choices)) = found.as_ref() else {
        return;
    };
    for (mut option, mut dropdown, mut node) in &mut options {
        let field = option.field;
        let shown = takes.contains(&field);
        let display = if shown { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
        let now = if shown {
            values(field, choices)
        } else {
            Vec::new()
        };
        // Asked for, but no longer one it can be: rolled as usual again.
        let chosen = asked(&request, field);
        if chosen.is_some_and(|chosen| !now.contains(&chosen)) {
            request = request.without(field);
        }
        let selected = asked(&request, field)
            .and_then(|chosen| now.iter().position(|&value| value == chosen))
            .map_or(0, |at| at + 1);
        if option.values != now {
            let names = now.iter().map(|&value| words(value));
            dropdown.options = std::iter::once(any(field).to_owned())
                .chain(names)
                .collect();
            option.values = now;
        }
        if dropdown.selected != selected {
            dropdown.selected = selected;
        }
    }
    if form.request != request {
        form.request = request;
    }
}

/// Follows the option dropdowns.
pub(in crate::debug_menu) fn choose_give_option(
    mut chosen: MessageReader<DropdownChosen>,
    options: Query<&GiveOption>,
    mut form: ResMut<GiveForm>,
) {
    for &DropdownChosen(dropdown, index) in chosen.read() {
        let Ok(option) = options.get(dropdown) else {
            continue;
        };
        form.request = match index.checked_sub(1).and_then(|at| option.values.get(at)) {
            Some(&value) => asking(form.request, value),
            None => form.request.without(option.field),
        };
    }
}
