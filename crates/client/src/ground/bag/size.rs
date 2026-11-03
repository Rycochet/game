//! How big a bag of a look is, and its outline's curve.

use shared::ground::Load;

/// The size of a bag of a look: how wide and tall, how far its top slumps over, and how
/// big it is drawn, as part of the biggest size.
#[derive(Clone, Copy)]
pub(in crate::ground) struct Size {
    pub radius: f32,
    pub height: f32,
    pub(super) lean: f32,
    pub scale: f32,
}

pub(in crate::ground) fn size_of(load: Load) -> Size {
    let (radius, height, lean, scale) = match load {
        Load::Flat => (8.5, 3.6, 4.0, 0.5),
        Load::Slack => (7.5, 6.5, 2.0, 0.625),
        Load::Half => (7.0, 10.5, 1.0, 0.75),
        Load::Full => (8.0, 13.5, 0.6, 0.875),
        Load::Bulging => (9.5, 16.0, 0.3, 1.0),
    };
    Size {
        radius: radius * scale,
        height: height * scale,
        lean: lean * scale,
        scale,
    }
}

/// A bag's outline from the bottom's middle to the middle of its opening, as part of
/// its radius and height.
pub(super) fn profile(load: Load) -> [(f32, f32); 10] {
    // Where the belly is: the less in it, the lower it all sags.
    let (belly, shoulder) = match load {
        Load::Flat | Load::Slack => (0.2, 0.42),
        Load::Half => (0.26, 0.5),
        Load::Full => (0.33, 0.56),
        Load::Bulging => (0.38, 0.6),
    };
    [
        (0.0, 0.0),
        (0.72, 0.0),
        (1.0, belly),
        (0.88, shoulder),
        (0.34, TIED_AT - 0.08),
        (0.2, TIED_AT),
        (0.36, 0.91),
        (0.5, 1.0),
        (0.28, 0.93),
        (0.0, 0.88),
    ]
}

/// How far up its height the rope is.
pub(super) const TIED_AT: f32 = 0.78;

/// Along a line of points: a smooth curve through them all, `along` from 0 to 1.
pub(super) fn on_curve(points: &[(f32, f32)], along: f32) -> (f32, f32) {
    let spans = points.len() - 1;
    let at = along.clamp(0.0, 1.0) * spans as f32;
    let span = (at as usize).min(spans - 1);
    let t = at - span as f32;
    let point = |index: isize| points[index.clamp(0, spans as isize) as usize];
    let span = span as isize;
    let (a, b, c, d) = (
        point(span - 1),
        point(span),
        point(span + 1),
        point(span + 2),
    );
    let blend = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2.0 * b
            + (c - a) * t
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t * t
            + (3.0 * b - a - 3.0 * c + d) * t * t * t)
    };
    (
        blend(a.0, b.0, c.0, d.0),
        blend(a.1, b.1, c.1, d.1).max(0.0),
    )
}
