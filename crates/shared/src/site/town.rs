//! Towns: roads with solid buildings along them, on the land outside.
//!
//! The middle of a town is a straight road, a T junction or a crossroads, with
//! a square over it in bigger towns. It grows a plot at a time: each new
//! building or garden goes beside one already there, more often an early one,
//! lined up with it, across the road from it, or round a corner on a new side
//! road, which gets likelier the longer a road has gone without one. A plot
//! with no room left beside it is taken off the list. Every building is solid,
//! with its door in a straight stretch of wall facing something walkable: a
//! road, except that houses may open onto any ground.
//!
//! What a building is, how it looks and its numbers come from hashes of the
//! seed and the building's number rather than the layout's random sequence, so
//! adding to them later doesn't move anything.

use serde::{Deserialize, Serialize};

use crate::dungeon::{Cell, Level, RoomId};
use crate::rng::{Rng, Seed};

use super::SiteLevel;
use super::grid::{Reach, STEPS, chance, roll};

/// How wide the ground round the whole town is.
const BORDER: i64 = 3;

/// A town's smallest and largest number of buildings on which the size-based
/// weights blend from their small to their large values.
const SMALLEST: u64 = 3;
const LARGEST: u64 = 150;

/// Towns with fewer buildings than this have no side roads.
const SIDE_ROADS_FROM: i64 = 8;

/// Tiles of frontage along a road before a side road gets likelier, and how
/// much likelier for each one after.
const SIDE_ROAD_AFTER: i64 = 8;
const SIDE_ROAD_WEIGHT: u64 = 6;

/// The longest line of building that can be built on: anything joining it
/// must leave a gap first, so nobody has further than this to walk round.
const LONGEST_WITHOUT_GAP: i64 = 10;

/// How far apart parallel roads must be, in cells: room for a building.
const BETWEEN_ROADS: i64 = 3;

/// How far along a road to look for a free plot, past whatever is in the way.
const LOOK_ALONG: i64 = 8;

/// Houses behind other buildings start at towns this big, and make up at most
/// this share of the town, in percent.
const BEHIND_FROM: i64 = 15;
const BEHIND_PERCENT: i64 = 20;

/// The share of buildings that are houses, in percent, at least.
const HOUSE_PERCENT: i64 = 30;

/// One garden per this many plots, on average.
const PLOTS_PER_GARDEN: u64 = 25;

/// A garden's smallest side is the larger of 3 and buildings divided by this.
const BUILDINGS_PER_GARDEN_SIDE: i64 = 20;

/// Squares are possible from this many buildings, and certain from the next.
const SQUARE_POSSIBLE: i64 = 10;
const SQUARE_CERTAIN: i64 = 50;

/// How a town is generated: its number of buildings, at most and at least.
/// The poi's kind decides these, so its size is known before any visit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TownOptions {
    pub buildings: (i64, i64),
}

/// What a building is. Decides its door's picture; later what it trades in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    Shrine,
    Inn,
    Tavern,
    Blacksmith,
    Herbalist,
    Carpenter,
    Tailor,
    Leatherworker,
    Fletcher,
    Church,
    Weaponsmith,
    Armoursmith,
    PotionShop,
    TrainingHall,
    ScrollShop,
    AdventurersGuild,
    Jeweller,
    Library,
    Bank,
    GemCutter,
    BlackMarket,
    Cathedral,
}

impl BuildingKind {
    pub const ALL: [Self; 23] = [
        Self::House,
        Self::Shrine,
        Self::Inn,
        Self::Tavern,
        Self::Blacksmith,
        Self::Herbalist,
        Self::Carpenter,
        Self::Tailor,
        Self::Leatherworker,
        Self::Fletcher,
        Self::Church,
        Self::Weaponsmith,
        Self::Armoursmith,
        Self::PotionShop,
        Self::TrainingHall,
        Self::ScrollShop,
        Self::AdventurersGuild,
        Self::Jeweller,
        Self::Library,
        Self::Bank,
        Self::GemCutter,
        Self::BlackMarket,
        Self::Cathedral,
    ];

    /// What players call it.
    pub fn name(self) -> &'static str {
        match self {
            Self::House => "House",
            Self::Shrine => "Shrine",
            Self::Inn => "Inn",
            Self::Tavern => "Tavern",
            Self::Blacksmith => "Blacksmith",
            Self::Herbalist => "Herbalist",
            Self::Carpenter => "Carpenter",
            Self::Tailor => "Tailor",
            Self::Leatherworker => "Leatherworker",
            Self::Fletcher => "Fletcher",
            Self::Church => "Church",
            Self::Weaponsmith => "Weaponsmith",
            Self::Armoursmith => "Armoursmith",
            Self::PotionShop => "Potion shop",
            Self::TrainingHall => "Training hall",
            Self::ScrollShop => "Scroll shop",
            Self::AdventurersGuild => "Adventurers' guild",
            Self::Jeweller => "Jeweller",
            Self::Library => "Library",
            Self::Bank => "Bank",
            Self::GemCutter => "Gem cutter",
            Self::BlackMarket => "Black market",
            Self::Cathedral => "Cathedral",
        }
    }

    /// The fewest buildings a town needs to have one, and its weight in the
    /// smallest and largest towns.
    fn odds(self) -> (i64, u64, u64) {
        match self {
            Self::House => (3, 0, 0), // Chosen first, not by weight.
            Self::Shrine => (3, 40, 5),
            Self::Inn => (3, 35, 8),
            Self::Tavern => (3, 25, 10),
            Self::Blacksmith => (3, 15, 10),
            Self::Herbalist => (3, 10, 8),
            Self::Carpenter => (6, 8, 8),
            Self::Tailor => (8, 6, 8),
            Self::Leatherworker => (8, 5, 8),
            Self::Fletcher => (10, 4, 7),
            Self::Church => (15, 2, 6),
            Self::Weaponsmith => (20, 1, 8),
            Self::Armoursmith => (20, 1, 8),
            Self::PotionShop => (12, 1, 8),
            Self::TrainingHall => (20, 1, 6),
            Self::ScrollShop => (25, 1, 6),
            Self::AdventurersGuild => (25, 1, 5),
            Self::Jeweller => (30, 1, 5),
            Self::Library => (30, 1, 4),
            Self::Bank => (30, 1, 4),
            Self::GemCutter => (40, 1, 4),
            Self::BlackMarket => (40, 1, 2),
            Self::Cathedral => (50, 1, 1),
        }
    }
}

/// What a building's walls are made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WallKind {
    Wattle,
    Timber,
    Plaster,
    Brick,
    Stone,
}

impl WallKind {
    pub const ALL: [Self; 5] = [
        Self::Wattle,
        Self::Timber,
        Self::Plaster,
        Self::Brick,
        Self::Stone,
    ];

    /// Its weight in the smallest and largest towns.
    fn odds(self) -> (u64, u64) {
        match self {
            Self::Wattle => (40, 2),
            Self::Timber => (35, 18),
            Self::Plaster => (15, 20),
            Self::Brick => (7, 30),
            Self::Stone => (3, 30),
        }
    }
}

