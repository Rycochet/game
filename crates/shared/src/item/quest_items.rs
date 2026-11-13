//! Quest items (owner): each carries the quest it belongs to (`Item::quest`).
//! They are worth nothing else: never given or traded, destroyed rather than
//! dropped, and gone wherever they are once their quest fails, is given up or
//! is handed in.

use crate::item::{Inventory, Item, ItemKind};
use crate::quest::QuestId;

impl Item {
    /// Whether it belongs to a quest.
    pub fn is_quest_item(&self) -> bool {
        self.quest.is_some()
    }

    /// Whether it, or anything in it or stacked on it, belongs to a quest.
    pub fn holds_quest_item(&self) -> bool {
        self.is_quest_item()
            || self
                .contents
                .iter()
                .any(|placed| placed.item.holds_quest_item())
            || self.above.iter().any(Item::holds_quest_item)
    }

    /// Takes every quest item out of what is in it, at any depth. How many
    /// went.
    pub fn strip_quest_items(&mut self) -> usize {
        let before = self.contents.len();
        self.contents.retain(|placed| !placed.item.is_quest_item());
        let mut gone = before - self.contents.len();
        for placed in &mut self.contents {
            gone += placed.item.strip_quest_items();
        }
        gone
    }

    /// Whether it is an item of `quest`, of `kind` if one is given.
    fn of_quest(&self, quest: QuestId, kind: Option<ItemKind>) -> bool {
        self.quest == Some(quest) && kind.is_none_or(|kind| self.kind == kind)
    }

    /// Takes the items of `quest` (of `kind`, if given) out of what is in
    /// it, at any depth. How many went.
    fn remove_quest_items(&mut self, quest: QuestId, kind: Option<ItemKind>) -> usize {
        let before = self.contents.len();
        self.contents
            .retain(|placed| !placed.item.of_quest(quest, kind));
        let mut gone = before - self.contents.len();
        for placed in &mut self.contents {
            gone += placed.item.remove_quest_items(quest, kind);
        }
        gone
    }
}

impl Inventory {
    /// Every item of `quest` it holds, wherever: carried, in bags, worn or in
    /// the bin. How many went.
    pub fn remove_quest_items(&mut self, quest: QuestId) -> usize {
        self.remove_quest_kind(quest, None)
    }

    /// As `remove_quest_items`, only those of `kind` if one is given: a
    /// delivery's parcels, say.
    pub fn remove_quest_kind(&mut self, quest: QuestId, kind: Option<ItemKind>) -> usize {
        let before = self.items.len() + self.worn.len();
        self.items
            .retain(|placed| !placed.item.of_quest(quest, kind));
        self.worn.retain(|item| !item.of_quest(quest, kind));
        let mut gone = before - self.items.len() - self.worn.len();
        for placed in &mut self.items {
            gone += placed.item.remove_quest_items(quest, kind);
        }
        for item in &mut self.worn {
            gone += item.remove_quest_items(quest, kind);
        }
        if self
            .bin
            .as_ref()
            .is_some_and(|item| item.of_quest(quest, kind))
        {
            self.bin = None;
            gone += 1;
        } else if let Some(bin) = &mut self.bin {
            gone += bin.remove_quest_items(quest, kind);
        }
        gone
    }

    /// How many items of `quest` it carries (worn and in bags too, not the
    /// bin), counting stacks.
    pub fn quest_items(&self, quest: QuestId) -> u32 {
        self.quest_kind(quest, None)
    }

    /// As `quest_items`, only those of `kind` if one is given.
    pub fn quest_kind(&self, quest: QuestId, kind: Option<ItemKind>) -> u32 {
        fn count(item: &Item, quest: QuestId, kind: Option<ItemKind>) -> u32 {
            let own = if item.of_quest(quest, kind) {
                item.count()
            } else {
                0
            };
            own + (item.contents.iter())
                .map(|placed| count(&placed.item, quest, kind))
                .sum::<u32>()
        }
        let carried = (self.items.iter()).map(|placed| count(&placed.item, quest, kind));
        let worn = self.worn.iter().map(|item| count(item, quest, kind));
        carried.chain(worn).sum()
    }
}
