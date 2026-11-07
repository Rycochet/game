//! The Character panel's tabs: Character (what they are like), Inventory (what they
//! wear and carry, and the ground), Abilities (filled by `abilities`) and Skills
//! (filled by `skills_page`). Only the
//! chosen tab's page is laid out; the others are kept, hidden.

use bevy::prelude::*;
use shared::item::Holder;

use super::super::body::spawn_rows;
use super::super::ground::spawn_section;
use super::grids::spawn_grid;
use super::{COLUMN_GAP, FONT};
use crate::common::{AbilitiesPage, SheetText, SkillsPage};
use crate::widgets::{INK, TabChosen, UiId, spawn_tabs};

/// A tab of the Character panel, in the order they are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::inventory) enum Page {
    Character,
    #[default]
    Inventory,
    Abilities,
    Skills,
}

impl Page {
    const ALL: [Self; 4] = [
        Self::Character,
        Self::Inventory,
        Self::Abilities,
        Self::Skills,
    ];
    const NAMES: [&str; 4] = ["Character", "Inventory", "Abilities", "Skills"];

    fn index(self) -> usize {
        self as usize
    }
}

/// The open Character panel's tab row and pages.
#[derive(Clone, Copy, Debug)]
pub(in crate::inventory) struct Pages {
    pub(super) row: Entity,
    pages: [Entity; 4],
}

impl Pages {
    /// Shows `page` and hides the others.
    pub(super) fn show(&self, page: Page, commands: &mut Commands) {
        for (index, &node) in self.pages.iter().enumerate() {
            let display = if index == page.index() {
                Display::Flex
            } else {
                Display::None
            };
            commands
                .entity(node)
                .entry::<Node>()
                .and_modify(move |mut node| node.display = display);
        }
    }
}

/// The tabs and their pages, `shown` on show. Returns them, and the holder of the
/// ground's section.
pub(super) fn spawn_pages(
    content: &mut ChildSpawnerCommands,
    shown: Page,
    size: (u8, u8),
) -> (Pages, Entity) {
    let row = spawn_tabs(content, &Page::NAMES, shown.index(), FONT);
    content
        .commands()
        .entity(row)
        .insert(UiId::new("inventory", "tabs"));
    let page = |page: Page| Node {
        display: if page == shown {
            Display::Flex
        } else {
            Display::None
        },
        column_gap: px(COLUMN_GAP),
        margin: UiRect::top(px(COLUMN_GAP / 2.0)),
        ..default()
    };
    let character = content
        .spawn(page(Page::Character))
        .with_children(|page| {
            page.spawn((SheetText, Text::default(), FONT.font(), TextColor(INK)));
        })
        .id();
    let mut ground = None;
    let inventory = content
        .spawn(page(Page::Inventory))
        .with_children(|columns| {
            spawn_rows(columns);
            spawn_grid(columns, Holder::Inventory, size, false);
            ground = Some(spawn_section(columns));
        })
        .id();
    let abilities = content.spawn((page(Page::Abilities), AbilitiesPage)).id();
    let skills = content.spawn((page(Page::Skills), SkillsPage)).id();
    let pages = Pages {
        row,
        pages: [character, inventory, abilities, skills],
    };
    (pages, ground.expect("spawned"))
}

/// A tab pressed shows its page, and is remembered for next time.
pub(in crate::inventory) fn switch_pages(
    mut chosen: MessageReader<TabChosen>,
    mut open: ResMut<super::OpenGrids>,
    mut commands: Commands,
) {
    for &TabChosen(row, index) in chosen.read() {
        let Some(pages) = open.pages.filter(|pages| pages.row == row) else {
            continue;
        };
        let page = Page::ALL[index];
        pages.show(page, &mut commands);
        open.page = page;
    }
}