/// What a building's roof is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoofKind {
    MudAndSticks,
    Thatch,
    WoodenShingles,
    ClayTiles,
    SlateTiles,
}

impl RoofKind {
    pub const ALL: [Self; 5] = [
        Self::MudAndSticks,
        Self::Thatch,
        Self::WoodenShingles,
        Self::ClayTiles,
        Self::SlateTiles,
    ];

    /// Its weight in the smallest and largest towns.
    fn odds(self) -> (u64, u64) {
        match self {
            Self::MudAndSticks => (45, 1),
            Self::Thatch => (35, 9),
            Self::WoodenShingles => (15, 25),
            Self::ClayTiles => (4, 35),
            Self::SlateTiles => (1, 30),
        }
    }
}

/// The road surfaces, poorest first, with their weights in the smallest and
/// largest towns.
const ROADS: [(Cell, u64, u64); 6] = [
    (Cell::DirtTrack, 60, 2),
    (Cell::GravelTrack, 25, 8),
    (Cell::WoodenBoards, 12, 8),
    (Cell::Cobblestone, 3, 35),
    (Cell::Flagstones, 0, 25),
    (Cell::DressedStone, 0, 22),
];

/// A rectangle of cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

/// What anyone can see of a building (players know a town's layout at once).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingLook {
    /// Its `RoomId` on the level.
    pub id: u16,
    pub kind: BuildingKind,
    pub walls: WallKind,
    pub roof: RoofKind,
    /// The cells it covers, as rectangles that don't overlap.
    pub rects: Vec<Rect>,
    /// Its door's cell, and the way the door faces, as a grid step.
    pub door: (i64, i64),
    pub faces: (i64, i64),
}

/// A building of a town. The numbers stay on the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TownBuilding {
    pub look: BuildingLook,
    /// How good it is, from 0 to 1000 (for 0 to 1).
    pub quality: u16,
    /// How many trades it has made when generated, from 10 to 1000, higher
    /// in bigger towns. Players' trades add to it (kept by the server).
    pub trades: u32,
}

/// A town's plan: how many buildings it aims for.
#[derive(Clone, Debug)]
pub struct Plan {
    seed: Seed,
    target: i64,
}

impl Plan {
    /// How many buildings it aims to have.
    pub fn target(&self) -> i64 {
        self.target
    }

    pub fn new(options: &TownOptions, seed: Seed) -> Self {
        let mut rng = Seed::new(seed.derive("town size").at(&[0])).rng();
        Self {
            seed,
            target: roll(&mut rng, options.buildings.0, options.buildings.1),
        }
    }

    /// Generates the town (its only level).
    pub fn level(&self) -> SiteLevel {
        Town::new(self.seed, self.target).generate()
    }
}

/// A weight between `small` and `large` for a town of `size` buildings.
fn blend(small: u64, large: u64, size: i64) -> u64 {
    let along = (size.max(0) as u64).clamp(SMALLEST, LARGEST) - SMALLEST;
    let span = LARGEST - SMALLEST;
    (small * (span - along) + large * along) / span
}

/// One of `choices` at random, more likely the more it weighs.
fn pick<T: Copy>(rng: &mut Rng, choices: &[(T, u64)]) -> T {
    let total: u64 = choices.iter().map(|&(_, weight)| weight).sum();
    let mut left = rng.range_u64(0, total.max(1));
    for &(choice, weight) in choices {
        if left < weight {
            return choice;
        }
        left -= weight;
    }
    choices[0].0
}

/// A rectangle of cells (`x1`, `y1` just beyond it).
#[derive(Clone, Copy, Debug)]
struct Block {
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
}

impl Block {
    fn width(self) -> i64 {
        self.x1 - self.x0
    }
}

/// A straight road buildings stand along.
#[derive(Clone, Copy, Debug)]
struct Road {
    east_west: bool,
    /// The rows (or columns) it covers.
    across: (i64, i64),
    /// How far it goes, from the first up to just before the second.
    along: (i64, i64),
    /// An index into `ROADS`.
    kind: usize,
    /// Whether it stays as it is: main roads and a square's edges.
    fixed: bool,
    /// Tiles of frontage since its last side road.
    run: i64,
}

impl Road {
    /// The cells it covers from `from` up to just before `to`.
    fn area(&self, from: i64, to: i64) -> Block {
        if self.east_west {
            Block {
                x0: from,
                x1: to,
                y0: self.across.0,
                y1: self.across.1,
            }
        } else {
            Block {
                x0: self.across.0,
                x1: self.across.1,
                y0: from,
                y1: to,
            }
        }
    }

    /// A plot starting `a` along it, on its side `side` (1 is south or east,
    /// -1 north or west).
    fn frame(&self, side: i64, a: i64) -> Frame {
        let edge = if side > 0 {
            self.across.1
        } else {
            self.across.0 - 1
        };
        if self.east_west {
            Frame {
                corner: (a, edge),
                along: (1, 0),
                inward: (0, side),
            }
        } else {
            Frame {
                corner: (edge, a),
                along: (0, 1),
                inward: (side, 0),
            }
        }
    }
}

/// A building or garden on a road, which more can be put beside.
#[derive(Clone, Copy, Debug)]
struct Plot {
    road: usize,
    side: i64,
    along: (i64, i64),
    deep: i64,
}

/// A way to put something beside a plot.
#[derive(Clone, Copy, Debug)]
enum Beside {
    /// Further along the road, lined up with it: `1` or `-1`.
    Along(i64),
    /// Across the road, lined up with one end of it.
    Across,
    /// On a new side road past its end: `1` or `-1`.
    Corner(i64),
    /// A house behind it, its door away from it.
    Behind,
}

/// A building being placed.
struct Placed {
    cells: Vec<(i64, i64)>,
    door: (i64, i64),
    faces: (i64, i64),
    /// Whether its door opens onto a road, so it can be more than a house.
    on_road: bool,
}

/// A town being generated.
struct Town {
    seed: Seed,
    target: i64,
    rng: Rng,
    level: Level,
    placed: Vec<Placed>,
    roads: Vec<Road>,
    /// What can still have something put beside it, oldest first.
    plots: Vec<Plot>,
    /// The middle of the town, where growth starts.
    middle: (i64, i64),
    /// How many houses stand behind others.
    behind: i64,
    /// The bare ground in front of their doors, kept clear.
    door_fronts: Vec<(i64, i64)>,
}

impl Town {
    fn new(seed: Seed, target: i64) -> Self {
        // A generous area, cut down to what is used at the end.
        let span = (target * 90).isqrt().max(24) + 2 * BORDER;
        let mut level = Level::new(0, 0, span, span);
        for y in 0..span {
            for x in 0..span {
                level.set(x, y, Cell::Ground);
            }
        }
        Self {
            seed,
            target,
            rng: Seed::new(seed.derive("town").at(&[0])).rng(),
            level,
            placed: Vec::new(),
            roads: Vec::new(),
            plots: Vec::new(),
            middle: (span / 2, span / 2),
            behind: 0,
            door_fronts: Vec::new(),
        }
    }

