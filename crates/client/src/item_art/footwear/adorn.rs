//! Decoration: broguing, embroidery, beads, slashing, stitching, pompoms, tassels,
//! fringe, carving and jewels.

use bevy::prelude::*;
use shared::footwear::{Decoration, Top};

use super::shoe::Shoe;
use super::upper::{add_shaft, on_shaft, sheet};
use crate::item_art::armour::surface::{lines, patch};
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::{blob, limb};

pub(super) fn add_decoration(model: &mut Model, shoe: &Shoe) {
    let thick = shoe.thick();
    let last = shoe.last(shoe.lift() + thick);
    let (toes, length) = (shoe.foot.toes(), shoe.length());
    let tip = last.tip();
    let vamp = last.patch((toes - 0.32 * length, toes - 0.04 * length), 0.75);
    let top = shoe.top();
    match shoe.form.decoration {
        Decoration::None | Decoration::Plaited => {}
        Decoration::Broguing => {
            for at in 0..13 {
                let round = -1.3 + 2.6 * at as f32 / 12.0;
                let z = toes - 0.2 * length + 0.05 * length * (round * 1.6).cos();
                let (point, out) = last.point_out(z, round);
                blob(model.of(Stuff::Shade), point + out * 0.6, Vec3::splat(1.3));
            }
        }
        Decoration::Embroidery | Decoration::Beading => {
            let stuff = Stuff::Field;
            if shoe.form.decoration == Decoration::Embroidery {
                patch(model, stuff, &vamp, (0.15, 0.85), (0.1, 0.9), 0.4, 0.8);
            } else {
                for row in 0..4 {
                    for column in 0..5 {
                        let (point, out) =
                            vamp((column as f32 + 0.5) / 5.0, (row as f32 + 0.5) / 4.0);
                        blob(model.of(stuff), point + out * 1.2, Vec3::splat(1.8));
                    }
                }
            }
            if shoe.form.boot() {
                let middle = (shoe.foot.frame.ankle as f32 + top) / 2.0;
                add_shaft(
                    model,
                    shoe,
                    stuff,
                    (middle - 10.0, middle + 10.0),
                    thick + 0.4,
                    0.8,
                );
            }
        }
        Decoration::Slashing => {
            let toe = last.patch((toes - 0.3 * length, tip - 4.0), 1.4);
            for stripe in 0..6 {
                let u = (stripe as f32 + 0.5) / 6.0;
                patch(
                    model,
                    Stuff::Cloth,
                    &toe,
                    (u - 0.03, u + 0.03),
                    (0.0, 1.0),
                    0.5,
                    1.2,
                );
            }
        }
        Decoration::Stitching => {
            lines(model, &vamp, 5, 0.5);
            if shoe.form.boot() {
                let mut y = top - 20.0;
                while y > top - 80.0 && y > shoe.foot.frame.ankle as f32 {
                    add_shaft(
                        model,
                        shoe,
                        Stuff::Shade,
                        (y - 0.8, y + 0.8),
                        thick + 0.3,
                        0.6,
                    );
                    y -= 14.0;
                }
            }
        }
        Decoration::Pompom => {
            let point = last.at(tip - 4.0, 0.0);
            blob(
                model.of(Stuff::Bone),
                point + Vec3::Y * 8.0,
                Vec3::splat(15.0),
            );
        }
        Decoration::Tassels => {
            let (point, out) = if shoe.form.top == Top::Notch {
                on_shaft(shoe, top - 0.14 * length, 0.0, 1.0)
            } else {
                last.point_out(shoe.throat() + 8.0, 0.0)
            };
            for side in [-6.0, 6.0] {
                let knot = point + out * 3.0 + Vec3::X * side;
                limb(
                    model.of(Stuff::Bone),
                    &[(knot, 1.5), (knot - Vec3::Y * 18.0, 3.5)],
                );
            }
        }
        Decoration::Fringe => {
            for at in 0..9 {
                let round = -1.0 + 2.0 * at as f32 / 8.0;
                let (point, out) = if shoe.form.boot() {
                    on_shaft(shoe, top - 2.0, round, 1.0)
                } else {
                    last.point_out(shoe.throat() + 6.0, round)
                };
                limb(
                    model.of(Stuff::Main),
                    &[(point, 1.2), (point + out * 2.0 - Vec3::Y * 22.0, 1.0)],
                );
            }
        }
        Decoration::Carved => {
            let side_of = last.patch((shoe.foot.heel() + 20.0, toes - 20.0), 2.3);
            for groove in [0.08, 0.16, 0.84, 0.92] {
                patch(
                    model,
                    Stuff::Shade,
                    &side_of,
                    (groove - 0.01, groove + 0.01),
                    (0.1, 0.9),
                    0.4,
                    1.0,
                );
            }
        }
        Decoration::Jewels => {
            for at in [-0.3, 0.0, 0.3] {
                let (point, out) = last.point_out(shoe.throat() + 20.0, at);
                blob(model.of(Stuff::Etch), point + out * 3.0, Vec3::splat(3.2));
            }
        }
    }
    if shoe.form.contrast.is_some()
        && !shoe.form.boot()
        && shoe.form.decoration != Decoration::Slashing
    {
        // A saddle across the instep.
        let saddle = last.patch((shoe.throat() - 30.0, shoe.throat() + 25.0), 2.4);
        sheet(model, Stuff::Cloth, &saddle, 0.8);
    }
}
