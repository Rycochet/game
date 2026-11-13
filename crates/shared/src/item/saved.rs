//! Reading an `Item`: the same fields, then any counted kind's old stack (from
//! before counted stacks, 2026-11-12: one item with an id for each coin) folded
//! into its count. Works the same for messages, which never have such stacks.

use serde::Deserialize;

use crate::id::Id;
use crate::item::item::one;
use crate::item::{Favour, Item, ItemKind, Known, Made, MadeAt, Placed, Quality};
use crate::item_form::ItemForm;
use crate::quest::QuestId;

/// An `Item` as written: see `Item` for each field.
#[derive(Deserialize)]
pub(super) struct Saved {
    id: Id,
    kind: ItemKind,
    contents: Vec<Placed>,
    made: Made,
    #[serde(default)]
    quality: Option<Quality>,
    #[serde(default)]
    made_at: Option<MadeAt>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    set: bool,
    #[serde(default)]
    form: Option<ItemForm>,
    #[serde(default)]
    favour: Option<Favour>,
    #[serde(default)]
    known: Known,
    #[serde(default)]
    above: Vec<Item>,
    #[serde(default = "one")]
    count: u32,
    #[serde(default)]
    quest: Option<QuestId>,
}

impl From<Saved> for Item {
    fn from(saved: Saved) -> Self {
        let mut item = Item {
            id: saved.id,
            kind: saved.kind,
            contents: saved.contents,
            made: saved.made,
            quality: saved.quality,
            made_at: saved.made_at,
            name: saved.name,
            set: saved.set,
            form: saved.form,
            favour: saved.favour,
            known: saved.known,
            above: saved.above,
            count: saved.count,
            quest: saved.quest,
        };
        if item.kind.def().counted {
            // The bottom's id and `made_at` stay the stack's.
            let above: u32 = item.above.drain(..).map(|one| one.count).sum();
            item.count = item.count.max(1) + above;
        } else {
            item.count = 1;
        }
        item
    }
}