    /// A road surface, by the size of the town, no better than `best` (an
    /// index into `ROADS`).
    fn road_kind(&mut self, best: usize) -> usize {
        let choices: Vec<(usize, u64)> = ROADS[..=best]
            .iter()
            .enumerate()
            .map(|(i, &(_, small, large))| (i, blend(small, large, self.target)))
            .collect();
        pick(&mut self.rng, &choices)
    }

    fn paint(&mut self, area: Block, cell: Cell) {
        for y in area.y0..area.y1 {
            for x in area.x0..area.x1 {
                self.level.set(x, y, cell);
            }
        }
    }

    fn generate(mut self) -> SiteLevel {
        let (arms, middle, main) = self.lay_out();
        self.finish(arms, middle, main)
    }

    /// The roads and plots. Returns which arms the middle has, its corner and
    /// the main road's width.
    fn lay_out(&mut self) -> ([bool; 4], (i64, i64), i64) {
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
        let arms = self.choose_arms();
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

    /// Which arms the middle of the town has: small towns are mostly one
    /// straight road, big ones mostly crossroads.
    fn choose_arms(&mut self) -> [bool; 4] {
        let straight = (60 - self.target).max(10) as u32;
        let t_junction = 40;
        let shape = roll(&mut self.rng, 0, 99) as u32;
        let shape = if self.target < 6 { 0 } else { shape };
        if shape < straight {
            if chance(&mut self.rng, 50) {
                [true, false, true, false]
            } else {
                [false, true, false, true]
            }
        } else if shape < straight + t_junction {
            let missing = roll(&mut self.rng, 0, 3) as usize;
            std::array::from_fn(|i| i != missing)
        } else {
            [true; 4]
        }
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

    /// Adds plots until the town has all its buildings or no room: each
    /// beside a plot from the list, more often an early one. One with nothing
    /// left beside it comes off the list.
    fn grow(&mut self) {
        while (self.placed.len() as i64) < self.target {
            if self.plots.is_empty() {
                if !self.start_plot() {
                    return;
                }
                continue;
            }
            let last = self.plots.len() as i64 - 1;
            let index = roll(&mut self.rng, 0, last).min(roll(&mut self.rng, 0, last));
            let plot = self.plots[index as usize];
            if !self.add_beside(plot) {
                self.plots.remove(index as usize);
            }
        }
    }

    /// A plot on any road, as near the middle as there is room.
    fn start_plot(&mut self) -> bool {
        let (_, _, span, _) = self.level.bounds();
        for distance in 0..span {
            for road in 0..self.roads.len() {
                let middle = if self.roads[road].east_west {
                    self.middle.0
                } else {
                    self.middle.1
                };
                for side in [1, -1] {
                    for sign in [1, -1] {
                        let widest = self.widest();
                        let width = roll(&mut self.rng, 3, widest);
                        let a = middle + sign * distance;
                        if self.place_on(road, side, a, width, None) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn widest(&self) -> i64 {
        if self.target <= 25 { 6 } else { 9 }
    }

    fn deepest(&self) -> i64 {
        if self.target <= 25 { 6 } else { 8 }
    }

    /// Tries the ways of adding something beside a plot, likeliest first by
    /// chance, until one fits.
    fn add_beside(&mut self, plot: Plot) -> bool {
        let run = self.roads[plot.road].run;
        let corner = if self.target < SIDE_ROADS_FROM {
            0
        } else {
            2 + (run - SIDE_ROAD_AFTER).max(0) as u64 * SIDE_ROAD_WEIGHT
        };
        let behind_room = self.behind * 100 < self.target * BEHIND_PERCENT;
        let behind = if self.target >= BEHIND_FROM && behind_room {
            10
        } else {
            0
        };
        let mut ways = vec![
            (Beside::Along(1), 30),
            (Beside::Along(-1), 30),
            (Beside::Across, 30),
            (Beside::Corner(1), corner),
            (Beside::Corner(-1), corner),
            (Beside::Behind, behind),
        ];
        ways.retain(|&(_, weight)| weight > 0);
        while !ways.is_empty() {
            let choices: Vec<(usize, u64)> =
                ways.iter().enumerate().map(|(i, w)| (i, w.1)).collect();
            let (way, _) = ways.remove(pick(&mut self.rng, &choices));
            let done = match way {
                Beside::Along(sign) => self.add_along(plot, sign),
                Beside::Across => self.add_across(plot),
                Beside::Corner(sign) => self.add_corner(plot, sign),
                Beside::Behind => self.add_behind(plot),
            };
            if done {
                return true;
            }
        }
        false
    }

    /// Further along the road: touching, or with an alley left as bare ground
    /// (only a side road would make it more), its back lined up if it can be.
    fn add_along(&mut self, plot: Plot, sign: i64) -> bool {
        let gap = pick(&mut self.rng, &[(0, 55), (1, 30), (2, 15)]);
        let widest = self.widest();
        let width = roll(&mut self.rng, 3, widest);
        let deep = chance(&mut self.rng, 60).then_some(plot.deep);
        (0..=LOOK_ALONG).any(|extra| {
            let a = if sign > 0 {
                plot.along.1 + gap + extra
            } else {
                plot.along.0 - gap - extra - width
            };
            self.place_on(plot.road, plot.side, a, width, deep)
        })
    }

    /// Across the road, lined up with one end of the plot.
    fn add_across(&mut self, plot: Plot) -> bool {
        let widest = self.widest();
        let width = roll(&mut self.rng, 3, widest);
        let starts = [plot.along.0, plot.along.1 - width];
        let first = roll(&mut self.rng, 0, 1) as usize;
        [starts[first], starts[1 - first]]
            .into_iter()
            .any(|a| self.place_on(plot.road, -plot.side, a, width, None))
    }

    /// A new side road past the plot's end, of a surface no better, running
    /// back along its flank, with something on it: behind the plot, or on its
    /// far side level with the plot's front.
    fn add_corner(&mut self, plot: Plot, sign: i64) -> bool {
        let road = self.roads[plot.road];
        let wide = (road.across.1 - road.across.0 - 1).clamp(2, 3);
        let band = if sign > 0 {
            (plot.along.1, plot.along.1 + wide)
        } else {
            (plot.along.0 - wide, plot.along.0)
        };
        let widest = self.widest();
        let width = roll(&mut self.rng, 3, widest);
        let length = plot.deep + width;
        let along = if plot.side > 0 {
            (road.across.1, road.across.1 + length)
        } else {
            (road.across.0 - length, road.across.0)
        };
        let side_road = Road {
            east_west: !road.east_west,
            across: band,
            along,
            kind: self.road_kind(road.kind),
            fixed: false,
            run: 0,
        };
        let area = side_road.area(along.0, along.1);
        if !self.road_fits(&side_road, area, None) {
            return false;
        }
        // It has to meet this road, which may need to reach it first.
        let reaches = band.0 >= road.along.0 && band.1 <= road.along.1;
        let extended = (road.along.0.min(band.0), road.along.1.max(band.1));
        let extra = [
            road.area(extended.0, road.along.0),
            road.area(road.along.1, extended.1),
        ];
        if !reaches && (road.fixed || !self.extension_fits(plot.road, extra)) {
            return false;
        }
        // Painted first, as doors look for a road; put back if nothing fits.
        let mut painted = Vec::new();
        for y in area.y0..area.y1 {
            for x in area.x0..area.x1 {
                if self.level.cell(x, y) == Cell::Ground {
                    self.level.set(x, y, ROADS[side_road.kind].0);
                    painted.push((x, y));
                }
            }
        }
        let index = self.roads.len();
        self.roads.push(side_road);
        // Towards the plot's side of the new road, starting behind the plot.
        let (towards, away) = (-sign, sign);
        let behind = if plot.side > 0 {
            along.0 + plot.deep
        } else {
            along.1 - plot.deep - width
        };
        let level_with = if plot.side > 0 {
            along.0
        } else {
            along.1 - width
        };
        let tries = if chance(&mut self.rng, 60) {
            [(towards, behind), (away, level_with)]
        } else {
            [(away, level_with), (towards, behind)]
        };
        if tries
            .into_iter()
            .any(|(side, a)| self.place_on(index, side, a, width, None))
        {
            for part in extra {
                self.paint(part, ROADS[road.kind].0);
            }
            self.roads[plot.road].along = extended;
            self.roads[plot.road].run = 0;
            return true;
        }
        self.roads.pop();
        for (x, y) in painted {
            self.level.set(x, y, Cell::Ground);
        }
        false
    }

    /// A house behind the plot, lined up with one end of it, its door away
    /// from the plot onto bare ground.
    fn add_behind(&mut self, plot: Plot) -> bool {
        let front = self.roads[plot.road].frame(plot.side, plot.along.0);
        let room = plot.along.1 - plot.along.0;
        if room < 3 {
            return false;
        }
        let width = roll(&mut self.rng, 3, room.min(5));
        let deep = roll(&mut self.rng, 3, 4);
        let gap = roll(&mut self.rng, 1, 2);
        let offset = if chance(&mut self.rng, 50) {
            0
        } else {
            room - width
        };
        let frame = Frame {
            corner: front.at(offset, plot.deep + gap + deep - 1),
            along: front.along,
            inward: front.out(),
        };
        let mask = vec![true; (width * deep) as usize];
        if self.line_too_long(&frame, width, deep)
            || !self.place_building(&frame, width, deep, &mask, false)
        {
            return false;
        }
        self.behind += 1;
        if let Some(placed) = self.placed.last() {
            let (x, y) = placed.door;
            self.door_fronts
                .push((x + placed.faces.0, y + placed.faces.1));
        }
        true
    }

    /// Whether a road could run over these cells: only bare ground inside the
    /// town's border (it never replaces another road), and no road running the
    /// same way beside it without room for a building between. `own` is the
    /// road being extended, if any.
    fn road_fits(&self, road: &Road, area: Block, own: Option<usize>) -> bool {
        let bare = (area.y0..area.y1).all(|y| {
            (area.x0..area.x1).all(|x| self.inside(x, y) && self.level.cell(x, y) == Cell::Ground)
        });
        let along = if road.east_west {
            (area.x0, area.x1)
        } else {
            (area.y0, area.y1)
        };
        bare && self.roads.iter().enumerate().all(|(index, other)| {
            let overlap = along.0 <= other.along.1 && other.along.0 <= along.1;
            Some(index) == own
                || other.east_west != road.east_west
                || !overlap
                || other.across.0 >= road.across.1 + BETWEEN_ROADS
                || road.across.0 >= other.across.1 + BETWEEN_ROADS
        })
    }

    /// Whether a road can be extended over these areas.
    fn extension_fits(&self, index: usize, areas: [Block; 2]) -> bool {
        let road = self.roads[index];
        areas.iter().all(|&area| {
            area.x0 >= area.x1 || area.y0 >= area.y1 || self.road_fits(&road, area, Some(index))
        })
    }

    /// Whether a building here would carry on a straight line of buildings
    /// that is already too long, or join two into one: it may take a line past
    /// the limit itself, but not build on one that has reached it.
    fn line_too_long(&self, frame: &Frame, width: i64, deep: i64) -> bool {
        let run = |start: (i64, i64), step: (i64, i64)| {
            let mut count = 0;
            let mut at = start;
            while self.level.cell(at.0, at.1).is_wall() && self.inside(at.0, at.1) {
                count += 1;
                at = (at.0 + step.0, at.1 + step.1);
            }
            count
        };
        let along = frame.along;
        let inward = frame.inward;
        let back = (-along.0, -along.1);
        let out = frame.out();
        (0..deep).any(|d| {
            run(frame.at(-1, d), back) + run(frame.at(width, d), along) >= LONGEST_WITHOUT_GAP
        }) || (0..width).any(|a| {
            run(frame.at(a, -1), out) + run(frame.at(a, deep), inward) >= LONGEST_WITHOUT_GAP
        })
    }

    /// Puts a garden or building on a road, `a` along it, extending the road
    /// in front of it if it needs to and can. `deep` asks for a depth, to line
    /// up with a neighbour. The new plot joins the list.
    fn place_on(&mut self, index: usize, side: i64, a: i64, width: i64, deep: Option<i64>) -> bool {
        let smallest_garden = (self.target / BUILDINGS_PER_GARDEN_SIDE).max(3);
        let garden = self.rng.range_u64(0, PLOTS_PER_GARDEN) == 0;
        let (width, deep) = if garden {
            (
                roll(&mut self.rng, smallest_garden, smallest_garden + 2),
                roll(&mut self.rng, smallest_garden, smallest_garden + 2),
            )
        } else {
            let most = self.deepest();
            (width, deep.unwrap_or_else(|| roll(&mut self.rng, 3, most)))
        };
        let road = self.roads[index];
        let reaches = a >= road.along.0 && a + width <= road.along.1;
        let frame = road.frame(side, a);
        if !self.is_free(&frame, width, deep) {
            return false;
        }
        // The road in front, from where it ends to the far end of the plot.
        let extended = (road.along.0.min(a), road.along.1.max(a + width));
        if !reaches && road.fixed {
            return false;
        }
        let extra = [
            road.area(extended.0, road.along.0),
            road.area(road.along.1, extended.1),
        ];
        if !reaches && !self.extension_fits(index, extra) {
            return false;
        }
        if !garden && self.line_too_long(&frame, width, deep) {
            return false;
        }
        for area in extra {
            self.paint(area, ROADS[road.kind].0);
        }
        self.roads[index].along = extended;
        let placed = if garden {
            self.place_garden(&frame, width, deep)
        } else {
            let mask = self.building_shape(width, deep);
            self.place_building(&frame, width, deep, &mask, true)
        };
        if placed {
            self.roads[index].run += width;
            self.plots.push(Plot {
                road: index,
                side,
                along: (a, a + width),
                deep,
            });
        }
        placed
    }

    /// Which cells of a plot a building or garden covers: mostly all, in
    /// bigger plots sometimes an L, T or H. The front row is always kept
    /// wide enough for a door, and every arm is at least 2 wide.
    fn building_shape(&mut self, width: i64, deep: i64) -> Vec<bool> {
        let mut mask = vec![true; (width * deep) as usize];
        let percent = (15 + self.target / 3).min(50) as u32;
        if width < 5 || deep < 5 || !chance(&mut self.rng, percent) {
            return mask;
        }
        let mut cut = |x0: i64, x1: i64, y0: i64, y1: i64| {
            for y in y0..y1 {
                for x in x0..x1 {
                    mask[(y * width + x) as usize] = false;
                }
            }
        };
        let choice = roll(&mut self.rng, 0, 2);
        if choice == 2 && width >= 8 {
            // An H: a bite out of the middle of the front and back, its arms
            // at least 3 wide, the bar between them at least 2 deep.
            let bite = roll(&mut self.rng, 2, width - 6);
            let x0 = (width - bite) / 2;
            cut(x0, x0 + bite, 0, 1);
            cut(x0, x0 + bite, deep - 1, deep);
        } else if choice == 1 && width >= 7 {
            // A T: the back corners gone, the stem at least 3 wide.
            let each = roll(&mut self.rng, 2, (width - 3) / 2);
            let bar = roll(&mut self.rng, 2, deep - 2);
            cut(0, each, bar, deep);
            cut(width - each, width, bar, deep);
        } else {
            // An L: one back corner gone.
            let across = roll(&mut self.rng, 2, width - 3);
            let bar = roll(&mut self.rng, 2, deep - 2);
            if chance(&mut self.rng, 50) {
                cut(0, across, bar, deep);
            } else {
                cut(width - across, width, bar, deep);
            }
        }
        mask
    }

    /// Whether a plot is all bare ground.
    fn is_free(&self, frame: &Frame, width: i64, deep: i64) -> bool {
        (0..deep).all(|d| {
            (0..width).all(|a| {
                let (x, y) = frame.at(a, d);
                self.inside(x, y)
                    && self.level.cell(x, y) == Cell::Ground
                    && !self.door_fronts.contains(&(x, y))
            })
        })
    }

    /// Inside the town's border.
    fn inside(&self, x: i64, y: i64) -> bool {
        let (_, _, span, _) = self.level.bounds();
        (BORDER..span - BORDER).contains(&x) && (BORDER..span - BORDER).contains(&y)
    }

    fn place_garden(&mut self, frame: &Frame, width: i64, deep: i64) -> bool {
        if !self.is_free(frame, width, deep) {
            return false;
        }
        // Gardens never touch, so each keeps its own hedge.
        let touching = (-1..=deep).any(|d| {
            (-1..=width).any(|a| {
                let (x, y) = frame.at(a, d);
                self.level.cell(x, y) == Cell::Garden
            })
        });
        if touching {
            return false;
        }
        let mask = self.building_shape(width, deep);
        for d in 0..deep {
            for a in 0..width {
                if mask[(d * width + a) as usize] {
                    let (x, y) = frame.at(a, d);
                    self.level.set(x, y, Cell::Garden);
                }
            }
        }
        true
    }

    /// A building on a plot, its door in the front wall: facing a road if
    /// `on_road`, otherwise bare ground.
    fn place_building(
        &mut self,
        frame: &Frame,
        width: i64,
        deep: i64,
        mask: &[bool],
        on_road: bool,
    ) -> bool {
        if !self.is_free(frame, width, deep) {
            return false;
        }
        let covered = |a: i64, d: i64| {
            (0..width).contains(&a) && (0..deep).contains(&d) && mask[(d * width + a) as usize]
        };
        // In a straight stretch of the front wall, with building behind.
        let doors: Vec<i64> = (0..width)
            .filter(|&a| {
                covered(a, 0) && covered(a - 1, 0) && covered(a + 1, 0) && covered(a, 1) && {
                    let (x, y) = frame.at(a, -1);
                    let cell = self.level.cell(x, y);
                    if on_road {
                        cell.is_road()
                    } else {
                        cell == Cell::Ground && self.inside(x, y)
                    }
                }
            })
            .collect();
        if doors.is_empty() {
            return false;
        }
        let door = doors[roll(&mut self.rng, 0, doors.len() as i64 - 1) as usize];
        let mut cells = Vec::new();
        for d in 0..deep {
            for a in 0..width {
                if covered(a, d) {
                    cells.push(frame.at(a, d));
                }
            }
        }
        self.add_building(cells, frame.at(door, 0), frame.out(), on_road);
        true
    }

    /// Records a building and marks its cells.
    fn add_building(
        &mut self,
        cells: Vec<(i64, i64)>,
        door: (i64, i64),
        faces: (i64, i64),
        on_road: bool,
    ) {
        for &(x, y) in &cells {
            self.level.set(x, y, Cell::Wall);
        }
        self.placed.push(Placed {
            cells,
            door,
            faces,
            on_road,
        });
    }

    /// Opens up anything walled in, cuts the level down to what is used, and
    /// decides what each building is.
    fn finish(mut self, arms: [bool; 4], middle: (i64, i64), main: i64) -> SiteLevel {
        let (_, _, span, _) = self.level.bounds();
        let entry = if arms[2] {
            (middle.0 + main / 2, span - 1)
        } else if arms[1] {
            (span - 1, middle.1 + main / 2)
        } else if arms[3] {
            (0, middle.1 + main / 2)
        } else {
            (middle.0 + main / 2, 0)
        };
        self.open_pockets(entry);

        // The used area plus the border, always including the middle.
        let (mut x0, mut y0) = (middle.0, middle.1);
        let (mut x1, mut y1) = (middle.0 + main, middle.1 + main);
        for y in 0..span {
            for x in 0..span {
                let cell = self.level.cell(x, y);
                if cell.is_wall() || cell == Cell::Garden {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        let (x0, y0) = (x0 - BORDER, y0 - BORDER);
        let (x1, y1) = (x1 + BORDER, y1 + BORDER);
        let mut level = Level::new(x0, y0, x1 - x0, y1 - y0);
        for y in y0..y1 {
            for x in x0..x1 {
                level.set(x, y, self.level.cell(x, y));
            }
        }
        let entrance = (entry.0.clamp(x0, x1 - 1), entry.1.clamp(y0, y1 - 1));
        self.level = level;
        self.roads_to_the_edge();

        let kinds = self.choose_kinds();
        let mut buildings = Vec::new();
        let placed = std::mem::take(&mut self.placed);
        for (index, (building, kind)) in placed.iter().zip(kinds).enumerate() {
            buildings.push(self.make_building(index, building, kind));
        }
        self.add_windows(&buildings);
        SiteLevel {
            level: self.level,
            stairs: Vec::new(),
            entrance: Some(entrance),
            buildings,
        }
    }

    /// Any road reaching the town's border carries on through it to the edge
    /// of the level, like the main roads, unless something is in the way.
    fn roads_to_the_edge(&mut self) {
        let (x0, y0, width, height) = self.level.bounds();
        for road in self.roads.clone() {
            let (first, last) = if road.east_west {
                (x0, x0 + width)
            } else {
                (y0, y0 + height)
            };
            // Each way, out from its end, until the edge or another road.
            let outwards: [Vec<i64>; 2] = [
                if road.along.0 <= first + BORDER {
                    (first..road.along.0).rev().collect()
                } else {
                    Vec::new()
                },
                if road.along.1 >= last - BORDER {
                    (road.along.1..last).collect()
                } else {
                    Vec::new()
                },
            ];
            for way in outwards {
                for a in way {
                    let area = road.area(a, a + 1);
                    let bare = (area.y0..area.y1)
                        .all(|y| (area.x0..area.x1).all(|x| self.level.cell(x, y) == Cell::Ground));
                    if !bare {
                        break;
                    }
                    self.paint(area, ROADS[road.kind].0);
                }
            }
        }
    }

    /// While any ground can't be walked to, takes away a building next to it
    /// (the first that also touches ground that can be).
    fn open_pockets(&mut self, entry: (i64, i64)) {
        loop {
            let mut reach = Reach::new(&self.level);
            reach.spread(&self.level, &[entry]);
            let Some(pocket) = reach.first_unreached(&self.level) else {
                return;
            };
            let mut inside = Reach::new(&self.level);
            inside.spread(&self.level, &[pocket]);
            let touches = |cells: &[(i64, i64)], reached: &Reach| {
                cells
                    .iter()
                    .any(|&(x, y)| STEPS.iter().any(|(dx, dy)| reached.is(x + dx, y + dy)))
            };
            let bordering = |p: &&Placed| touches(&p.cells, &inside);
            let best = self
                .placed
                .iter()
                .filter(bordering)
                .position(|p| touches(&p.cells, &reach));
            let index = match best {
                Some(_) => self
                    .placed
                    .iter()
                    .position(|p| touches(&p.cells, &inside) && touches(&p.cells, &reach)),
                None => self.placed.iter().position(|p| touches(&p.cells, &inside)),
            };
            let Some(index) = index else {
                return;
            };
            let gone = self.placed.remove(index);
            for (x, y) in gone.cells {
                self.level.set(x, y, Cell::Ground);
            }
        }
    }

    /// What each building is. Houses first: all those not on a road, then
    /// others up to their share. The rest by weight, each copy of a kind
    /// halving its weight; the first is always a shrine or an inn.
    fn choose_kinds(&mut self) -> Vec<BuildingKind> {
        let count = self.placed.len() as i64;
        let mut rng = Seed::new(self.seed.derive("town kinds").at(&[0])).rng();
        let mut kinds = vec![None; self.placed.len()];
        let mut on_road = Vec::new();
        let mut houses = 0;
        for (index, placed) in self.placed.iter().enumerate() {
            if placed.on_road {
                on_road.push(index);
            } else {
                kinds[index] = Some(BuildingKind::House);
                houses += 1;
            }
        }
        // A shuffle, so which are houses isn't down to where they are.
        for i in (1..on_road.len()).rev() {
            on_road.swap(i, roll(&mut rng, 0, i as i64) as usize);
        }
        let share = (count * HOUSE_PERCENT + 50) / 100;
        let mut left = on_road.into_iter();
        while houses < share {
            let Some(index) = left.next() else {
                break;
            };
            kinds[index] = Some(BuildingKind::House);
            houses += 1;
        }
        let mut copies = std::collections::BTreeMap::new();
        let mut first = true;
        for index in left {
            let kind = if first {
                pick(
                    &mut rng,
                    &[(BuildingKind::Shrine, 1), (BuildingKind::Inn, 1)],
                )
            } else {
                let choices: Vec<(BuildingKind, u64)> = BuildingKind::ALL
                    .iter()
                    .filter(|kind| kind.odds().0 <= count && **kind != BuildingKind::House)
                    .map(|&kind| {
                        let (_, small, large) = kind.odds();
                        let had: u32 = copies.get(&kind).copied().unwrap_or(0);
                        // Past its fair share (a twentieth), much faster.
                        let share = (count / 20).max(2) as u32;
                        let had = had + 3 * had.saturating_sub(share);
                        // In fine steps, so halving doesn't round to nothing.
                        (
                            kind,
                            (blend(small, large, count) << 16 >> had.min(16)).max(1),
                        )
                    })
                    .collect();
                pick(&mut rng, &choices)
            };
            first = false;
            *copies.entry(kind).or_insert(0) += 1;
            kinds[index] = Some(kind);
        }
        kinds
            .into_iter()
            .map(|kind| kind.unwrap_or(BuildingKind::House))
            .collect()
    }

    /// The building's final form: its door, room and looks on the level.
    fn make_building(&mut self, index: usize, placed: &Placed, kind: BuildingKind) -> TownBuilding {
        let id = index as u16;
        for &(x, y) in &placed.cells {
            self.level.set_room(x, y, Some(RoomId(id)));
        }
        self.level
            .set(placed.door.0, placed.door.1, Cell::BuildingDoor);
        let count = self.target;
        let own = self.seed.derive("town building");
        let mut looks = Seed::new(own.at(&[index as i64, 0])).rng();
        let walls: Vec<_> = WallKind::ALL
            .iter()
            .map(|&k| (k, blend(k.odds().0, k.odds().1, count)))
            .collect();
        let roofs: Vec<_> = RoofKind::ALL
            .iter()
            .map(|&k| (k, blend(k.odds().0, k.odds().1, count)))
            .collect();
        let walls = pick(&mut looks, &walls);
        let roof = pick(&mut looks, &roofs);
        let mut numbers = Seed::new(own.at(&[index as i64, 1])).rng();
        let quality = numbers.range_u64(0, 1001) as u16;
        // Up to 1000 in the largest towns, with a floor rising with the top.
        let most = 10 + 990 * count.clamp(0, LARGEST as i64) / LARGEST as i64;
        let trades = roll(&mut numbers, (most / 5).max(10), most) as u32;
        TownBuilding {
            look: BuildingLook {
                id,
                kind,
                walls,
                roof,
                rects: rects_of(&placed.cells),
                door: placed.door,
                faces: placed.faces,
            },
            quality,
            trades,
        }
    }

    /// Some windows, for looks: in straight stretches of wall facing
    /// something walkable, never beside a door or another window. Nothing
    /// can be seen through them.
    fn add_windows(&mut self, buildings: &[TownBuilding]) {
        let windows = self.seed.derive("town windows");
        let (min_x, min_y, width, height) = self.level.bounds();
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let Some(room) = self.level.room(x, y) else {
                    continue;
                };
                if self.level.cell(x, y) != Cell::Wall
                    || !buildings.iter().any(|b| b.look.id == room.0)
                    || windows.at(&[x, y]) % 100 >= 30
                {
                    continue;
                }
                let same = |nx, ny| self.level.room(nx, ny) == Some(room);
                let plain = |nx, ny| same(nx, ny) && self.level.cell(nx, ny) == Cell::Wall;
                let fits = STEPS.iter().any(|&(dx, dy)| {
                    self.level.is_walkable(x + dx, y + dy)
                        && same(x - dx, y - dy)
                        && plain(x + dy, y + dx)
                        && plain(x - dy, y - dx)
                });
                if fits {
                    self.level.set(x, y, Cell::BlindWindow);
                }
            }
        }
    }
}

/// A plot's corner on a road, and the ways along the road and back from it.
struct Frame {
    corner: (i64, i64),
    along: (i64, i64),
    inward: (i64, i64),
}

impl Frame {
    /// The cell `a` along the road and `d` back from it.
    fn at(&self, a: i64, d: i64) -> (i64, i64) {
        (
            self.corner.0 + self.along.0 * a + self.inward.0 * d,
            self.corner.1 + self.along.1 * a + self.inward.1 * d,
        )
    }

    /// The way out towards the road.
    fn out(&self) -> (i64, i64) {
        (-self.inward.0, -self.inward.1)
    }
}

/// Cells as rectangles that don't overlap: runs along each row, merged with
/// the same run on the row before.
fn rects_of(cells: &[(i64, i64)]) -> Vec<Rect> {
    let mut sorted: Vec<(i64, i64)> = cells.iter().map(|&(x, y)| (y, x)).collect();
    sorted.sort_unstable();
    let mut rects: Vec<Rect> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let (y, x) = sorted[i];
        let mut end = i + 1;
        while end < sorted.len() && sorted[end] == (y, x + (end - i) as i64) {
            end += 1;
        }
        let width = (end - i) as i64;
        match rects
            .iter_mut()
            .find(|r| r.x == x && r.width == width && r.y + r.height == y)
        {
            Some(rect) => rect.height += 1,
            None => rects.push(Rect {
                x,
                y,
                width,
                height: 1,
            }),
        }
        i = end;
    }
    rects
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn town(target: i64, seed: u64) -> SiteLevel {
        Town::new(Seed::new(seed), target).generate()
    }

    /// Towns of every size, from a few seeds.
    fn towns() -> impl Iterator<Item = (i64, SiteLevel)> {
        [3, 5, 8, 12, 20, 30, 50, 80, 120, 150]
            .into_iter()
            .flat_map(|target| (0..12).map(move |seed| (target, town(target, seed * 31 + 7))))
    }

    /// The cells of a building.
    fn cells_of(look: &BuildingLook) -> Vec<(i64, i64)> {
        look.rects
            .iter()
            .flat_map(|r| {
                (r.y..r.y + r.height).flat_map(move |y| (r.x..r.x + r.width).map(move |x| (x, y)))
            })
            .collect()
    }

    #[test]
    fn the_same_seed_makes_the_same_town() {
        assert_eq!(town(40, 5), town(40, 5));
        assert_ne!(town(40, 5), town(40, 6));
    }

    #[test]
    fn towns_have_about_the_buildings_asked_for() {
        for (target, site) in towns() {
            let count = site.buildings.len() as i64;
            assert!(count >= 3, "{target}: only {count}");
            assert!(
                count <= target && count * 10 >= target * 6,
                "{target}: {count}"
            );
        }
    }

    /// No building has a single cell sticking out, and each holds a 3×3.
    #[test]
    fn buildings_are_solid_shapes() {
        for (_, site) in towns() {
            for building in &site.buildings {
                let cells = cells_of(&building.look);
                let has = |x, y| cells.contains(&(x, y));
                let square =
                    |x, y, side: i64| (0..side).all(|d| (0..side).all(|a| has(x + a, y + d)));
                for &(x, y) in &cells {
                    assert!(
                        [(0, 0), (-1, 0), (0, -1), (-1, -1)]
                            .iter()
                            .any(|(dx, dy)| square(x + dx, y + dy, 2)),
                        "a single cell sticks out at ({x}, {y})"
                    );
                    assert_eq!(site.level.room(x, y), Some(RoomId(building.look.id)));
                }
                assert!(cells.iter().any(|&(x, y)| square(x, y, 3)));
            }
        }
    }

    #[test]
    fn gardens_hold_a_three_by_three_and_nothing_sticks_out() {
        let mut found = 0;
        for (_, site) in towns() {
            let level = &site.level;
            let (x0, y0, w, h) = level.bounds();
            let is = |x, y| level.cell(x, y) == Cell::Garden;
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    if is(x, y) {
                        found += 1;
                        assert!([(0, 0), (-1, 0), (0, -1), (-1, -1)].iter().any(|(dx, dy)| {
                            is(x + dx, y + dy)
                                && is(x + dx + 1, y + dy)
                                && is(x + dx, y + dy + 1)
                                && is(x + dx + 1, y + dy + 1)
                        }));
                    }
                }
            }
        }
        assert!(found > 0);
    }

    /// Doors stand in straight walls facing something walkable, a road unless
    /// the building is a house.
    #[test]
    fn doors_face_the_right_ground() {
        for (_, site) in towns() {
            let level = &site.level;
            for building in &site.buildings {
                let (x, y) = building.look.door;
                let (dx, dy) = building.look.faces;
                assert_eq!(level.cell(x, y), Cell::BuildingDoor);
                let front = level.cell(x + dx, y + dy);
                assert!(front.is_walkable());
                if building.look.kind != BuildingKind::House {
                    assert!(front.is_road(), "{:?}", building.look.kind);
                }
                let same = |nx, ny| level.room(nx, ny) == Some(RoomId(building.look.id));
                assert!(same(x + dy, y + dx) && same(x - dy, y - dx));
                assert!(same(x - dx, y - dy));
            }
        }
    }

    /// All walkable cells can be walked to, by steps along the grid, so
    /// anything surrounded by buildings has a gap, and the way in is on the
    /// edge of the level.
    #[test]
    fn everything_can_be_reached() {
        for (target, site) in towns() {
            let entrance = site.entrance.unwrap();
            assert!(super::super::leads_out(&site.level, entrance.0, entrance.1));
            let mut reach = Reach::new(&site.level);
            reach.spread(&site.level, &[entrance]);
            assert_eq!(reach.first_unreached(&site.level), None, "{target}");
        }
    }

    #[test]
    fn numbers_stay_in_range_and_grow_with_the_town() {
        let top = |target| {
            town(target, 3)
                .buildings
                .iter()
                .map(|b| b.trades)
                .max()
                .unwrap()
        };
        assert!(top(150) > top(5));
        for (_, site) in towns() {
            for building in &site.buildings {
                assert!(building.quality <= 1000);
                assert!((10..=1000).contains(&building.trades));
            }
        }
    }

    #[test]
    fn building_kinds_follow_their_rules() {
        for (_, site) in towns() {
            let count = site.buildings.len() as i64;
            let mut copies: BTreeMap<BuildingKind, i64> = BTreeMap::new();
            for building in &site.buildings {
                *copies.entry(building.look.kind).or_default() += 1;
                assert!(building.look.kind.odds().0 <= count);
            }
            // Houses are at least about their share.
            let houses = copies.get(&BuildingKind::House).copied().unwrap_or(0);
            assert!(houses * 100 >= count * HOUSE_PERCENT - 50);
            // Nothing else takes over: halving spreads them out.
            for (&kind, &had) in &copies {
                if kind != BuildingKind::House {
                    assert!(had <= (count / 8).max(4), "{kind:?}: {had} of {count}");
                }
            }
            // Every town has a shrine or an inn, if it has more than houses.
            if copies.len() > 1 {
                assert!(
                    copies.contains_key(&BuildingKind::Shrine)
                        || copies.contains_key(&BuildingKind::Inn)
                );
            }
        }
    }

    #[test]
    fn roads_are_better_in_bigger_towns() {
        let good = |target| {
            (0..40)
                .filter(|&seed| {
                    let site = town(target, seed);
                    let (x, y) = site.entrance.unwrap();
                    matches!(
                        site.level.cell(x, y),
                        Cell::Cobblestone | Cell::Flagstones | Cell::DressedStone
                    )
                })
                .count()
        };
        assert!(good(150) > good(4));
    }

    #[test]
    fn big_towns_always_have_a_square_and_small_ones_never() {
        let road_in_a_block = |site: &SiteLevel| {
            let level = &site.level;
            let (x0, y0, w, h) = level.bounds();
            (y0..y0 + h - 5).any(|y| {
                (x0..x0 + w - 5)
                    .any(|x| (0..6).all(|d| (0..6).all(|a| level.cell(x + a, y + d).is_road())))
            })
        };
        for seed in 0..10 {
            assert!(road_in_a_block(&town(60, seed)));
            assert!(!road_in_a_block(&town(5, seed)));
        }
    }

    /// Nobody has too far to walk round: a line of building only passes the
    /// limit by the one building that was last to join it.
    #[test]
    fn long_lines_of_building_have_gaps() {
        const MOST: i64 = LONGEST_WITHOUT_GAP - 1 + 9;
        for (target, site) in towns() {
            let level = &site.level;
            let (x0, y0, w, h) = level.bounds();
            let longest = |cells: Vec<Vec<(i64, i64)>>| {
                cells
                    .iter()
                    .map(|line| {
                        line.split(|&(x, y)| !level.cell(x, y).is_wall())
                            .map(|run| run.len() as i64)
                            .max()
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0)
            };
            let rows = (y0..y0 + h)
                .map(|y| (x0..x0 + w).map(|x| (x, y)).collect())
                .collect();
            let columns = (x0..x0 + w)
                .map(|x| (y0..y0 + h).map(|y| (x, y)).collect())
                .collect();
            assert!(longest(rows) <= MOST, "{target}");
            assert!(longest(columns) <= MOST, "{target}");
        }
    }

    /// Roads don't run over each other, or side by side with no room for a
    /// building, unless they are all the main roads and square.
    #[test]
    fn roads_keep_apart() {
        for target in [12, 30, 60, 100, 150] {
            for seed in 0..12 {
                let mut town = Town::new(Seed::new(seed), target);
                town.lay_out();
                let roads = &town.roads;
                for (i, a) in roads.iter().enumerate() {
                    for b in &roads[i + 1..] {
                        if a.fixed && b.fixed {
                            continue;
                        }
                        let (m, n) = (a.area(a.along.0, a.along.1), b.area(b.along.0, b.along.1));
                        let overlap = m.x0 < n.x1 && n.x0 < m.x1 && m.y0 < n.y1 && n.y0 < m.y1;
                        assert!(!overlap, "{a:?} over {b:?}");
                        if a.east_west == b.east_west
                            && a.along.0 <= b.along.1
                            && b.along.0 <= a.along.1
                        {
                            assert!(
                                a.across.0 >= b.across.1 + BETWEEN_ROADS
                                    || b.across.0 >= a.across.1 + BETWEEN_ROADS,
                                "{a:?} beside {b:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// A road running up to the border carries on to the level's edge, unless
    /// it meets another road on the way.
    #[test]
    fn roads_at_the_border_reach_the_edge() {
        for target in [8, 20, 50, 80, 150] {
            for seed in 0..12 {
                let mut town = Town::new(Seed::new(seed), target);
                let (arms, middle, main) = town.lay_out();
                let roads = town.roads.clone();
                let site = town.finish(arms, middle, main);
                let level = &site.level;
                let (x0, y0, width, height) = level.bounds();
                let on_a_road = |x: i64, y: i64| {
                    roads.iter().any(|r| {
                        let area = r.area(r.along.0, r.along.1);
                        (area.x0..area.x1).contains(&x) && (area.y0..area.y1).contains(&y)
                    })
                };
                for road in &roads {
                    let (first, last) = if road.east_west {
                        (x0, x0 + width)
                    } else {
                        (y0, y0 + height)
                    };
                    // Out from its end: every section road, up to one that
                    // another road crosses or the edge.
                    let stops_short = |way: Vec<i64>| {
                        way.into_iter().any(|a| {
                            let area = road.area(a, a + 1);
                            let cells: Vec<(i64, i64)> = (area.y0..area.y1)
                                .flat_map(|y| (area.x0..area.x1).map(move |x| (x, y)))
                                .collect();
                            !cells.iter().any(|&(x, y)| on_a_road(x, y))
                                && cells.iter().any(|&(x, y)| !level.cell(x, y).is_road())
                        })
                    };
                    let crossed = |way: &[i64]| {
                        way.iter().position(|&a| {
                            let area = road.area(a, a + 1);
                            (area.y0..area.y1).any(|y| (area.x0..area.x1).any(|x| on_a_road(x, y)))
                        })
                    };
                    let before = |way: Vec<i64>| {
                        let stop = crossed(&way).unwrap_or(way.len());
                        way[..stop].to_vec()
                    };
                    if road.along.0 <= first + BORDER {
                        let way = before((first..road.along.0).rev().collect());
                        assert!(!stops_short(way), "{target}, {seed}: {road:?}");
                    }
                    if road.along.1 >= last - BORDER {
                        let way = before((road.along.1..last).collect());
                        assert!(!stops_short(way), "{target}, {seed}: {road:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn neighbouring_buildings_do_not_join() {
        let mut level = Level::new(0, 0, 5, 3);
        for x in 0..4 {
            level.set(x, 1, Cell::Wall);
            level.set_room(x, 1, Some(RoomId(u16::from(x >= 2))));
        }
        assert!(level.wall_shape(0, 1).unwrap().east);
        assert!(!level.wall_shape(1, 1).unwrap().east);
        assert!(!level.wall_shape(2, 1).unwrap().west);
    }
}
