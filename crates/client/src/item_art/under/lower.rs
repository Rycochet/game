//! What hangs from the waist: braies down the legs, loincloths between them, and
//! petticoats round them.

use shared::body::Side;
use shared::underclothes::{
    Braies, BraiesLength, Ends, Fit, LoinForm, Loincloth, Petticoat, SkirtFullness, SkirtLength,
    SkirtMake, SkirtWaist,
};

use super::shape::{BACK, Cloth, FITTED, FRONT, FULL, LOOSE};
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::limb;

impl Cloth<'_> {
    pub(super) fn braies(&self, model: &mut Model, braies: &Braies) {
        let (waist, crotch) = (self.waist(), self.crotch());
        let end = match braies.length {
            BraiesLength::Thigh => Self::between(crotch, self.knee(), 0.5),
            BraiesLength::Knee => self.knee() - 20.0,
            BraiesLength::Calf => Self::between(self.knee(), self.ankle(), 0.5),
            BraiesLength::Ankle => self.ankle() + 30.0,
        };
        let ease = match braies.fit {
            Fit::Fitted => FITTED,
            Fit::Loose => LOOSE,
            Fit::Full => FULL,
        };
        // The seat to just below the crotch, then each leg, gathered in where tucked.
        self.trunk(model, Stuff::Main, waist + 20.0, crotch - 10.0, &|_| ease);
        let tucked = braies.ends == Ends::Tucked;
        let leg_ease = move |height: f32| match tucked {
            true if height < end + 60.0 => FITTED,
            _ => ease,
        };
        for side in Side::BOTH {
            self.leg(model, Stuff::Main, side, (crotch + 30.0, end), &leg_ease);
            if braies.ends == Ends::TiedAtCalf {
                self.leg(model, Stuff::Shade, side, (end + 25.0, end + 15.0), &|_| {
                    ease + 1.0
                });
            }
        }
        // The belt through the turned-down top.
        self.trunk_band(model, Stuff::Shade, waist + 10.0, 14.0, ease);
    }

    pub(super) fn loincloth(&self, model: &mut Model, loin: &Loincloth) {
        let (waist, crotch) = (self.waist(), self.crotch());
        let ease = FITTED;
        if loin.form == LoinForm::Briefs {
            self.trunk(model, Stuff::Main, waist - 20.0, crotch - 10.0, &|_| ease);
            for side in Side::BOTH {
                self.leg(
                    model,
                    Stuff::Main,
                    side,
                    (crotch + 20.0, crotch - 60.0),
                    &|_| ease,
                );
            }
            return;
        }
        // A cord or band round the waist, then the cloth down the front and back.
        let band = if loin.form == LoinForm::Strip {
            30.0
        } else {
            8.0
        };
        self.trunk_band(model, Stuff::Main, waist - 30.0, band, ease);
        let (wide, narrowing) = match loin.form {
            LoinForm::Narrow => (0.35, 0.0),
            LoinForm::Triangle => (1.1, 0.85),
            _ => (0.75, 0.3),
        };
        let width = move |v: f32| 1.0 - v * narrowing;
        let front_drop = match loin.form {
            LoinForm::Apron | LoinForm::Tied => Self::between(crotch, self.knee(), 0.45),
            _ => crotch - 15.0,
        };
        self.trunk_panel(
            model,
            Stuff::Main,
            (FRONT - wide, FRONT + wide),
            (waist - 30.0, front_drop),
            ease,
            &width,
        );
        let back = if loin.twisted { 0.12 } else { wide };
        self.trunk_panel(
            model,
            Stuff::Main,
            (BACK - back, BACK + back),
            (waist - 30.0, crotch - 15.0),
            ease,
            &width,
        );
    }

    pub(super) fn petticoat(&self, model: &mut Model, skirt: &Petticoat) {
        let waist = self.waist();
        let hem = match skirt.length {
            SkirtLength::Knee => self.knee() - 30.0,
            SkirtLength::Calf => Self::between(self.knee(), self.ankle(), 0.5),
            SkirtLength::Ankle => self.ankle() + 50.0,
        };
        let flare = match skirt.fullness {
            SkirtFullness::Fitted => 0.07,
            SkirtFullness::Full => 0.16,
            SkirtFullness::VeryFull => 0.26,
        };
        let ease = move |height: f32| FITTED + (waist - height).max(0.0) * flare;
        self.trunk(model, Stuff::Main, waist + 15.0, hem, &ease);
        if skirt.make == SkirtMake::Quilted {
            let rows = 5;
            for row in 1..rows {
                let height = Self::between(waist, hem, row as f32 / rows as f32);
                self.trunk_band(model, Stuff::Shade, height, 3.0, ease(height));
            }
        }
        self.trunk_band(model, Stuff::Shade, waist + 8.0, 12.0, FITTED);
        if skirt.waist == SkirtWaist::Wrap {
            // The overlap down the front.
            let line = |height: f32| {
                (
                    self.hips()
                        .at(
                            height,
                            FRONT + 0.3,
                            self.ease(ease(height), height < self.crotch()) + 2.0,
                        )
                        .0,
                    2.0,
                )
            };
            limb(model.of(Stuff::Shade), &[line(waist), line(hem + 5.0)]);
        }
    }
}
