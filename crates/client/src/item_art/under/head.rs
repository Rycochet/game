//! Linen coifs, undercaps, hairnets and nightcaps over the head, and wimples round
//! the throat and chin. They stand clear of the hair, which is drawn under them.

use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::underclothes::{Coif, CoifForm, Wimple, WimpleForm};

use super::shape::Cloth;
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::{limb, trunk_depth};

/// Off the head: clear of hair (6 mm, 9 long).
const OFF_HAIR: f32 = 11.0;
/// A face left open either side of the front.
const FACE: f32 = 1.05;

impl Cloth<'_> {
    pub(super) fn coif(&self, model: &mut Model, coif: &Coif) {
        match coif.form {
            CoifForm::Coif => {
                self.head(model, Stuff::Main, (1.0, 0.55), OFF_HAIR, 0.0);
                self.head(model, Stuff::Main, (0.6, 0.12), OFF_HAIR, FACE);
                self.chin_ties(model);
                self.crown_seam(model);
            }
            CoifForm::Undercap => self.head(model, Stuff::Main, (1.0, 0.45), OFF_HAIR, 0.0),
            CoifForm::Net => {
                self.head(model, Stuff::Main, (1.0, 0.4), OFF_HAIR - 2.0, 0.0);
                // The net's mesh as rows and lines in a darker shade.
                for row in 1..6 {
                    let share = 1.0 - row as f32 * 0.1;
                    self.head(
                        model,
                        Stuff::Shade,
                        (share, share - 0.015),
                        OFF_HAIR - 1.0,
                        0.0,
                    );
                }
                for line in 0..10 {
                    self.meridian(model, line as f32 * TAU / 10.0, OFF_HAIR - 1.0, 0.4);
                }
            }
            CoifForm::BrimmedNightcap => {
                self.head(model, Stuff::Main, (1.0, 0.5), OFF_HAIR, 0.0);
                self.head(model, Stuff::Shade, (0.62, 0.5), OFF_HAIR + 4.0, 0.0);
            }
            CoifForm::PointedNightcap => {
                self.head(model, Stuff::Main, (1.0, 0.5), OFF_HAIR, 0.0);
                self.point(model);
            }
            CoifForm::TiedNightcap => {
                self.head(model, Stuff::Main, (1.0, 0.25), OFF_HAIR, FACE * 0.5);
                self.chin_ties(model);
            }
        }
    }

    /// Ties from below each ear to a knot under the chin.
    fn chin_ties(&self, model: &mut Model) {
        let around = self.around();
        let (chin, high) = (around.chin(), around.head_height());
        let ear = chin + high * 0.3;
        let left = around.head(1.6, ear, OFF_HAIR);
        let right = around.head(TAU - 1.6, ear, OFF_HAIR);
        let knot = around.head(0.0, chin - 8.0, 6.0);
        limb(
            model.of(Stuff::Shade),
            &[(left, 3.0), (knot, 3.5), (right, 3.0)],
        );
    }

    /// The seam over the crown from the brow to the nape.
    fn crown_seam(&self, model: &mut Model) {
        self.meridian(model, 0.0, OFF_HAIR + 1.0, 0.55);
        self.meridian(model, TAU / 2.0, OFF_HAIR + 1.0, 0.55);
    }

    /// A line from the crown down to `to` (a share from the chin up) at this angle.
    fn meridian(&self, model: &mut Model, angle: f32, off: f32, to: f32) {
        let around = self.around();
        let (chin, high) = (around.chin(), around.head_height());
        let points: Vec<_> = (0..=8)
            .map(|step| {
                let share = 0.99 - (0.99 - to) * step as f32 / 8.0;
                (around.head(angle, chin + high * share, off), 1.5)
            })
            .collect();
        limb(model.of(Stuff::Shade), &points);
    }

    /// A long nightcap's point, falling to one side.
    fn point(&self, model: &mut Model) {
        let around = self.around();
        let crown = around.chin() + around.head_height();
        let side = self.frame.head.0 as f32;
        let points = [
            (Vec3::new(0.0, crown - 10.0, 0.0), 45.0),
            (Vec3::new(side * 0.4, crown + 35.0, -10.0), 30.0),
            (Vec3::new(side * 1.1, crown + 20.0, -25.0), 18.0),
            (Vec3::new(side * 1.5, crown - 60.0, -30.0), 10.0),
        ];
        limb(model.of(Stuff::Main), &points);
    }

    pub(super) fn wimple(&self, model: &mut Model, wimple: &Wimple) {
        let around = self.around();
        let (chin, high, base) = (around.chin(), around.head_height(), self.neck_base());
        let neck = around.neck();
        // Round the throat, from the shoulders up over the chin.
        let surface = |u: f32, v: f32| {
            let angle = u * TAU;
            let y = base - 15.0 + (chin + 10.0 - (base - 15.0)) * v;
            let reach = neck + 14.0 + 20.0 * (1.0 - v);
            let point = Vec3::new(reach * angle.sin(), y, reach * angle.cos() + 10.0 * v);
            (point, Vec3::new(angle.sin(), 0.0, angle.cos()))
        };
        let grid = crate::item_art::armour::surface::ring_grid(&surface, 6, 32);
        crate::item_art::armour::plate(model.of(Stuff::Main), &grid, super::shape::CLOTH, true);
        // Up the sides of the face to be pinned at the temples.
        self.head(model, Stuff::Main, (0.75, 0.0), OFF_HAIR + 2.0, FACE * 0.8);
        if wimple.form == WimpleForm::Barbe {
            for pleat in 0..7 {
                let angle = -0.6 + pleat as f32 * 0.2;
                let top = Vec3::new(
                    30.0 * angle.sin(),
                    chin + high * 0.15,
                    (neck + 30.0) * angle.cos(),
                );
                // Lying on the chest, which stands out in front of the neck.
                let (x, y) = (55.0 * angle.sin(), base - 60.0);
                let chest = trunk_depth(self.frame, x, y, true) + 6.0;
                let bottom = Vec3::new(x, y, ((neck + 45.0) * angle.cos()).max(chest));
                limb(model.of(Stuff::Shade), &[(top, 1.5), (bottom, 1.5)]);
            }
        }
    }
}
