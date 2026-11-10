//! The shapes every piece of underclothing is made of, in millimetres on a body frame:
//! cloth round the trunk, down a leg or an arm, over a foot or a head, each an `ease`
//! off the skin. Worn under something (`Fit::Tucked`) cloth keeps close, so nothing
//! pokes through the clothes and armour over it.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::feet::Toe;
use shared::body::{Frame, Side};

use crate::item_art::armour::arms::Arm;
use crate::item_art::armour::feet::{Foot, Last};
use crate::item_art::armour::legs::Leg;
use crate::item_art::armour::neck::Around;
use crate::item_art::armour::surface::ring_grid;
use crate::item_art::armour::waist::Hips;
use crate::item_art::armour::{Model, Stuff, plate};

/// How thick cloth is drawn.
pub(super) const CLOTH: f32 = 1.2;
/// Points round a tube, and rows down a short one.
const ROUND: usize = 32;
/// Ease off the skin, in millimetres: close, fitted, loose and full.
pub(super) const CLOSE: f32 = 1.5;
pub(super) const FITTED: f32 = 3.0;
pub(super) const LOOSE: f32 = 8.0;
pub(super) const FULL: f32 = 18.0;
/// The most a tucked piece stands off the skin: on the body, and where it hangs free
/// below the hips (still inside armour's 15 mm).
const TUCKED: f32 = 2.5;
const TUCKED_HANGING: f32 = 12.0;

/// Whether a piece is drawn alone, as loose as it is cut, or worn under others.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::item_art) enum Fit {
    Alone,
    Tucked,
}

/// One piece being built on a frame.
pub(super) struct Cloth<'a> {
    pub frame: &'a Frame,
    pub fit: Fit,
}

