//! Curtains that hang round the neck (aventails, standards, mantles, the bib of a padded
//! collar and a gorget's plates), and how each construction covers one: mail in rows,
//! rings on leather, scales, laced lamellae, quilting, leather.

use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::neck::Hem;

use super::around::{Around, Profile, out_at};
use crate::item_art::armour::surface::{grid, patch, scales, studs};
use crate::item_art::armour::{Model, Stuff, plate};

/// Hem points or scallops round a whole circle.
const TEETH_ROUND: f32 = 26.0;

/// A curtain between two angles, each angle's profile from top to bottom, its lower
/// edge cut as its hem says.
pub(super) struct Curtain<'a> {
    pub from: f32,
    pub to: f32,
    pub profile: &'a dyn Fn(f32) -> Profile,
    pub hem: Hem,
    /// How deep the hem's points or scallops are.
    pub depth: f32,
    /// The body it is kept this far clear of.
    pub clear: (&'a Around<'a>, f32),
}

impl Curtain<'_> {
    fn teeth(&self) -> f32 {
        ((self.to - self.from) / TAU * TEETH_ROUND).round().max(3.0)
    }

    /// Columns enough to follow the hem.
    pub fn columns(&self) -> usize {
        let least = ((self.to - self.from) / TAU * 32.0).ceil() as usize;
        match self.hem {
            Hem::Straight => least.max(6),
            _ => (self.teeth() as usize * 4).max(least),
        }
    }

    /// How far the hem carries on past the bottom here.
    fn hem_at(&self, u: f32) -> f32 {
        let frac = (u * self.teeth()).fract();
        match self.hem {
            Hem::Straight => 0.0,
            Hem::Vandyked => self.depth * (1.0 - (2.0 * frac - 1.0).abs()),
            Hem::Scalloped => self.depth * (1.0 - (2.0 * frac - 1.0).powi(2)).sqrt(),
        }
    }

    pub fn point(&self, u: f32, v: f32) -> Vec3 {
        let angle = self.from + (self.to - self.from) * u;
        let point = (self.profile)(angle).longer(self.hem_at(u)).at(v);
        let (around, off) = self.clear;
        around.clear(point, off)
    }

    /// The curtain as a surface, lifted `lift` off where it hangs.
    pub fn surface(&self, lift: f32) -> impl Fn(f32, f32) -> (Vec3, Vec3) + '_ {
        move |u, v| {
            let out = out_at(|u, v| self.point(u, v), u, v);
            (self.point(u, v) + out * lift, out)
        }
    }

    /// A sheet over all of it.
    pub fn sheet(&self, model: &mut Model, stuff: Stuff, lift: f32, thick: f32) {
        let surface = self.surface(lift);
        plate(
            model.of(stuff),
            &grid(&surface, 10, self.columns()),
            thick,
            false,
        );
    }

    /// A strip round it between two heights along it, finely enough to follow it.
    pub fn band(
        &self,
        model: &mut Model,
        stuff: Stuff,
        (v0, v1): (f32, f32),
        lift: f32,
        thick: f32,
    ) {
        let surface = self.surface(lift);
        let strip = |u: f32, v: f32| surface(u, v0 + (v1 - v0) * v);
        plate(
            model.of(stuff),
            &grid(&strip, 2, self.columns()),
            thick,
            false,
        );
    }

    /// Thin darker lines round it: rows of mail, or quilting.
    pub fn lines(&self, model: &mut Model, count: usize, lift: f32) {
        for line in 1..count {
            let v = line as f32 / count as f32;
            self.band(model, Stuff::Shade, (v - 0.012, v + 0.012), lift, 0.6);
        }
    }

    /// Rows down it, each standing out over the one below so its edge shows: lames
    /// and lamellae.
    pub fn bands(&self, model: &mut Model, stuff: Stuff, count: usize, step: f32, thick: f32) {
        for band in 0..count {
            let v0 = band as f32 / count as f32;
            let v1 = ((band + 1) as f32 / count as f32 + 0.12).min(1.0);
            let lift = (count - band) as f32 * step;
            self.band(model, stuff, (v0, v1), lift, thick);
        }
    }
}

/// Covers a curtain as its construction is made: the main part and its look.
pub(super) fn dress(
    model: &mut Model,
    curtain: &Curtain,
    construction: Construction,
    studded: bool,
) {
    let columns = curtain.columns();
    let surface = curtain.surface(0.0);
    match construction {
        Construction::Mail => {
            curtain.sheet(model, Stuff::Main, 0.0, 2.0);
            curtain.lines(model, 12, 1.2);
        }
        Construction::RingMail => {
            curtain.sheet(model, Stuff::Backing, 0.0, 2.0);
            studs(model, Stuff::Main, &surface, (7, columns / 2), 1.5, 2.4);
        }
        Construction::Scale => {
            curtain.sheet(model, Stuff::Backing, 0.0, 1.0);
            scales(model, Stuff::Main, &surface, 7, columns / 2);
        }
        Construction::Lamellar | Construction::Laminar => {
            curtain.sheet(model, Stuff::Backing, 0.0, 1.0);
            curtain.bands(model, Stuff::Main, 5, 1.0, 1.6);
            // The laces through each row.
            studs(model, Stuff::Cloth, &surface, (5, columns / 2), 3.5, 1.4);
        }
        Construction::MailAndPlate => {
            curtain.sheet(model, Stuff::Main, 0.0, 2.0);
            curtain.lines(model, 12, 1.2);
            // Small plates set in the mail, in rows.
            for row in 0..3 {
                let v = 0.2 + row as f32 * 0.25;
                for column in 0..columns / 4 {
                    let u = (column as f32 + 0.5) / (columns / 4) as f32;
                    let half = 0.35 / (columns / 4) as f32;
                    patch(
                        model,
                        Stuff::Main,
                        &surface,
                        (u - half, u + half),
                        (v, v + 0.12),
                        2.0,
                        1.5,
                    );
                }
            }
        }
        Construction::Padded => {
            curtain.sheet(model, Stuff::Main, 0.0, 4.0);
            curtain.lines(model, 8, 2.2);
            if studded {
                studs(model, Stuff::Rivet, &surface, (5, columns / 3), 2.5, 2.0);
            }
        }
        _ => {
            curtain.sheet(model, Stuff::Main, 0.0, 3.0);
            if studded {
                studs(model, Stuff::Rivet, &surface, (5, columns / 3), 2.0, 2.0);
            }
        }
    }
}
