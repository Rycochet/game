//! Laying out the middle of a town: its main roads and square.

use super::super::grid::roll;
use super::{BORDER, Block, ROADS, Road, SQUARE_CERTAIN, SQUARE_POSSIBLE, Town};

impl Town {
    /// The roads and plots. Returns which arms the middle has, its corner and
    /// the main road's width.
    pub(super) fn lay_out(&mut self) -> ([bool; 4], (i64, i64), i64) {
        let (_, _, span, _) = self.level.bounds();
        let inside = Block {
            x0: BORDER,
            y0: BORDER,
            x1: span - BORDER,
            y1: span - BORDER,
        };
        let main = if self.target <= 25 {
            2
        } else if self.target <= 60 {
            roll(&mut self.rng, 2, 3)
        } else {
            roll(&mut self.rng, 3, 4)
        };
        let main_kind = self.road_kind(ROADS.len() - 1);
        let wander = inside.width() / 16;
        let middle = span / 2 - main / 2;
        let mid_x = middle + roll(&mut self.rng, -wander, wander);
        let mid_y = middle + roll(&mut self.rng, -wander, wander);

        // The roads through the middle: which arms it has (north, east,
        // south, west). They run to the edge, where they lead out.
        let arms = self.arms;
        self.middle = (mid_x + main / 2, mid_y + main / 2);
        // A square over the middle, in the main road's surface: buildings
        // stand along its edges as along roads, and start there.
        if let Some(square) = self.choose_square(self.middle.0, self.middle.1, inside) {
            self.paint(square, ROADS[main_kind].0);
            let road = |east_west, across, along| Road {
                east_west,
                across,
                along,
                kind: main_kind,
                fixed: true,
                run: 0,
            };
            self.roads
                .push(road(true, (square.y0, square.y1), (square.x0, square.x1)));
            self.roads
                .push(road(false, (square.x0, square.x1), (square.y0, square.y1)));
        }
        if arms[0] || arms[2] {
            self.roads.push(Road {
                east_west: false,
                across: (mid_x, mid_x + main),
                along: (
                    if arms[0] { 0 } else { mid_y },
                    if arms[2] { span } else { mid_y + main },
                ),
                kind: main_kind,
                fixed: true,
                run: 0,
            });
        }
        if arms[1] || arms[3] {
            self.roads.push(Road {
                east_west: true,
                across: (mid_y, mid_y + main),
                along: (
                    if arms[3] { 0 } else { mid_x },
                    if arms[1] { span } else { mid_x + main },
                ),
                kind: main_kind,
                fixed: true,
                run: 0,
            });
        }
        for road in self.roads.clone() {
            let cells = road.area(road.along.0, road.along.1);
            for y in cells.y0..cells.y1 {
                for x in cells.x0..cells.x1 {
                    // Over the square, which is the same surface anyway.
                    self.level.set(x, y, ROADS[road.kind].0);
                }
            }
        }
        self.grow();
        (arms, (mid_x, mid_y), main)
    }

    /// A square centred near `(x, y)`, if this town has one: the chance grows
    /// with its size.
    fn choose_square(&mut self, x: i64, y: i64, inside: Block) -> Option<Block> {
        let span = SQUARE_CERTAIN - SQUARE_POSSIBLE;
        let percent = ((self.target - SQUARE_POSSIBLE) * 100 / span).clamp(0, 100);
        let rolled = roll(&mut self.rng, 0, 99);
        if self.target < SQUARE_POSSIBLE || rolled >= percent {
            return None;
        }
        let half = (3 + self.target / 24).min(inside.width() / 6);
        Some(Block {
            x0: (x - half).max(inside.x0),
            y0: (y - half).max(inside.y0),
            x1: (x + half).min(inside.x1),
            y1: (y + half).min(inside.y1),
        })
    }
}
