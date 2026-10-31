//! Where things on a map's grid land in its picture.

/// A view of a grid in a picture: turned, scaled and centred on a point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    /// The grid position at the middle of the picture.
    pub centre: (f64, f64),
    /// The middle of the picture, in pixels.
    pub middle: (f32, f32),
    /// Grid units per pixel.
    pub units_per_pixel: f64,
    /// The grid direction of the picture's right and of its down.
    pub right: (f64, f64),
    pub down: (f64, f64),
}

impl Projection {
    /// The picture's top is north.
    pub const NORTH_UP: ((f64, f64), (f64, f64)) = ((1.0, 0.0), (0.0, 1.0));

    /// The grid position of a point in the picture.
    pub fn to_grid(self, x: f32, y: f32) -> (f64, f64) {
        let across = f64::from(x - self.middle.0) * self.units_per_pixel;
        let along = f64::from(y - self.middle.1) * self.units_per_pixel;
        (
            self.centre.0 + across * self.right.0 + along * self.down.0,
            self.centre.1 + across * self.right.1 + along * self.down.1,
        )
    }

    /// The point in the picture of a grid position.
    pub fn to_picture(self, x: f64, y: f64) -> (f32, f32) {
        let (dx, dy) = (x - self.centre.0, y - self.centre.1);
        (
            self.middle.0 + ((dx * self.right.0 + dy * self.right.1) / self.units_per_pixel) as f32,
            self.middle.1 + ((dx * self.down.0 + dy * self.down.1) / self.units_per_pixel) as f32,
        )
    }

    /// Whether the grid lines run along the picture's rows and columns.
    pub fn is_square_on(&self) -> bool {
        self.right.0.abs().max(self.right.1.abs()) > 0.9999
    }

    /// The smallest box of grid positions covering a square of `reach` pixels
    /// each way from the middle of the picture: `(min, max)`.
    pub fn grid_box(&self, reach: f32) -> ((f64, f64), (f64, f64)) {
        let (mut min, mut max) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
        for (x, y) in [
            (-reach, -reach),
            (reach, -reach),
            (-reach, reach),
            (reach, reach),
        ] {
            let at = self.to_grid(self.middle.0 + x, self.middle.1 + y);
            min = (min.0.min(at.0), min.1.min(at.1));
            max = (max.0.max(at.0), max.1.max(at.1));
        }
        (min, max)
    }
}

/// How far a drag can go from `from` towards `to` while `allowed` stays true,
/// along each axis in turn (so the view slides along the edge of what is
/// known). `allowed(from)` should be true.
pub fn slide(from: (f64, f64), to: (f64, f64), allowed: impl Fn((f64, f64)) -> bool) -> (f64, f64) {
    /// Halvings of the distance: plenty for a screenful.
    const STEPS: u32 = 10;
    let furthest = |from: f64, to: f64, allowed: &dyn Fn(f64) -> bool| {
        if allowed(to) {
            return to;
        }
        let (mut good, mut bad) = (from, to);
        for _ in 0..STEPS {
            let middle = (good + bad) / 2.0;
            if allowed(middle) {
                good = middle;
            } else {
                bad = middle;
            }
        }
        good
    };
    let x = furthest(from.0, to.0, &|x| allowed((x, from.1)));
    let y = furthest(from.1, to.1, &|y| allowed((x, y)));
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turned(right: (f64, f64), down: (f64, f64)) -> Projection {
        Projection {
            centre: (100.0, 200.0),
            middle: (50.0, 50.0),
            units_per_pixel: 2.0,
            right,
            down,
        }
    }

    #[test]
    fn grid_and_picture_positions_turn_into_each_other() {
        let north_up = turned((1.0, 0.0), (0.0, 1.0));
        assert_eq!(north_up.to_grid(60.0, 40.0), (120.0, 180.0));
        assert_eq!(north_up.to_picture(120.0, 180.0), (60.0, 40.0));
        // East at the top: right on the picture is south.
        let east_up = turned((0.0, 1.0), (-1.0, 0.0));
        assert_eq!(east_up.to_grid(60.0, 40.0), (120.0, 220.0));
        assert_eq!(east_up.to_picture(120.0, 220.0), (60.0, 40.0));
    }

    #[test]
    fn a_drag_stops_at_the_edge_of_what_is_allowed() {
        let inside = |(x, y): (f64, f64)| x < 10.0 && y < 10.0;
        let (x, y) = slide((0.0, 0.0), (100.0, 5.0), inside);
        // It stops short in x, and still moves in y.
        assert!((9.0..10.0).contains(&x));
        assert_eq!(y, 5.0);
        assert_eq!(slide((0.0, 0.0), (3.0, 4.0), inside), (3.0, 4.0));
    }
}
