//! What a "Collect" step asks to have brought back (owner): a part natural to
//! the creature, only where its body has it (fingernails with hands, toenails
//! on a person's feet, hooves, horns, a tail), from the poi's own seed so
//! every town's quest at the poi asks the same.

use crate::body::{Feet, Horns, Parts, Snout, Wings, has_hands};
use crate::creature::CreatureKind;
use crate::poi::Poi;
use crate::story::{BodyPart, Remains};

/// The parts a kind of creature has that a quest may ask for, with how
/// likely each is (Claude's first guesses).
pub(super) fn natural_parts(kind: CreatureKind) -> Vec<(BodyPart, u64)> {
    let parts = Parts::of(kind);
    let muzzle = matches!(parts.snout, Snout::Muzzle | Snout::Long);
    let mut found = vec![(BodyPart::Heart, 2), (BodyPart::Liver, 1)];
    let mut has = |part: BodyPart, weight: u64, yes: bool| {
        if yes {
            found.push((part, weight));
        }
    };
    has(BodyPart::Tooth, 2, !muzzle && parts.snout != Snout::Beak);
    has(BodyPart::Fang, 3, muzzle);
    has(
        BodyPart::Claw,
        3,
        matches!(parts.feet, Feet::Digitigrade | Feet::Talons),
    );
    has(
        BodyPart::Fingernail,
        2,
        has_hands(kind) && parts.wings == Wings::None,
    );
    has(BodyPart::Toenail, 2, parts.feet == Feet::Plantigrade);
    has(BodyPart::Tail, 3, parts.tail > 0);
    has(BodyPart::Horn, 3, parts.horns != Horns::None);
    has(BodyPart::Tusk, 3, parts.tusks > 0);
    has(BodyPart::Ear, 2, parts.ears > 0);
    has(BodyPart::Feather, 3, parts.wings != Wings::None);
    has(BodyPart::Hoof, 3, parts.feet == Feet::Hooves);
    has(
        BodyPart::Scale,
        3,
        matches!(kind, CreatureKind::Kobold | CreatureKind::Lizardfolk),
    );
    found
}

/// What the poi's "Collect" step of `kind` brings back.
pub(super) fn remains_of(poi: &Poi, kind: CreatureKind) -> Remains {
    let parts = natural_parts(kind);
    let part = poi.seed.derive("quest remains").rng().pick_weighted(&parts);
    Remains { kind, part }
}
