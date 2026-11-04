//! What can be seen of what a creature wears: the pieces a character's picture is
//! drawn with, which others are told of.

use serde::{Deserialize, Serialize};

use super::coverage;
use crate::item::{Inventory, Item, Quality};
use crate::item_form::ItemForm;

/// One worn thing as others see it: its form, how well made it looks, and the
/// fingers it is made for. Nothing secret: no id, magic or curses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WornLook {
    pub form: ItemForm,
    pub quality: Option<Quality>,
    pub fingers: Option<u8>,
}

impl ItemForm {
    /// Whether a character's picture draws it: armour and footwear, built in place on
    /// the body. Clothes without a form, rings and the like aren't drawn yet.
    pub fn drawn_worn(&self) -> bool {
        matches!(
            self,
            Self::Hands(_)
                | Self::Shoulders(_)
                | Self::Torso(_)
                | Self::Head(_)
                | Self::Arms(_)
                | Self::Legs(_)
                | Self::Waist(_)
                | Self::Feet(_)
                | Self::Neck(_)
                | Self::Footwear(_)
                | Self::Set(_)
        )
    }
}

/// Its form, if a picture draws it.
fn drawn(item: &Item) -> Option<&ItemForm> {
    item.form.as_ref().filter(|form| form.drawn_worn())
}

impl Inventory {
    /// What a picture of this creature shows of what it wears, innermost first:
    /// everything drawn, except what something else drawn covers at every place it
    /// is (a gambeson under a harness). A surcoat isn't drawn yet, so what it covers
    /// still shows.
    pub fn shown(&self) -> Vec<WornLook> {
        let mut shown: Vec<&Item> = self
            .worn
            .iter()
            .filter(|item| drawn(item).is_some())
            .filter(|item| {
                let places = coverage(item);
                !places.iter().all(|&(location, layer)| {
                    self.worn.iter().any(|other| {
                        other.id != item.id
                            && drawn(other).is_some()
                            && coverage(other).iter().any(|&(theirs, their_layer)| {
                                theirs == location && their_layer > layer
                            })
                    })
                })
            })
            .collect();
        shown.sort_by_key(|item| item.kind.def().wear.map(|wear| wear.layer));
        shown
            .into_iter()
            .filter_map(|item| {
                Some(WornLook {
                    form: drawn(item)?.clone(),
                    quality: item.quality,
                    fingers: item.made.fingers,
                })
            })
            .collect()
    }
}
