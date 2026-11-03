//! How a style looks: flutes, edges and the like.

use shared::armour::shoulders::ShouldersForm;
use shared::armour::{Construction, Style};

/// How a style shapes plates.
#[derive(Clone, Copy)]
pub(super) struct Look {
    pub(super) thick: f32,
    /// Flutes over the cop and lames: how many across a half turn, and how deep.
    flutes: (f32, f32),
    /// How far each lame's edge is cut into points, and how deep.
    pub(super) cusps: f32,
    /// A leaf point at the bottom of each lame, and a peak at the top.
    pub(super) leaf: f32,
    /// How flat the cop's top is, out of 1.
    pub(super) squat: f32,
    pub(super) rim: f32,
    pub(super) step: f32,
    /// Heavier rivets, and a band of runes.
    pub(super) runes: bool,
    pub(super) etched: bool,
}

pub(super) fn look_of(style: Style) -> Look {
    let base = Look {
        thick: 2.0,
        flutes: (0.0, 0.0),
        cusps: 0.0,
        leaf: 0.0,
        squat: 1.0,
        rim: 2.2,
        step: 1.6,
        runes: false,
        etched: false,
    };
    match style {
        // Regional styles shape their own constructions, not plate.
        Style::White
        | Style::Japanese
        | Style::Chinese
        | Style::Steppe
        | Style::Classical
        | Style::Byzantine
        | Style::IndoPersian => base,
        Style::Greenwich => Look {
            etched: true,
            ..base
        },
        Style::Milanese => Look { rim: 3.0, ..base },
        Style::Gothic => Look {
            flutes: (7.0, 4.0),
            cusps: 7.0,
            rim: 0.0,
            ..base
        },
        Style::Maximilian => Look {
            flutes: (16.0, 2.6),
            ..base
        },
        Style::Kastenbrust => Look {
            squat: 0.75,
            ..base
        },
        Style::Elven => Look {
            thick: 1.4,
            leaf: 22.0,
            rim: 0.0,
            ..base
        },
        Style::Dwarven => Look {
            thick: 3.2,
            squat: 0.8,
            rim: 3.5,
            step: 4.0,
            runes: true,
            ..base
        },
    }
}

/// How plates are shaped: by style for plate, plainly for anything else.
pub(super) fn shaped(form: &ShouldersForm) -> Look {
    let look = look_of(form.style);
    match form.construction {
        Construction::Plate | Construction::Mirror => look,
        Construction::Leather => Look {
            thick: 4.0,
            flutes: (0.0, 0.0),
            rim: 0.0,
            etched: false,
            ..look
        },
        _ => Look {
            flutes: (0.0, 0.0),
            cusps: 0.0,
            leaf: 0.0,
            rim: 0.0,
            etched: false,
            runes: false,
            ..look
        },
    }
}

pub(super) fn flute(look: Look, angle: f32) -> f32 {
    let (count, deep) = look.flutes;
    if count == 0.0 {
        0.0
    } else {
        deep * (angle * count).cos().max(0.0).powi(2)
    }
}

/// How far a lame's bottom edge comes down at this angle, for cusps and leaf points.
pub(super) fn edge_drop(look: Look, angle: f32) -> f32 {
    look.cusps * (angle * 6.0).sin().abs() + look.leaf * angle.cos().max(0.0).powi(6)
}
