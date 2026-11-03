//! How far a field has grown, and what stands in it at each stage.

use shared::site::FieldLook;

/// How far a field has grown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Bare earth, in furrows running north to south.
    Ploughed,
    /// Flat green.
    Grass,
    /// Green, and 6 pixels tall.
    Short,
    /// Turning yellow-green, and 12 pixels tall.
    Tall,
    /// Yellow and ready to harvest, 18 pixels tall.
    Ripe,
}

impl Stage {
    pub(super) const CROPS: [Self; 5] = [
        Self::Ploughed,
        Self::Grass,
        Self::Short,
        Self::Tall,
        Self::Ripe,
    ];
    const PASTURE: [Self; 2] = [Self::Grass, Self::Short];

    /// The stage of a field. Pasture is only ever kept grazed.
    pub fn of(field: &FieldLook) -> Self {
        let choices: &[Self] = if field.livestock {
            &Self::PASTURE
        } else {
            &Self::CROPS
        };
        choices[field.variety as usize % choices.len()]
    }

    /// What stands in the field: which tile of the crops' picture, and how
    /// many pixels tall.
    pub(super) fn crop(self) -> Option<(f32, f32)> {
        match self {
            Self::Ploughed | Self::Grass => None,
            Self::Short => Some((0.0, 6.0)),
            Self::Tall => Some((1.0, 12.0)),
            Self::Ripe => Some((2.0, 18.0)),
        }
    }
}

/// How many stages a field of crops goes through (`FieldLook::variety`, going
/// round).
pub const CROP_STAGES: usize = Stage::CROPS.len();
