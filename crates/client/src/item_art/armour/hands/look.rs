//! How a style and cuff look: flare, fluting and the like.

use shared::armour::hands::{Cuff, HandsForm};
use shared::armour::{Construction, Style};

/// How a style shapes plates.
pub(super) struct Look {
    pub(super) thick: f32,
    /// Flutes round a cuff and across the back, and how deep.
    flutes: (f32, f32),
    /// How far the cuff's top is drawn out to a point on the outside.
    pub(super) point: f32,
    pub(super) cusps: bool,
    pub(super) rim: f32,
    pub(super) square: bool,
    pub(super) pointed_tips: bool,
}

pub(super) fn look_of(style: Style, cuff: Cuff) -> Look {
    let base = Look {
        thick: 2.0,
        flutes: (0.0, 0.0),
        point: if cuff == Cuff::Pointed { 40.0 } else { 0.0 },
        cusps: false,
        rim: 2.2,
        square: false,
        pointed_tips: false,
    };
    match style {
        // Regional styles shape their own constructions, not plate.
        Style::White
        | Style::Greenwich
        | Style::Japanese
        | Style::Chinese
        | Style::Steppe
        | Style::Classical
        | Style::Byzantine
        | Style::IndoPersian => base,
        Style::Milanese => Look { rim: 2.8, ..base },
        Style::Gothic => Look {
            flutes: (6.0, 3.0),
            point: base.point.max(25.0),
            cusps: true,
            rim: 0.0,
            pointed_tips: true,
            ..base
        },
        Style::Maximilian => Look {
            flutes: (12.0, 2.5),
            ..base
        },
        Style::Kastenbrust => Look {
            square: true,
            ..base
        },
        Style::Elven => Look {
            thick: 1.4,
            point: base.point.max(45.0),
            rim: 0.0,
            pointed_tips: true,
            ..base
        },
        Style::Dwarven => Look {
            thick: 3.2,
            rim: 3.5,
            ..base
        },
    }
}

/// Whether plates are drawn with the style's shapes: plate, and the plates of the
/// others that have them.
pub(super) fn styled(form: &HandsForm) -> bool {
    matches!(
        form.construction,
        Construction::Plate | Construction::PlatesInCloth | Construction::MailAndPlate
    )
}

pub(super) fn flute(look: &Look) -> impl Fn(f32) -> f32 + '_ {
    move |angle: f32| {
        let (count, deep) = look.flutes;
        if count == 0.0 {
            0.0
        } else {
            deep * (angle * count).cos().max(0.0).powi(2)
        }
    }
}
