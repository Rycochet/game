//! A character standing in a tall picture, to one scale for every playable kind, so
//! a short kind looks short beside a tall one (choosing a character).

use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::sheet::{Appearance, HEIGHT_SHARES, HEIGHT_WHOLE};
use shared::wear::WornLook;

use super::super::Look;
use super::character_look;

/// A portrait is two cells wide and three long.
pub(in crate::item_art) const PORTRAIT_CELLS: (u8, u8) = (2, 3);

/// The share of the picture's height the tallest playable kind stands, leaving room
/// over its head for hair, horns and crests.
const TALLEST_SHARE: f32 = 0.85;

/// How a playable creature looks standing, to scale: as `character_look`, in a
/// portrait. `None` for a kind with no frame to wear things on (animals).
pub fn portrait_look(
    kind: CreatureKind,
    appearance: &Appearance,
    worn: &[WornLook],
) -> Option<Look> {
    match character_look(kind, appearance, worn)? {
        character @ Look::Character { .. } => Some(Look::Portrait(Box::new(character))),
        _ => None,
    }
}

/// How big a millimetre of this kind's model is, as a share of its picture's height,
/// for one standing `height` thousandths of its kind's usual height. Every frame is a
/// person's height, so it is shrunk to the kind's own, then to its own Height; the
/// tallest playable kind at its tallest fills `TALLEST_SHARE`.
pub(in crate::item_art) fn portrait_scale(kind: CreatureKind, height: i32) -> f32 {
    let tallest = CreatureKind::ALL
        .into_iter()
        .filter(|kind| kind.def().playable)
        .map(Frame::usual_height)
        .max()
        .unwrap_or(1) as f32
        * tallest_share();
    let frame = Frame::usual_height(CreatureKind::Human) as f32;
    let own = Frame::usual_height(kind) as f32 * height as f32 / HEIGHT_WHOLE as f32;
    TALLEST_SHARE * own / (tallest * frame)
}

/// The tallest Height there is, as a share of the usual.
fn tallest_share() -> f32 {
    HEIGHT_SHARES.into_iter().max().unwrap_or(HEIGHT_WHOLE) as f32 / HEIGHT_WHOLE as f32
}
