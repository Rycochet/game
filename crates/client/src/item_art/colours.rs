//! The colours a creature's body is drawn in, from how it looks.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::sheet::Appearance;

/// Skin (or coat) and feathers, as red, green and blue from 0 to 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Colours {
    pub skin: [u8; 3],
    pub feathers: [u8; 3],
}

impl Colours {
    /// A plain body, as a tailor's dummy, with white feathers so wings show apart from
    /// the arms.
    pub const PLAIN: Self = Self {
        skin: [199, 184, 163],
        feathers: [242, 242, 238],
    };

    /// A creature of this kind looking like this: plain where it has no choice. A body
    /// covered in feathers alone (a bird's) is drawn in its feathers' colour.
    pub fn of(kind: CreatureKind, appearance: &Appearance) -> Self {
        Self {
            skin: appearance
                .skin(kind)
                .or_else(|| appearance.feathers(kind))
                .unwrap_or(Self::PLAIN.skin),
            feathers: appearance.feathers(kind).unwrap_or(Self::PLAIN.feathers),
        }
    }
}

/// A material in one of these colours, as bodies are drawn.
pub(super) fn material(rgb: [u8; 3]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb_u8(rgb[0], rgb[1], rgb[2]),
        perceptual_roughness: 0.8,
        ..default()
    }
}
