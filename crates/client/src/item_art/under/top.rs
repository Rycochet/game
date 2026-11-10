//! Shirts and breast bands: cloth round the trunk, over the shoulders and down the arms.

use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::body::Side;
use shared::underclothes::{Band, BandForm, Cut, Length, Neck, Shirt, Sleeves, Stays};

use super::shape::{BACK, CLOTH, Cloth, FITTED, FRONT, FULL, LOOSE};
use crate::item_art::armour::surface::ring_grid;
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::limb;

impl Cloth<'_> {
    /// Where a shirt's hem falls.
    fn hem(&self, length: Length) -> f32 {
        match length {
            Length::Hip => self.hips().hip(),
            Length::Thigh => Self::between(self.crotch(), self.knee(), 0.4),
            Length::Knee => self.knee(),
            Length::Calf => Self::between(self.knee(), self.ankle(), 0.5),
            Length::Ankle => self.ankle() + 40.0,
        }
    }

    pub(super) fn shirt(&self, model: &mut Model, shirt: &Shirt) {
        let (shoulder, waist) = (self.shoulder(), self.waist());
        let flare = match shirt.cut {
            Cut::Gussets => 0.05,
            Cut::Gored => 0.12,
            Cut::Full => 0.16,
        };
        let body = if shirt.cut == Cut::Full { FULL } else { LOOSE };
        let ease = move |height: f32| body + (waist - height).max(0.0) * flare;
        self.trunk(model, Stuff::Main, shoulder, self.hem(shirt.length), &ease);
        self.yoke(model, ease(shoulder));
        self.neckline(model, shirt.neck, ease(shoulder));
        self.sleeves(model, shirt.sleeves);
        if shirt.stays != Stays::None {
            self.stays(model, shirt.stays);
        }
    }

    /// Over the shoulders, from the body's top to round the neck.
    fn yoke(&self, model: &mut Model, ease: f32) {
        let (hips, around) = (self.hips(), self.around());
        let (shoulder, base) = (self.shoulder(), self.neck_base());
        let neck = around.neck() + self.ease(FITTED, false);
        let ease = self.ease(ease, false);
        let surface = |u: f32, v: f32| {
            let angle = u * TAU;
            let (outer, _) = hips.at(shoulder, angle, ease);
            let inner = Vec3::new(neck * angle.cos(), base, neck * angle.sin());
            (outer.lerp(inner, v), Vec3::Y)
        };
        plate(
            model.of(Stuff::Main),
            &ring_grid(&surface, 3, 32),
            CLOTH,
            true,
        );
    }

    /// The neck's opening as lines and bands in a darker shade, or a little collar.
    fn neckline(&self, model: &mut Model, neck: Neck, ease: f32) {
        let around = self.around();
        let (base, ring) = (
            self.neck_base(),
            around.neck() + self.ease(FITTED, false) + 1.0,
        );
        let at = |angle: f32, y: f32, out: f32| {
            Vec3::new((ring + out) * angle.cos(), y, (ring + out) * angle.sin())
        };
        let front = FRONT;
        match neck {
            Neck::Slit => limb(
                model.of(Stuff::Shade),
                &[
                    (at(front, base, 1.0), 2.0),
                    (self.hips().at(base - 110.0, front, ease + 1.0).0, 2.0),
                ],
            ),
            Neck::Collar => self.trunk(model, Stuff::Main, base + 30.0, base - 5.0, &|_| FITTED),
            Neck::Crossover => {
                let (high, _) = self.hips().at(base - 10.0, front + 0.5, ease + 1.0);
                let (low, _) = self.hips().at(self.waist(), front - 0.6, ease + 1.0);
                limb(model.of(Stuff::Shade), &[(high, 3.0), (low, 3.0)]);
            }
            Neck::Square => {
                let corner = |side: f32| {
                    self.hips()
                        .at(base - 80.0, front + side * 0.5, ease + 1.0)
                        .0
                };
                limb(
                    model.of(Stuff::Shade),
                    &[
                        (at(front + 0.6, base, 1.0), 2.0),
                        (corner(1.0), 2.0),
                        (corner(-1.0), 2.0),
                        (at(front - 0.6, base, 1.0), 2.0),
                    ],
                );
            }
            Neck::Round | Neck::Gathered => {
                let points: Vec<_> = (0..=24)
                    .map(|i| (at(i as f32 * TAU / 24.0, base + 2.0, 1.0), 2.5))
                    .collect();
                limb(model.of(Stuff::Shade), &points);
            }
        }
    }

    fn sleeves(&self, model: &mut Model, sleeves: Sleeves) {
        let (elbow, wrist) = (self.elbow(), self.wrist());
        let end = match sleeves {
            Sleeves::None => return,
            Sleeves::Short => elbow * 0.45,
            Sleeves::Elbow => elbow + 10.0,
            Sleeves::Wrist | Sleeves::Full => wrist - 10.0,
        };
        let full = sleeves == Sleeves::Full;
        // Full sleeves widen down the arm and gather back in at the wrist.
        let ease = move |down: f32| match full {
            true if down > end - 25.0 => FITTED,
            true => LOOSE + (FULL * 1.5 - LOOSE) * (down / end),
            false => LOOSE,
        };
        for side in Side::BOTH {
            self.arm(model, Stuff::Main, side, (-20.0, end), &ease);
        }
    }

    /// Stays laced over the shirt, from the chest to just below the waist, tabbed.
    fn stays(&self, model: &mut Model, stays: Stays) {
        let (top, bottom) = (
            Self::between(self.waist(), self.shoulder(), 0.75),
            self.waist() - 50.0,
        );
        let ease = LOOSE + 2.0;
        self.trunk(model, Stuff::Cloth, top, bottom, &|_| ease);
        // Lacing down the back (stays) or the front too (a pair of bodies).
        let mut ways = vec![BACK];
        if stays == Stays::PairOfBodies {
            ways.push(FRONT);
        }
        for way in ways {
            let line = |height: f32| {
                (
                    self.hips().at(height, way, self.ease(ease, false) + 2.0).0,
                    1.5,
                )
            };
            limb(model.of(Stuff::Shade), &[line(top), line(bottom)]);
        }
    }

    pub(super) fn band(&self, model: &mut Model, band: &Band) {
        let chest = Self::between(self.waist(), self.shoulder(), 0.62);
        let (top, bottom) = (chest + 45.0, chest - 45.0);
        let ease = FITTED;
        match band.form {
            BandForm::Wound | BandForm::Twisted => {
                let high = if band.form == BandForm::Twisted {
                    50.0
                } else {
                    80.0
                };
                self.trunk(
                    model,
                    Stuff::Main,
                    chest + high / 2.0,
                    chest - high / 2.0,
                    &|_| ease,
                );
                self.windings(model, chest, high, ease);
            }
            BandForm::WideWrap => {
                self.trunk(model, Stuff::Main, top, self.waist(), &|_| ease);
                self.windings(
                    model,
                    Self::between(self.waist(), top, 0.5),
                    top - self.waist(),
                    ease,
                );
            }
            BandForm::Tied => {
                self.trunk(model, Stuff::Main, top, bottom, &|_| ease);
                self.knot(model, chest, ease);
            }
            BandForm::Panel => {
                let width = |v: f32| 1.0 - v * 0.35;
                self.trunk_panel(
                    model,
                    Stuff::Main,
                    (FRONT - 0.9, FRONT + 0.9),
                    (self.neck_base() - 60.0, self.waist() - 20.0),
                    ease,
                    &width,
                );
                self.straps(model, self.neck_base() - 60.0, ease);
            }
            BandForm::Strapped => {
                self.trunk(model, Stuff::Main, top, bottom, &|_| ease);
                self.straps(model, top, ease);
            }
        }
    }

    /// Lines across a wound band, as its turns lie over each other.
    fn windings(&self, model: &mut Model, middle: f32, high: f32, ease: f32) {
        let turns = (high / 30.0).max(2.0) as usize;
        for turn in 1..turns {
            let height = middle - high / 2.0 + high * turn as f32 / turns as f32;
            self.trunk_band(model, Stuff::Shade, height, 3.0, ease);
        }
    }

    /// A knot of the ends at the back.
    fn knot(&self, model: &mut Model, height: f32, ease: f32) {
        let (point, out) = self.hips().at(height, BACK, self.ease(ease, false) + 4.0);
        limb(
            model.of(Stuff::Main),
            &[
                (point + Vec3::Y * 15.0, 7.0),
                (point + out * 6.0, 9.0),
                (point - Vec3::Y * 40.0 + out * 4.0, 5.0),
            ],
        );
    }

    /// Straps from the front over each shoulder to the back.
    fn straps(&self, model: &mut Model, from: f32, ease: f32) {
        let hips = self.hips();
        let ease = self.ease(ease, false) + 1.0;
        for side in [1.0, -1.0] {
            let over = side * (self.frame.neck as f32 + 30.0);
            let top = Vec3::new(over, self.neck_base() + 6.0, 0.0);
            let front = hips.at(from, FRONT - side * 0.45, ease).0;
            let back = hips.at(from, BACK + side * 0.45, ease).0;
            limb(
                model.of(Stuff::Main),
                &[(front, 6.0), (top, 6.0), (back, 6.0)],
            );
        }
    }
}
