//! Rolling what a blade is made of and how it is decorated, finer with better quality.

use super::Holding;
use crate::blade::Decoration;
use crate::item::Quality;
use crate::material::Material;
use crate::rng::Rng;

/// What is rolled for a blade's materials and decoration.
pub(super) struct MaterialRolls {
    pub(super) blade_metal: Material,
    pub(super) handle_material: Material,
    pub(super) fittings: Material,
    pub(super) decoration: Decoration,
}

pub(super) fn roll_materials(rng: &mut Rng, quality: Quality, holding: Holding) -> MaterialRolls {
    let held = holding.held;
    // Better work goes into finer things, more often.
    let fine = quality as u64;
    let blade_metal = rng.pick_weighted(&[
        (Material::Iron, 80),
        (Material::Copper, 10),
        (Material::Silver, 2 + fine * 3),
        (Material::Gold, fine),
    ]);
    let handle_material = if held {
        rng.pick_weighted(&[
            (Material::Wood, 60),
            (Material::Bone, 25),
            (Material::Iron, 15),
        ])
    } else {
        Material::Wood
    };
    let fittings = rng.pick_weighted(&[
        (Material::Iron, 60),
        (Material::Copper, 25),
        (Material::Silver, 5 + fine * 5),
        (Material::Gold, 1 + fine * 3),
    ]);
    let decoration = rng.pick_weighted(&[
        (Decoration::Plain, 70),
        (Decoration::Engraved, 10 + fine * 8),
        (Decoration::Inlaid, 2 + fine * 5),
        (Decoration::Tasselled, 5),
    ]);
    MaterialRolls {
        blade_metal,
        handle_material,
        fittings,
        decoration,
    }
}
