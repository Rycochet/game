//! The Credits layer: who made the game and what it is made with, scrolling by
//! itself when there is more than fits. `docs/credits.md` says the same for
//! the website, with the licences and links.

use bevy::prelude::*;

use super::spawn::{spawn_layer, spawn_title_row};
use super::{ABOVE_MENU_LAYER, GAP, MARGIN, SystemMenu};
use crate::widgets::{AutoScroll, INK, TextSize, spawn_scroll_area};

const CREDITS_WIDTH: f32 = 560.0;
/// The most of the window's height the list takes before it scrolls.
const LIST_SHARE: f32 = 65.0;
/// The entries' width: the panel's, less its margins and room for a scroll bar.
const ENTRY_WIDTH: f32 = CREDITS_WIDTH - 2.0 * MARGIN - 24.0;
const ENTRY_GAP: f32 = 20.0;
/// How faint the line saying what someone did is.
const WHAT_ALPHA: f32 = 0.7;

/// What was done, and by whom (or what it is, and whose).
pub(super) const CREDITS: &[(&str, &str)] = &[
    ("Created by", "Ryc O'Chet"),
    ("Code and placeholder 3D images", "Claude Code (Opus 5.5)"),
    (
        "Monster and terrain tiles, and the AngbandTk tiles",
        "David E. Gervais",
    ),
    (
        "Icons, from game-icons.net",
        "Lorc, Delapouite and the site's other contributors",
    ),
    ("Title font: Cinzel Decorative", "Natanael Gama"),
    (
        "Writing font: Lexend",
        "Bonnie Shaver-Troup, Thomas Jockin, Santiago Orozco, Héctor Gómez and Superunion",
    ),
    ("Emoji font: Noto Emoji", "Google"),
    (
        "Some creatures (the drider): System Reference Document 5.1, CC BY 4.0",
        "Wizards of the Coast LLC",
    ),
    (
        "Built with",
        "Bevy, and the many open source libraries it and the game use",
    ),
    (
        "Licence",
        "Hall of Worlds may be shared and changed, but not sold: Creative Commons \
         Attribution-NonCommercial 4.0. The art and fonts keep their own licences.",
    ),
    ("", "Thank you for playing!"),
];

/// The list, so opening the layer can start it from the top.
#[derive(Component)]
pub(super) struct CreditsList;

pub(super) fn spawn_credits(commands: &mut Commands) {
    let panel = spawn_layer(
        commands,
        ABOVE_MENU_LAYER,
        &[SystemMenu::Credits],
        px(CREDITS_WIDTH),
        Val::Auto,
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(MARGIN),
            ..default()
        },
    );
    commands.entity(panel).with_children(|panel| {
        spawn_title_row(panel, "Credits", "credits");
        let list = spawn_scroll_area(panel, Val::Vh(LIST_SHARE));
        panel.commands().entity(list).insert(CreditsList);
        panel.commands().entity(list).with_children(|list| {
            for &(what, who) in CREDITS {
                list.spawn(Node {
                    width: px(ENTRY_WIDTH),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(GAP / 2.0),
                    margin: UiRect::bottom(px(ENTRY_GAP)),
                    ..default()
                })
                .with_children(|entry| {
                    if !what.is_empty() {
                        entry.spawn((
                            Text::new(what),
                            TextSize::Medium.font(),
                            TextColor(INK.with_alpha(WHAT_ALPHA)),
                            TextLayout::justify(Justify::Center),
                        ));
                    }
                    entry.spawn((
                        Text::new(who),
                        TextSize::Large.font(),
                        TextColor(INK),
                        TextLayout::justify(Justify::Center),
                    ));
                });
            }
        });
    });
}

/// Each time the credits open, they start from the top and scroll by themselves
/// again, until scrolled by hand.
pub(super) fn start_credits(
    menu: Res<SystemMenu>,
    mut list: Single<(Entity, &mut ScrollPosition), With<CreditsList>>,
    mut commands: Commands,
) {
    if menu.is_changed() && *menu == SystemMenu::Credits {
        list.1.y = 0.0;
        commands.entity(list.0).insert(AutoScroll::default());
    }
}

#[cfg(test)]
mod tests;
