//! Hose and socks: close round the legs and over the feet, under the footwear.

use bevy::prelude::*;
use shared::body::Side;
use shared::underclothes::{
    Colours, Held, Hose, HoseFeet, HoseHeight, SockHeight, SockMake, SockTop, Socks, Toe,
};

use super::shape::{CLOSE, Cloth, FRONT};
use crate::item_art::armour::feet::Foot;
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::limb;

/// How far over the foot socks and footed hose lie, under the footwear's 4 mm.
const OVER_FOOT: f32 = 1.0;

impl Cloth<'_> {
    pub(super) fn hose(&self, model: &mut Model, hose: &Hose) {
        let crotch = self.crotch();
        let top = match hose.height {
            HoseHeight::Knee => self.knee() + 50.0,
            HoseHeight::Thigh | HoseHeight::Waist => crotch - 10.0,
        };
        let bottom = self.ankle() - 10.0;
        for side in Side::BOTH {
            // Parti-coloured hose have each leg its own colour: the right in the second.
            let stuff = match (hose.colours, side) {
                (Colours::Parti, Side::Right) => Stuff::Cloth,
                _ => Stuff::Main,
            };
            self.leg(model, stuff, side, (top, bottom), &|_| CLOSE);
            if hose.feet == HoseFeet::Footed {
                self.foot(model, stuff, side, OVER_FOOT);
            }
            if hose.feet == HoseFeet::Stirrup {
                self.stirrup(model, side);
            }
            if hose.colours == Colours::Striped {
                let rows = ((top - bottom) / 60.0) as usize;
                for row in 0..rows {
                    let height = bottom + 30.0 + row as f32 * 60.0;
                    self.leg(
                        model,
                        Stuff::Cloth,
                        side,
                        (height + 12.0, height - 12.0),
                        &|_| CLOSE + 0.5,
                    );
                }
            }
            if hose.held == Held::Garters {
                let below = self.knee() - 40.0;
                self.leg(
                    model,
                    Stuff::Shade,
                    side,
                    (below + 8.0, below - 8.0),
                    &|_| CLOSE + 1.5,
                );
            }
            if hose.sole {
                self.foot(model, Stuff::Backing, side, OVER_FOOT + 1.5);
            }
        }
        if hose.height == HoseHeight::Waist {
            self.trunk(
                model,
                Stuff::Main,
                self.waist() - 10.0,
                crotch - 10.0,
                &|_| CLOSE,
            );
            if hose.codpiece {
                let (point, out) = self.hips().at(crotch + 25.0, FRONT, CLOSE);
                limb(
                    model.of(Stuff::Main),
                    &[
                        (point + Vec3::Y * 30.0, 14.0),
                        (point + out * 18.0, 20.0),
                        (point - Vec3::Y * 25.0, 12.0),
                    ],
                );
            }
        }
    }

    /// A strap under the arch.
    fn stirrup(&self, model: &mut Model, side: Side) {
        let foot = Foot {
            frame: self.frame,
            side,
        };
        let z = (foot.heel() + foot.toes()) / 2.0 - 20.0;
        let x = foot.x();
        let wide = self.frame.ankle_radius as f32 * 1.35;
        let points = [
            (Vec3::new(x - wide, self.ankle(), z), 5.0),
            (Vec3::new(x - wide, 2.0, z), 5.0),
            (Vec3::new(x + wide, 2.0, z), 5.0),
            (Vec3::new(x + wide, self.ankle(), z), 5.0),
        ];
        limb(model.of(Stuff::Main), &points);
    }

    pub(super) fn socks(&self, model: &mut Model, socks: &Socks) {
        let lift = match socks.make {
            SockMake::Nalbound | SockMake::Wrapped => OVER_FOOT + 1.5,
            SockMake::Felt => OVER_FOOT + 2.5,
            _ => OVER_FOOT,
        };
        let top = match socks.height {
            SockHeight::Foot => self.ankle() + 5.0,
            SockHeight::Ankle => self.ankle() + 45.0,
            SockHeight::Calf => Self::between(self.knee(), self.ankle(), 0.45),
            SockHeight::Knee => self.knee() - 30.0,
        };
        for side in Side::BOTH {
            self.foot(model, Stuff::Main, side, lift);
            self.leg(
                model,
                Stuff::Main,
                side,
                (top, self.ankle() - 15.0),
                &|_| lift,
            );
            match socks.top {
                SockTop::Banded | SockTop::Cuff => {
                    let high = if socks.top == SockTop::Cuff {
                        30.0
                    } else {
                        15.0
                    };
                    self.leg(model, Stuff::Shade, side, (top + 2.0, top - high), &|_| {
                        lift + 1.0
                    });
                }
                SockTop::Tied | SockTop::Hooks => {
                    self.leg(model, Stuff::Shade, side, (top - 5.0, top - 10.0), &|_| {
                        lift + 1.0
                    });
                }
                SockTop::Plain => {}
            }
            if socks.toe == Toe::Split {
                let foot = Foot {
                    frame: self.frame,
                    side,
                };
                // The split between the big toe and the rest, along the top of the toes.
                let x = foot.x() - side.sign() as f32 * 8.0;
                let front = foot.toes() + lift;
                let points = [
                    (Vec3::new(x, 22.0, front - 35.0), 2.0),
                    (Vec3::new(x, 12.0, front + 1.0), 2.0),
                ];
                limb(model.of(Stuff::Shade), &points);
            }
        }
    }
}
