//! The helmets a roll starts from: each named row lists what it allows of every
//! property and the styles it is made in. Unnamed rows only add variety, and their
//! helmets take a generic name. A form takes the name of the first row it fits, so
//! narrower rows come before wider ones that would also fit.

use super::{Back, Bowl, Brim, Face, Skull, Top, Visor};
use crate::armour::{Construction, Style};

/// Whether a row's helmets have cheek pieces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cheeks {
    No,
    Yes,
    Either,
}

impl Cheeks {
    pub(super) fn fits(self, cheeks: bool) -> bool {
        match self {
            Self::No => !cheeks,
            Self::Yes => cheeks,
            Self::Either => true,
        }
    }
}

pub(super) struct Row {
    pub name: Option<&'static str>,
    pub constructions: &'static [Construction],
    pub bowls: &'static [Bowl],
    pub skulls: &'static [Skull],
    pub faces: &'static [Face],
    pub cheeks: Cheeks,
    pub backs: &'static [Back],
    pub brims: &'static [Brim],
    pub tops: &'static [Top],
    pub styles: &'static [Style],
    pub weight: u64,
    /// Its own plates guard the throat: a bevored helm.
    pub bevor: bool,
}

use Back as K;
use Bowl as B;
use Brim as R;
use Cheeks::{Either, No, Yes};
use Construction::{Lamellar, Laminar, Leather, Plate, Woven};
use Face as F;
use Skull as S;
use Style::{
    Byzantine, Chinese, Classical, Dwarven, Elven, Gothic, Greenwich, IndoPersian, Japanese,
    Kastenbrust, Maximilian, Milanese, Steppe, White,
};
use Top as T;

const LATE: &[Style] = &[White, Gothic, Milanese, Kastenbrust];
const RENAISSANCE: &[Style] = &[White, Milanese, Maximilian, Greenwich, Dwarven];
const OPEN_NO_BACK: &[Back] = &[K::None];

/// A row with everything given.
#[allow(clippy::too_many_arguments)]
const fn row(
    name: &'static str,
    constructions: &'static [Construction],
    bowls: &'static [Bowl],
    skulls: &'static [Skull],
    faces: &'static [Face],
    cheeks: Cheeks,
    backs: &'static [Back],
    brims: &'static [Brim],
    tops: &'static [Top],
    styles: &'static [Style],
    weight: u64,
) -> Row {
    Row {
        name: if name.is_empty() { None } else { Some(name) },
        constructions,
        bowls,
        skulls,
        faces,
        cheeks,
        backs,
        brims,
        tops,
        styles,
        weight,
        bevor: false,
    }
}

/// The same, guarding the throat.
const fn bevored(row: Row) -> Row {
    Row { bevor: true, ..row }
}

const R0: &[Brim] = &[R::None];
const T0: &[Top] = &[T::None];
const RAISED: &[Bowl] = &[B::Raised];
const PLATE: &[Construction] = &[Plate];

