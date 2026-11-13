//! Gate codes: a gate as players see it, in runes rather than letters.
//!
//! A code is `CODE_LENGTH` runes of the 32 in `RUNES`, 5 bits each, most
//! significant first: the gate's address (32 bits: where it leads, so
//! neighbouring gates' codes are unalike) with its design (3 bits) above it.
//! The design is worked out from the gate's place, so it also checks the
//! code: changing any rune gives a code whose design doesn't match 7 times in
//! 8.

use core::fmt;

use serde::{Deserialize, Serialize};

use super::{GateAddress, GatePlace};
use crate::world::WorldSeed;

/// The runes codes are written in: the Anglo-Saxon futhorc in its usual
/// order, without æsc (ᚫ), the easiest to mistake for ac (ᚪ).
pub const RUNES: [char; 32] = [
    '\u{16A0}', // ᚠ feoh
    '\u{16A2}', // ᚢ ur
    '\u{16A6}', // ᚦ þorn
    '\u{16A9}', // ᚩ os
    '\u{16B1}', // ᚱ rad
    '\u{16B3}', // ᚳ cen
    '\u{16B7}', // ᚷ gyfu
    '\u{16B9}', // ᚹ wynn
    '\u{16BB}', // ᚻ hægl
    '\u{16BE}', // ᚾ nyd
    '\u{16C1}', // ᛁ is
    '\u{16C4}', // ᛄ ger
    '\u{16C7}', // ᛇ eoh
    '\u{16C8}', // ᛈ peorð
    '\u{16C9}', // ᛉ eolh
    '\u{16CB}', // ᛋ sigel
    '\u{16CF}', // ᛏ tiw
    '\u{16D2}', // ᛒ beorc
    '\u{16D6}', // ᛖ eh
    '\u{16D7}', // ᛗ mann
    '\u{16DA}', // ᛚ lagu
    '\u{16DD}', // ᛝ ing
    '\u{16DF}', // ᛟ ēþel
    '\u{16DE}', // ᛞ dæg
    '\u{16AA}', // ᚪ ac
    '\u{16A3}', // ᚣ yr
    '\u{16E1}', // ᛡ ior
    '\u{16E0}', // ᛠ ear
    '\u{16B8}', // ᚸ gar
    '\u{16E3}', // ᛣ calc
    '\u{16E2}', // ᛢ cweorð
    '\u{16E5}', // ᛥ stan
];

/// Runes in a code.
pub const CODE_LENGTH: usize = 7;
/// Shown between a code's first 3 runes and its last 4 (the runic single
/// punctuation dot), and ignored when one is read.
pub const SEPARATOR: char = '\u{16EB}';
const SHOWN_BEFORE_SEPARATOR: usize = 3;
const BITS_PER_RUNE: u32 = 5;

/// A gate code, as each rune's index in `RUNES`. Any 7 runes make one, but
/// only those whose design matches lead anywhere (`place`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GateCode(pub [u8; CODE_LENGTH]);

impl GateCode {
    /// A planet's own name in runes (owner: from its seed): the code of its
    /// first World Gate. `None` if it isn't one of the Hall's planets.
    pub fn of_planet(world: WorldSeed) -> Option<Self> {
        GateAddress::for_world(world, 0).map(|address| Self::of(address.place()))
    }

    pub fn of(place: GatePlace) -> Self {
        let value = (u64::from(place.design().index()) << 32) | u64::from(place.address().bits());
        let mut runes = [0; CODE_LENGTH];
        for (i, rune) in runes.iter_mut().enumerate() {
            let shift = BITS_PER_RUNE * (CODE_LENGTH - 1 - i) as u32;
            *rune = ((value >> shift) & 31) as u8;
        }
        Self(runes)
    }

    /// The gate this code names, or `None` if its design doesn't match (a
    /// mistake).
    pub fn place(self) -> Option<GatePlace> {
        let value = self.0.iter().fold(0u64, |value, &rune| {
            (value << BITS_PER_RUNE) | u64::from(rune & 31)
        });
        let place = GateAddress::from_bits(value as u32).place();
        (u64::from(place.design().index()) == value >> 32).then_some(place)
    }

    /// Reads a code as typed: `CODE_LENGTH` of `RUNES`, ignoring spaces and
    /// `SEPARATOR`s. `None` for anything else.
    pub fn parse(text: &str) -> Option<Self> {
        let mut runes = [0; CODE_LENGTH];
        let mut count = 0;
        for c in text
            .chars()
            .filter(|&c| c != SEPARATOR && !c.is_whitespace())
        {
            let index = RUNES.iter().position(|&rune| rune == c)?;
            *runes.get_mut(count)? = index as u8;
            count += 1;
        }
        (count == CODE_LENGTH).then_some(Self(runes))
    }
}

impl fmt::Display for GateCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, &rune) in self.0.iter().enumerate() {
            if i == SHOWN_BEFORE_SEPARATOR {
                write!(f, "{SEPARATOR}")?;
            }
            write!(f, "{}", RUNES[usize::from(rune & 31)])?;
        }
        Ok(())
    }
}
