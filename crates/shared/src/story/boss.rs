//! Named bosses: a name in their own people's sounds (beasts' harsher ones)
//! and an epithet each language words.

use serde::{Deserialize, Serialize};

use super::family_names::pick;
use super::{Epithet, is_people};
use crate::creature::{CreatureKind, three_alike};
use crate::gender::Gender;
use crate::lang::{Args, Lang, RenderError};
use crate::rng::Seed;

/// A creature leading others, with a name of its own.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Boss {
    pub kind: CreatureKind,
    pub gender: Gender,
    pub name: String,
    pub epithet: Epithet,
}

/// Beasts' names: growls rather than any people's sounds.
const BEAST_STARTS: &[&str] = &[
    "Gr", "Skar", "Mor", "Hrak", "Vor", "Grim", "Kral", "Ur", "Bael", "Thrag",
];
const BEAST_MIDDLES: &[&str] = &["a", "o", "u", ""];
const BEAST_ENDS: &[&str] = &["ak", "oth", "ul", "rax", "gor", "mok", "fang", "nar"];

impl Boss {
    /// A boss of `kind`, from a seed of its own.
    pub fn new(kind: CreatureKind, seed: Seed) -> Self {
        let mut rng = seed.derive("boss name").rng();
        let name = if is_people(kind) {
            kind.random_name(&mut rng)
        } else {
            let mut name = String::new();
            for _ in 0..8 {
                name = [
                    pick(&mut rng, BEAST_STARTS),
                    pick(&mut rng, BEAST_MIDDLES),
                    pick(&mut rng, BEAST_ENDS),
                ]
                .concat();
                if !three_alike(&name) {
                    break;
                }
            }
            name
        };
        let epithets = Epithet::ALL;
        let which = seed
            .derive("epithet")
            .rng()
            .range_u64(0, epithets.len() as u64);
        Self {
            kind,
            gender: Gender::rolled(seed.derive("gender").value()),
            name,
            epithet: epithets[which as usize],
        }
    }

    /// Its name with its epithet, in `lang` ("Skarnak the Grey").
    pub fn full_name(&self, lang: &Lang) -> Result<String, RenderError> {
        let epithet = lang.phrase(&self.epithet)?;
        lang.render(
            "boss-full-name",
            &Args::new()
                .name("name", &self.name)
                .text("epithet", epithet),
        )
    }
}
