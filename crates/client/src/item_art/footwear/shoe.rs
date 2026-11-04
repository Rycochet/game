//! One shoe being built: the form, the foot it goes on, and the measurements every
//! part works from, in millimetres. The shape over the foot is foot armour's `Last`,
//! so sabatons built over a shoe fit it.

use shared::armour::feet::Toe as LastToe;
use shared::footwear::{Build, FootwearForm, Toe};

use crate::item_art::armour::feet::{Foot, Last};
use crate::item_art::armour::legs::Leg;

/// How far the upper stands off the bare foot.
const LIFT: f32 = 3.0;
/// Soft and felt boots sit looser.
const LOOSE_LIFT: f32 = 7.0;

pub(super) struct Shoe<'a> {
    pub form: &'a FootwearForm,
    pub foot: &'a Foot<'a>,
    pub leg: Leg<'a>,
}

impl<'a> Shoe<'a> {
    pub fn new(form: &'a FootwearForm, foot: &'a Foot<'a>) -> Self {
        let leg = Leg {
            frame: foot.frame,
            side: foot.side,
        };
        Self { form, foot, leg }
    }

    /// The foot's length on this frame.
    pub fn length(&self) -> f32 {
        self.foot.frame.foot_length as f32
    }

    /// Hundredths of the foot's length, in millimetres.
    pub fn of_length(&self, hundredths: f32) -> f32 {
        self.length() * hundredths / 100.0
    }

    /// How high the top is above the ground.
    pub fn top(&self) -> f32 {
        self.of_length(self.form.shaft as f32)
    }

    /// How far the upper stands off the skin.
    pub fn lift(&self) -> f32 {
        match self.form.build {
            Build::Felted => LOOSE_LIFT,
            Build::Soft if self.form.boot() => LOOSE_LIFT,
            _ => LIFT,
        }
    }

    /// How thick the upper is.
    pub fn thick(&self) -> f32 {
        match self.form.build {
            Build::Wooden if self.form.upper == shared::material::Material::Wood => 6.0,
            Build::Felted => 5.0,
            Build::Soft | Build::Rawhide => 1.6,
            _ => 2.0,
        }
    }

    /// How thick the sole is, under the foot.
    pub fn sole(&self) -> f32 {
        match self.form.build {
            Build::Sewn => 8.0,
            Build::Soft | Build::Rawhide => 2.5,
            Build::Woven => 12.0,
            Build::Felted => 6.0,
            Build::Strapped => 8.0,
            Build::Wooden if self.form.open() => 18.0,
            Build::Wooden => 30.0,
        }
    }

    /// The shape over the foot at this lift: foot armour's toes, the nearest to each of
    /// footwear's.
    pub fn last(&self, lift: f32) -> Last<'_> {
        let form = self.form;
        let (toe, point, width, up) = match form.toe {
            Toe::Round | Toe::Open => (LastToe::Round, 0.0, 100.0, 0.0),
            Toe::Almond => (LastToe::Pointed, 10.0, 100.0, 0.0),
            Toe::Pointed => (LastToe::Pointed, 28.0, 100.0, 0.0),
            Toe::LongPoint => (
                LastToe::Pointed,
                self.of_length(form.point as f32),
                100.0,
                8.0,
            ),
            Toe::Square => (LastToe::CowMouth, 0.0, 112.0, 0.0),
            Toe::Broad => (LastToe::DuckBill, 0.0, form.width as f32, 0.0),
            Toe::Upturned => (LastToe::Pointed, 24.0, 100.0, 30.0),
        };
        Last {
            foot: self.foot,
            lift,
            toe,
            point,
            width,
            up,
        }
    }

    /// Where the opening of a low shoe starts, along the foot: further forward on
    /// slip-ons, as their vamps are low.
    pub fn throat(&self) -> f32 {
        let (heel, toes) = (self.foot.heel(), self.foot.toes());
        let share = match self.form.fastening {
            shared::footwear::Fastening::PullOn => 0.62,
            _ => 0.55,
        };
        heel + (toes - heel) * share
    }
}
