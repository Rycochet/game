//! Giving a new item its form as asked for: `give_form_to`, which every other way of
//! giving a form comes down to.

use super::armour::Piece;
use super::armour_ask::armour_form_to;
use super::gem::{GemAsk, roll_gem};
use super::give::set_blade;
use super::request::{Field, Refused, RollRequest, keep_only};
use super::ring_ask::RingAsk;
use super::ring_roll::{roll_ring, roll_ring_size};
use super::{ItemForm, Limits};
use crate::blade;
use crate::creature::Size;
use crate::id::Id;
use crate::item::{Item, ItemKind, Made};
use crate::rng::Seed;

/// Gives a new item its form, for kinds that have one, going by its quality and
/// rolled from `seed`, keeping to what `request` asks; `RollRequest::NONE` rolls what
/// `give_form_from` does. `size` is what it must be; without one, it is rolled. A
/// blade within limits may become another family's kind. Anything inside it, such as
/// a ring's gems, gets its id from `new_id`. Refused, changing nothing, if the kind
/// doesn't take an option asked for or nothing can be made so.
pub fn give_form_to(
    item: &mut Item,
    size: Option<Size>,
    seed: Seed,
    request: &RollRequest,
    new_id: &mut impl FnMut() -> Id,
) -> Result<(), Refused> {
    if let Some(field) = request.not_for(item.kind) {
        return Err(Refused(field));
    }
    let mut rng = seed.derive("item form").rng();
    let quality = item.quality.unwrap_or_default();
    let (form, rolled_size) = match item.kind {
        kind if let Some(piece) = Piece::of(kind) => {
            let form = match *request == RollRequest::NONE {
                true => piece.roll(&mut rng, quality, None),
                false => armour_form_to(piece, &mut rng, quality, request)?,
            };
            (form, size.unwrap_or(Size::Medium))
        }
        ItemKind::Gem => {
            let ask = GemAsk::of(request);
            ask.check(quality, true)?;
            let (gem, size) = roll_gem(&mut rng, quality, true, ask);
            (ItemForm::Gem(gem), size)
        }
        ItemKind::Ring => {
            let ask = RingAsk::of(request);
            ask.check(quality)?;
            let size = size.unwrap_or_else(|| roll_ring_size(&mut rng));
            let made_at = item.made_at;
            let ring = roll_ring(
                &mut rng,
                quality,
                size,
                ask,
                &mut |gem, gem_size, quality| {
                    let mut stone = Item::new(new_id(), ItemKind::Gem);
                    stone.made = Made::for_body(ItemKind::Gem, gem_size, None, false);
                    stone.quality = Some(quality);
                    stone.made_at = made_at;
                    stone.form = Some(ItemForm::Gem(gem));
                    stone
                },
            );
            (ItemForm::Ring(Box::new(ring)), size)
        }
        kind => {
            let Some(family) = kind.blade_family() else {
                return Ok(());
            };
            if !keep_only(&mut blade::blade_metals(quality), request.material) {
                return Err(Refused(Field::Material));
            }
            let limits = Limits {
                metal: request.material,
                ..request.limits.unwrap_or(Limits::NONE)
            };
            if request.limits.is_some() {
                let limits = Limits {
                    wielder: limits.wielder.or(size),
                    ..limits
                };
                let rolled = blade::roll_blade_within(&mut rng, family, quality, &limits);
                set_blade(item, rolled.ok_or(Refused(Field::Limits))?);
                return Ok(());
            }
            let size = size.unwrap_or(Size::Medium);
            let blade = blade::roll_blade_at_within(&mut rng, family, quality, size, &limits)
                .ok_or(Refused(Field::Material))?;
            (ItemForm::Blade(Box::new(blade)), size)
        }
    };
    let size = size.unwrap_or(rolled_size);
    item.made = Made::for_body(item.kind, size, item.made.fingers, item.made.magical);
    item.form = Some(form);
    Ok(())
}
