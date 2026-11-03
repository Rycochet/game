//! What each building is and looks like, and its windows.

use super::super::grid::{STEPS, roll};
use super::{
    BuildingKind, BuildingLook, HOUSE_PERCENT, LARGEST, Placed, RoofKind, Town, TownBuilding,
    WallKind, blend, pick, rects_of,
};
use crate::dungeon::{Cell, RoomId};
use crate::rng::Seed;

impl Town {
    /// What each building is. Houses first: all those not on a road, then
    /// others up to their share. The rest by weight, each copy of a kind
    /// halving its weight; the first is always a shrine or an inn.
    pub(super) fn choose_kinds(&mut self) -> Vec<BuildingKind> {
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
    pub(super) fn make_building(
        &mut self,
        index: usize,
        placed: &Placed,
        kind: BuildingKind,
    ) -> TownBuilding {
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
    pub(super) fn add_windows(&mut self, buildings: &[TownBuilding]) {
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
