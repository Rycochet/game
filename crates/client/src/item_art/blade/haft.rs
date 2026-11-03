//! Weapons on a haft: the top of the shaft, heads and parts at the side, backs, tines, barbs and tassels.

use bevy::prelude::*;
use shared::blade::{Back, BladeForm, Decoration, SidePart, Tips, Top};

use super::{HANDLE_RADIUS, Model, Stuff, half_width};

/// The socket a head on a shaft sits on, and any strips holding it on.
pub(super) fn add_shaft_top(model: &mut Model, form: &BladeForm) {
    let handle = form.handle as f32;
    let socket = (handle * 0.06).clamp(35.0, 120.0);
    let path = [
        (Vec2::new(0.0, 6.0), HANDLE_RADIUS + 1.0),
        (Vec2::new(0.0, -socket), HANDLE_RADIUS + 3.0),
    ];
    model.tube(Stuff::Blade, &path, 0.0);
    if form.langets {
        let reach = (handle * 0.14).clamp(80.0, 350.0);
        for side in [-1.0, 1.0] {
            let x = side * (HANDLE_RADIUS + 1.0);
            let path = [
                (Vec2::new(x, -socket), 4.0),
                (Vec2::new(x, -socket - reach), 3.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}

/// How far a part on the side of a shaft reaches.
pub(super) fn side_reach(form: &BladeForm) -> f32 {
    (form.blade as f32 / 2.0).clamp(40.0, 150.0)
}

/// How long a side head's edge is, drawn on the side `side` (1 for right, -1 for left).
pub(super) fn add_side_head(model: &mut Model, form: &BladeForm, side: f32, at: f32) {
    let edge = form.blade as f32;
    let deep = (edge / 2.0).min(150.0);
    let rings: Vec<Vec<Vec3>> = (0..=16)
        .map(|step| {
            let v = step as f32 / 16.0 * 2.0 - 1.0;
            let bare = v.abs();
            let y = at + v * edge / 2.0;
            let back = HANDLE_RADIUS * 0.6 + deep * 0.72 * bare * bare;
            let front = HANDLE_RADIUS + deep * (0.8 + 0.2 * (1.0 - bare * bare));
            let thick = 8.0 * (1.0 - 0.6 * bare);
            let middle = back + (front - back) * 0.6;
            [
                Vec3::new(back * side, y, thick),
                Vec3::new(middle * side, y, thick * 0.7),
                Vec3::new(front * side, y, 0.2),
                Vec3::new(middle * side, y, -thick * 0.7),
                Vec3::new(back * side, y, -thick),
            ]
            .to_vec()
        })
        .collect();
    model.loft(Stuff::Blade, &rings);
    match form.decoration {
        Decoration::Engraved | Decoration::Inlaid => {
            let stuff = if form.decoration == Decoration::Engraved {
                Stuff::Dark
            } else {
                Stuff::Inlay
            };
            let centre = Vec2::new(side * (HANDLE_RADIUS + deep * 0.4), at);
            let from = centre - Vec2::Y * edge * 0.18;
            let to = centre + Vec2::Y * edge * 0.18;
            model.ribbon(stuff, from, to, 2.5, 8.5);
        }
        Decoration::Plain | Decoration::Tasselled => {}
    }
}

/// Something other than an axe's head sticking out of a shaft.
pub(super) fn add_side_part(
    model: &mut Model,
    form: &BladeForm,
    part: SidePart,
    side: f32,
    at: f32,
) {
    let reach = side_reach(form);
    let root = Vec2::new(side * HANDLE_RADIUS * 0.6, at);
    let out = |x: f32, y: f32| root + Vec2::new(side * x, y);
    match part {
        SidePart::None => {}
        SidePart::AxeBlade => add_side_head(model, form, side, at),
        SidePart::Spike => model.spike(Stuff::Blade, root, out(reach, 0.0), 9.0),
        SidePart::Hook => {
            let path = [
                (root, 9.0),
                (out(reach * 0.55, -reach * 0.1), 7.0),
                (out(reach, reach * 0.2), 4.0),
                (out(reach * 0.9, reach * 0.65), 0.5),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
        SidePart::Beak => {
            let path = [
                (root, 12.0),
                (out(reach * 0.5, reach * 0.05), 8.0),
                (out(reach * 0.9, -reach * 0.25), 0.5),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
        SidePart::Lug => {
            let path = [(root, 7.0), (out(reach * 0.5, reach * 0.25), 0.8)];
            model.tube(Stuff::Blade, &path, 0.0);
        }
    }
}

pub(super) fn add_back(model: &mut Model, form: &BladeForm, at: f32) {
    let root = Vec2::new(-HANDLE_RADIUS * 0.6, at);
    match form.back {
        Back::None => {}
        Back::Spike => {
            let to = root - Vec2::X * side_reach(form).max(60.0);
            model.spike(Stuff::Blade, root, to, 10.0);
        }
        Back::Hammer => {
            let path = [
                (root, 11.0),
                (root - Vec2::X * 28.0, 12.0),
                (root - Vec2::X * 34.0, 17.0),
                (root - Vec2::X * 46.0, 17.0),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
        Back::Pipe => {
            let path = [
                (root, 9.0),
                (root - Vec2::X * 34.0, 7.0),
                (root + Vec2::new(-38.0, 26.0), 9.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}

/// A head on the side of a shaft, whose top is at 0.
pub(super) fn add_side_build(model: &mut Model, form: &BladeForm) {
    let edge = form.blade as f32;
    let at = -edge * 0.45;
    add_side_part(model, form, form.sides[0], 1.0, at);
    add_side_part(model, form, form.sides[1], -1.0, at);
    if form.sides[1] == SidePart::None {
        add_back(model, form, at);
    }
    let top_length = (edge * 0.45).max(80.0);
    match form.top {
        Top::None => {
            let path = [
                (Vec2::Y * 8.0, HANDLE_RADIUS * 0.8),
                (Vec2::ZERO, HANDLE_RADIUS + 1.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
        Top::Spike => model.spike(Stuff::Blade, Vec2::ZERO, Vec2::Y * top_length, 11.0),
        Top::Blade => {
            let path = [
                (Vec2::ZERO, 9.0),
                (Vec2::Y * top_length * 0.3, 16.0),
                (Vec2::Y * top_length, 0.5),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
    }
}

/// A spear's divided point: a stem, a bar, and a tine for each.
pub(super) fn add_tines(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    let places: &[f32] = if form.tips == Tips::TwoTines {
        &[-1.0, 1.0]
    } else {
        &[-1.0, 0.0, 1.0]
    };
    let fork = length * 0.3;
    let reach = half * 1.5;
    let stem = [(Vec2::ZERO, 11.0), (Vec2::Y * fork, 8.0)];
    model.tube(Stuff::Blade, &stem, 0.0);
    let bar = [
        (Vec2::new(-reach, fork), 6.0),
        (Vec2::new(reach, fork), 6.0),
    ];
    model.tube(Stuff::Blade, &bar, 0.0);
    for &place in places {
        let from = Vec2::new(place * reach, fork);
        let to = Vec2::new(place * reach, length);
        model.spike(Stuff::Blade, from, to, 6.0);
    }
}

pub(super) fn add_barbs(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    for side in [-1.0, 1.0] {
        let from = Vec2::new(side * half * 0.6, length * 0.2);
        let to = Vec2::new(side * half * 1.6, -length * 0.02);
        model.spike(Stuff::Blade, from, to, 6.0);
    }
}

/// Cords hanging from where a grip ends, or below a head on a shaft.
pub(super) fn add_tassel(model: &mut Model, from: Vec2) {
    for strand in [-1.0, 0.0, 1.0] {
        let path = [
            (from, 3.0),
            (from + Vec2::new(strand * 10.0, -30.0), 3.5),
            (from + Vec2::new(strand * 22.0, -75.0), 1.5),
        ];
        model.tube(Stuff::Cloth, &path, HANDLE_RADIUS * 0.3);
    }
}
