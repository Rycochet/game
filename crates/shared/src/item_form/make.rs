//! Making a new item to order, the same way everywhere: clothes for a new character
//! or a mob, and Give Item, whose preview on the client and the item the server makes
//! come from the same seed, so they agree.

use crate::creature::{CreatureKind, Size};
use crate::id::Id;
use crate::item::{Favour, Item, ItemKind, Made, MadeAt, Quality, artifact_name};
use crate::item_form::give_form_from;
use crate::rng::Seed;

/// What is asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemOrder {
    pub kind: ItemKind,
    /// `None` for the wearer's own, for what is worn; kinds that don't come in sizes
    /// are `Medium` whatever is asked for.
    pub size: Option<Size>,
    pub magical: bool,
    /// Ignored for kinds without quality.
    pub quality: Quality,
    /// Part of a set, if it has quality.
    pub set: bool,
    /// Blessed or cursed, only for what is worn.
    pub favour: Option<Favour>,
}

impl ItemOrder {
    /// An ordinary one of this kind and quality.
    pub fn plain(kind: ItemKind, quality: Quality) -> Self {
        Self {
            kind,
            size: None,
            magical: false,
            quality,
            set: false,
            favour: None,
        }
    }
}

/// The seed of the `n`th of a batch of different items from one seed: the first is
/// the seed itself, so a preview of the first is what the batch starts with.
pub fn batch_seed(seed: u64, n: u32) -> Seed {
    Seed::new(seed.wrapping_add(n as u64))
}

/// A new item as ordered, made for `wearer`, with everything about it rolled from
/// `seed`. Anything inside it, such as a ring's gem, gets its id from `new_id`.
pub fn make_item(
    order: &ItemOrder,
    wearer: CreatureKind,
    id: Id,
    seed: Seed,
    made_at: Option<MadeAt>,
    new_id: &mut impl FnMut() -> Id,
) -> Item {
    let kind = order.kind;
    let worn = kind.def().wear.is_some();
    // Asking for magic on what is too plain for it would quietly give none.
    let quality = match order.magical && worn {
        true => order.quality.max(Quality::Uncommon),
        false => order.quality,
    };
    let body = wearer.def();
    let size = order.size.filter(|_| kind.comes_in_sizes());
    // What is worn is made to fit the wearer unless told otherwise.
    let size = size.or(kind.def().wear.map(|_| body.size));
    let quality = kind.def().has_quality.then_some(quality);
    let name = (quality == Some(Quality::Artifact)).then(|| artifact_name(&mut seed.rng(), kind));
    let made = Made::for_body(
        kind,
        size.unwrap_or_default(),
        body.fingers(),
        order.magical,
    );
    let mut item = Item {
        made,
        quality,
        made_at,
        name,
        ..Item::new(id, kind)
    };
    give_form_from(&mut item, size, seed, new_id);
    item.set = order.set && item.quality.is_some();
    item.made.magical = order.magical && item.can_be_magical();
    item.favour = order.favour.filter(|_| worn);
    item
}