// One row to a line, as a table reads.
#[rustfmt::skip]
pub(super) const ROWS: &[Row] = &[
    // Europe, early and high medieval.
    row("Skull Cap", PLATE, RAISED, &[S::Low, S::Round], &[F::Open], No, OPEN_NO_BACK, R0, T0, &[White, Gothic, Milanese, Dwarven], 3),
    row("Nasal Helm", &[Plate, Laminar], &[B::Raised, B::Segmented], &[S::Conical], &[F::Nasal], No, OPEN_NO_BACK, R0, T0, &[White, Dwarven, Byzantine], 6),
    row("Spangenhelm", &[Plate, Laminar], &[B::Segmented], &[S::Conical, S::Round], &[F::Open, F::Nasal], Either, OPEN_NO_BACK, R0, &[T::None, T::Knob], &[White, Byzantine, Steppe], 6),
    row("Ridge Helmet", &[Plate, Laminar], &[B::Ridged], &[S::Round], &[F::Open, F::Nasal], Yes, &[K::Guard, K::None], R0, &[T::None, T::Comb], &[Classical, Byzantine], 4),
    row("Spectacle Helm", PLATE, &[B::Ridged, B::Segmented], &[S::Round], &[F::Spectacles], No, OPEN_NO_BACK, R0, &[T::None, T::Comb], &[White, Dwarven], 4),
    row("Sugarloaf Helm", PLATE, RAISED, &[S::Ogival], &[F::Faceplate], No, OPEN_NO_BACK, R0, T0, &[White, Gothic, Kastenbrust], 3),
    row("Great Helm", PLATE, RAISED, &[S::Flat, S::Ogival], &[F::Faceplate], No, OPEN_NO_BACK, R0, &[T::None, T::Crest], &[White, Kastenbrust, Dwarven], 6),
    row("Bascinet", PLATE, RAISED, &[S::Ogival], &[F::Open], No, OPEN_NO_BACK, R0, T0, LATE, 8),
    row("Hounskull", PLATE, RAISED, &[S::Ogival], &[F::Visor(Visor::Pointed)], No, OPEN_NO_BACK, R0, T0, LATE, 6),
    row("Klappvisor Bascinet", PLATE, RAISED, &[S::Ogival], &[F::Visor(Visor::Klapp)], No, OPEN_NO_BACK, R0, T0, &[White, Gothic, Kastenbrust], 3),
    bevored(row("Great Bascinet", PLATE, RAISED, &[S::Round], &[F::Visor(Visor::Rounded)], No, OPEN_NO_BACK, R0, T0, &[White, Milanese, Kastenbrust], 4)),
    bevored(row("Frog-mouth Helm", PLATE, RAISED, &[S::Flat], &[F::Slit], No, OPEN_NO_BACK, R0, T0, &[White, Gothic, Maximilian], 2)),
    row("Kettle Hat", PLATE, &[B::Raised, B::Segmented], &[S::Round, S::Conical], &[F::Open], No, OPEN_NO_BACK, &[R::Wide], &[T::None, T::Comb], &[White, Gothic, Milanese, Kastenbrust, Dwarven], 6),
    // Late medieval and Renaissance.
    row("Sallet", PLATE, RAISED, &[S::Round], &[F::Open, F::Slit, F::Visor(Visor::Klapp)], No, &[K::Tail], R0, &[T::None, T::Comb], &[White, Gothic, Kastenbrust, Elven], 8),
    row("Barbute", PLATE, RAISED, &[S::Round], &[F::Opening], No, OPEN_NO_BACK, R0, &[T::None, T::Comb], &[White, Gothic, Milanese], 6),
    row("Armet", PLATE, RAISED, &[S::Round], &[F::Visor(Visor::Rounded)], No, &[K::Flare], R0, &[T::Comb], &[White, Gothic, Milanese, Greenwich], 7),
    row("Close Helmet", PLATE, RAISED, &[S::Round], &[F::Visor(Visor::Bellows), F::Visor(Visor::Rounded), F::Visor(Visor::Grotesque)], No, &[K::Flare], R0, &[T::Comb, T::HighComb], RENAISSANCE, 8),
    row("Burgonet", PLATE, RAISED, &[S::Round], &[F::Open], Yes, &[K::Flare], &[R::Peak], &[T::Comb, T::HighComb], &[White, Maximilian, Greenwich, Elven], 7),
    row("Morion", PLATE, RAISED, &[S::Round], &[F::Open], Either, OPEN_NO_BACK, &[R::Boat], &[T::HighComb], &[White, Milanese, Greenwich], 6),
    row("Cabasset", PLATE, RAISED, &[S::Ogival], &[F::Open], No, OPEN_NO_BACK, &[R::Narrow], &[T::Stalk], &[White, Milanese, Greenwich], 4),
    row("Lobster-tailed Pot", PLATE, &[B::Raised, B::Segmented], &[S::Round], &[F::Bars, F::Nasal], Yes, &[K::Lames], &[R::Peak], &[T::None, T::Knob], &[White, Maximilian, Greenwich], 5),
    // Asia.
    row("Chichak", PLATE, RAISED, &[S::Conical], &[F::Nasal], Yes, &[K::Lames], &[R::Peak], &[T::Spike], &[IndoPersian], 6),
    row("Turban Helmet", PLATE, RAISED, &[S::Onion], &[F::Open, F::Nasal], No, OPEN_NO_BACK, R0, &[T::Spike], &[IndoPersian], 6),
    row("Kulah Khud", PLATE, RAISED, &[S::Round], &[F::Nasal], No, OPEN_NO_BACK, R0, &[T::Spike, T::Plume], &[IndoPersian], 6),
    row("Lamellar Helmet", &[Lamellar], &[B::Laced], &[S::Round, S::Conical], &[F::Open], No, &[K::Flaps], R0, &[T::Spike, T::Plume], &[Steppe, Chinese, Byzantine], 5),
    row("Steppe Helmet", &[Laminar, Lamellar], &[B::Plates], &[S::Conical, S::Round], &[F::Open, F::Nasal], No, &[K::Flaps], &[R::None, R::Peak], &[T::Spike, T::Plume], &[Steppe, Chinese], 6),
    row("Kabuto", &[Laminar, Plate], &[B::Plates], &[S::Low, S::Round], &[F::Open, F::Mask], No, &[K::Lames], &[R::Peak], T0, &[Japanese], 8),
    row("Jingasa", &[Plate, Leather], &[B::Raised, B::Moulded], &[S::Low], &[F::Open], No, OPEN_NO_BACK, &[R::Wide], T0, &[Japanese], 3),
    // Greece and Rome.
    row("Illyrian Helmet", PLATE, RAISED, &[S::Round], &[F::Opening], No, &[K::Flare], R0, &[T::Comb], &[Classical], 3),
    row("Corinthian Helm", PLATE, RAISED, &[S::Round], &[F::Opening], No, &[K::Flare], R0, &[T::None, T::Crest], &[Classical, Elven], 7),
    row("Attic Helmet", PLATE, RAISED, &[S::Round], &[F::Open], Yes, &[K::Flare], &[R::Peak], &[T::None, T::Crest], &[Classical, Elven], 6),
    row("Chalcidian Helmet", PLATE, RAISED, &[S::Round], &[F::Nasal], Yes, &[K::Flare], R0, &[T::None, T::Crest], &[Classical], 4),
    row("Boeotian Helmet", PLATE, RAISED, &[S::Round], &[F::Open], No, &[K::Flare], &[R::Wide], T0, &[Classical], 3),
    row("Phrygian Helmet", PLATE, RAISED, &[S::Forward], &[F::Open], Yes, &[K::Flare], &[R::Peak], &[T::None, T::Crest], &[Classical], 4),
    row("Pilos Helmet", PLATE, RAISED, &[S::Conical], &[F::Open], No, OPEN_NO_BACK, R0, T0, &[Classical], 3),
    row("Montefortino Helmet", PLATE, RAISED, &[S::Round], &[F::Open], Yes, &[K::Flare], R0, &[T::Knob], &[Classical], 5),
    row("Galea", PLATE, RAISED, &[S::Round], &[F::Open], Yes, &[K::Guard], &[R::Peak], &[T::None, T::Crest], &[Classical], 6),
    row("Mask Helmet", PLATE, RAISED, &[S::Round], &[F::Mask], No, &[K::Guard, K::None], R0, &[T::None, T::Crest], &[Classical], 2),
    row("Boar's Tusk Helmet", &[Leather], &[B::Moulded], &[S::Conical], &[F::Open], Yes, OPEN_NO_BACK, R0, &[T::Knob, T::Crest], &[Classical], 2),
    // Generic: leather and rattan.
    row("", &[Leather], &[B::Moulded], &[S::Round, S::Low], &[F::Open], Either, &[K::None, K::Flare], &[R::None, R::Peak], T0, &[White, Gothic, Elven, Dwarven, Steppe, Chinese, Byzantine], 4),
    row("", &[Woven], &[B::Woven], &[S::Round, S::Low], &[F::Open], No, OPEN_NO_BACK, &[R::None, R::Wide], T0, &[Chinese], 3),
];