impl Cloth<'_> {
    fn f(value: i32) -> f32 {
        value as f32
    }

    /// `ease`, or less when tucked: `hanging` is cloth below the hips, which falls free.
    pub fn ease(&self, ease: f32, hanging: bool) -> f32 {
        match self.fit {
            Fit::Alone => ease,
            Fit::Tucked if hanging => ease.min(TUCKED_HANGING),
            Fit::Tucked => ease.min(TUCKED),
        }
    }

    pub fn hips(&self) -> Hips<'_> {
        Hips { frame: self.frame }
    }

    pub fn crotch(&self) -> f32 {
        self.hips().crotch()
    }

    pub fn waist(&self) -> f32 {
        Self::f(self.frame.waist)
    }

    pub fn shoulder(&self) -> f32 {
        Self::f(self.frame.shoulder)
    }

    pub fn neck_base(&self) -> f32 {
        Self::f(self.frame.neck_base)
    }

    pub fn knee(&self) -> f32 {
        Self::f(self.frame.knee)
    }

    pub fn ankle(&self) -> f32 {
        Self::f(self.frame.ankle)
    }

    /// Halfway from `a` to `b`.
    pub fn between(a: f32, b: f32, share: f32) -> f32 {
        a + (b - a) * share
    }

    /// Cloth round the trunk from `top` down to `bottom`, `ease(height)` off the skin;
    /// below the crotch the trunk's lowest ring carries straight down, as a skirt.
    pub fn trunk(
        &self,
        model: &mut Model,
        stuff: Stuff,
        top: f32,
        bottom: f32,
        ease: &dyn Fn(f32) -> f32,
    ) {
        let hips = self.hips();
        let crotch = self.crotch();
        let surface = |u: f32, v: f32| {
            let height = Self::between(top, bottom, v);
            hips.at(height, u * TAU, self.ease(ease(height), height < crotch))
        };
        let rows = (((top - bottom) / 40.0).ceil() as usize).max(2);
        plate(
            model.of(stuff),
            &ring_grid(&surface, rows, ROUND),
            CLOTH,
            true,
        );
    }

    /// A narrow band round the trunk at `height`, standing `lift` over cloth `ease` off.
    pub fn trunk_band(&self, model: &mut Model, stuff: Stuff, height: f32, high: f32, ease: f32) {
        let ease = self.ease(ease, false) + 1.0;
        self.trunk(
            model,
            stuff,
            height + high / 2.0,
            height - high / 2.0,
            &|_| ease,
        );
    }

    /// Part of the way round the trunk, from angle `from` to `to` (0 the body's left, a
    /// quarter turn its front), from `top` down to `bottom`, `width(v)` of that at each
    /// share of the way down: panels, flaps and fronts.
    #[allow(clippy::too_many_arguments)]
    pub fn trunk_panel(
        &self,
        model: &mut Model,
        stuff: Stuff,
        (from, to): (f32, f32),
        (top, bottom): (f32, f32),
        ease: f32,
        width: &dyn Fn(f32) -> f32,
    ) {
        let hips = self.hips();
        let crotch = self.crotch();
        let middle = (from + to) / 2.0;
        let surface = |u: f32, v: f32| {
            let height = Self::between(top, bottom, v);
            let angle = middle + (u - 0.5) * (to - from) * width(v);
            hips.at(height, angle, self.ease(ease, height < crotch))
        };
        let grid = crate::item_art::armour::surface::grid(&surface, 6, 12);
        plate(model.of(stuff), &grid, CLOTH, false);
    }

    /// Cloth round one leg from `top` down to `bottom`.
    pub fn leg(
        &self,
        model: &mut Model,
        stuff: Stuff,
        side: Side,
        (top, bottom): (f32, f32),
        ease: &dyn Fn(f32) -> f32,
    ) {
        let leg = Leg {
            frame: self.frame,
            side,
        };
        let surface = |u: f32, v: f32| {
            let height = Self::between(top, bottom, v);
            leg.at(height, u * TAU, self.ease(ease(height), false))
        };
        let rows = (((top - bottom) / 40.0).ceil() as usize).max(2);
        plate(
            model.of(stuff),
            &ring_grid(&surface, rows, ROUND),
            CLOTH,
            true,
        );
    }

    /// Cloth round one arm from `from` to `to` millimetres down it from the shoulder.
    pub fn arm(
        &self,
        model: &mut Model,
        stuff: Stuff,
        side: Side,
        (from, to): (f32, f32),
        ease: &dyn Fn(f32) -> f32,
    ) {
        let arm = Arm {
            frame: self.frame,
            side,
            bend: 0.0,
        };
        let surface = |u: f32, v: f32| {
            let down = Self::between(from, to, v);
            arm.at(down, u * TAU, self.ease(ease(down), false))
        };
        let rows = (((to - from) / 40.0).ceil() as usize).max(2);
        plate(
            model.of(stuff),
            &ring_grid(&surface, rows, ROUND),
            CLOTH,
            true,
        );
    }

    /// How far down the arm the wrist is.
    pub fn wrist(&self) -> f32 {
        Self::f(self.frame.elbow_length + self.frame.wrist_length)
    }

    pub fn elbow(&self) -> f32 {
        Self::f(self.frame.elbow_length)
    }

    /// Cloth over one foot, `lift` off it, from the heel to the toes and up to the ankle.
    pub fn foot(&self, model: &mut Model, stuff: Stuff, side: Side, lift: f32) {
        let foot = Foot {
            frame: self.frame,
            side,
        };
        let last = Last {
            foot: &foot,
            lift,
            toe: Toe::Round,
            point: 0.0,
            width: 100.0,
            up: 0.0,
        };
        let (back, tip) = (last.back(), last.tip());
        let x = foot.x();
        let surface = |u: f32, v: f32| {
            // Rows from the heel to the toe, round the foot's section.
            let z = Self::between(back + 1.0, tip - 1.0, v);
            let (wide, high, middle) = last.section(z);
            let angle = u * TAU;
            let point = Vec3::new(x + wide * angle.cos(), middle + high * angle.sin(), z);
            let out = Vec3::new(
                angle.cos() / wide.max(1.0),
                angle.sin() / high.max(1.0),
                0.0,
            );
            (point, out.normalize_or(Vec3::Y))
        };
        plate(
            model.of(stuff),
            &ring_grid(&surface, 10, ROUND),
            CLOTH,
            true,
        );
    }

    /// The body round the neck and head.
    pub fn around(&self) -> Around<'_> {
        Around { frame: self.frame }
    }

    /// Cloth over the head from `top` (a share of the way from the chin to the crown,
    /// 1 at the crown) down to `bottom`, `off` the head: right round it when `face` is
    /// 0, or leaving the face open that far either side of the front.
    pub fn head(
        &self,
        model: &mut Model,
        stuff: Stuff,
        (top, bottom): (f32, f32),
        off: f32,
        face: f32,
    ) {
        let around = self.around();
        let (chin, high) = (around.chin(), around.head_height());
        let surface = |u: f32, v: f32| {
            let share = Self::between(top, bottom, v);
            let y = chin + high * share.min(0.995);
            let angle = face + u * (TAU - 2.0 * face);
            let point = around.head(angle, y, off);
            let middle = Vec3::new(0.0, chin + high / 2.0, 0.0);
            (point, (point - middle).normalize_or(Vec3::Y))
        };
        let grid = if face > 0.0 {
            crate::item_art::armour::surface::grid(&surface, 8, ROUND)
        } else {
            ring_grid(&surface, 8, ROUND)
        };
        plate(model.of(stuff), &grid, CLOTH, face <= 0.0);
    }
}

/// Half a turn: the back of the trunk is a quarter turn past it.
pub(super) const FRONT: f32 = PI / 2.0;
pub(super) const BACK: f32 = 3.0 * PI / 2.0;
