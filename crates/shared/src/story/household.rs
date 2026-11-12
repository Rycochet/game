//! A household: a family name shared by everyone in it, and each member's
//! given name and gender, all from the household's seed.

use serde::{Deserialize, Serialize};

use super::{family_names::family_name, is_people, pet::pet_name};
use crate::creature::CreatureKind;
use crate::gender::Gender;
use crate::lang::{Args, Lang, RenderError};
use crate::rng::Seed;

/// One family in a town: what kind its people are and the name they share.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Household {
    pub kind: CreatureKind,
    /// `None` for animals, which have no families of their own.
    pub family: Option<String>,
    seed: Seed,
}

/// Someone in a story, named as their people name them. Sent to clients as a
/// story's cast, so it is serialisable.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Person {
    pub kind: CreatureKind,
    pub gender: Gender,
    pub given: String,
    pub family: Option<String>,
}

impl Household {
    /// The household for `seed` (one of its own, derived by whoever places the
    /// household: never a poi's map seed).
    pub fn new(kind: CreatureKind, seed: Seed) -> Self {
        let family = is_people(kind).then(|| family_name(kind, seed.derive("family name")));
        Self { kind, family, seed }
    }

    /// Its `index`th member: the same index always gives the same person.
    /// Genders are rolled as mobs' are (`Gender::rolled`); animals get pets'
    /// names.
    pub fn member(&self, index: u32) -> Person {
        let own = Seed::new(self.seed.derive("member").at(&[i64::from(index)]));
        let mut rng = own.derive("given name").rng();
        let given = if is_people(self.kind) {
            self.kind.random_name(&mut rng)
        } else {
            pet_name(&mut rng)
        };
        Person {
            kind: self.kind,
            gender: Gender::rolled(own.derive("gender").value()),
            given,
            family: self.family.clone(),
        }
    }
}

impl Person {
    /// Their whole name in `lang`'s order ("Davin Ashford"), or the given name
    /// alone when they have no family name.
    pub fn full_name(&self, lang: &Lang) -> Result<String, RenderError> {
        match &self.family {
            Some(family) => lang.render(
                "person-full-name",
                &Args::new()
                    .name("given-name", &self.given)
                    .name("family-name", family),
            ),
            None => Ok(self.given.clone()),
        }
    }
}
