//! Where each quest marker goes in the 3D view: over the doors of the town's
//! givers, over the Hall's gates to planets with quests, and over pois on the
//! world map with a step to do there.

use bevy::prelude::*;
use shared::hall::{GatePlace, LEVEL_ROWS, LEVEL_TOP};
use shared::id::Id;
use shared::poi::ChunkPos;
use shared::protocol::LevelKind;
use shared::quest::QuestKind;

use super::{BossMarks, DoorMarkers, FindMarks, GateQuests, MarkLook, QuestPois};
use crate::common::{CurrentDungeon, DrawnGround, Other, Portal};
use crate::dungeon_map::CELL_SIZE;
use crate::view3d::{TOKEN_DIAMETER, TokenView};

/// How high over the ground at a door its marker floats, in cells: above the
/// door, under the eaves.
const OVER_DOOR: f32 = 1.35;
/// How high over a Hall gate's foot its marker floats, in cells: the tallest
/// designs are about 1.6.
const OVER_GATE: f32 = 1.9;
/// How high over a poi's token its marker floats, in token widths.
const OVER_POI: f32 = 1.1;
/// How high over a Medium creature's feet its marker floats, in cells: over
/// a person's head; others by their size.
const OVER_CREATURE: f32 = 1.4;
/// How high over the ground a quest item lying there has its marker, in cells.
const OVER_FIND: f32 = 0.6;

/// What a marker is over, to keep the same one as the view moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum MarkOver {
    Door(u16),
    Gate(i64, i64),
    Poi(ChunkPos),
    Creature(Id),
    Find(shared::spot::Spot),
}

/// One marker to show: what it is over, where in the view, and how it looks.
pub(super) type Anchor = (MarkOver, Vec3, MarkLook);

/// A place on the level in the view (cells, a cell's middle on its whole
/// numbers), at `up` cells over its drawn ground.
fn over_ground(ground: &DrawnGround, (x, y): (f32, f32), up: f32) -> Vec3 {
    let height = ground.drawn_between(x, y) + up;
    Vec3::new(x, height, y) * CELL_SIZE
}

/// The givers' doors with a marker, on the level shown.
pub(super) fn door_anchors(
    dungeon: &CurrentDungeon,
    ground: &DrawnGround,
    doors: &DoorMarkers,
) -> Vec<Anchor> {
    if doors.town.is_none() {
        return Vec::new();
    }
    (dungeon.buildings.iter())
        .filter_map(|building| {
            let marker = doors.markers.get(&building.id)?;
            // On the door's outside face.
            let x = building.door.0 as f32 + building.faces.0 as f32 * 0.5;
            let y = building.door.1 as f32 + building.faces.1 as f32 * 0.5;
            let at = over_ground(ground, (x, y), OVER_DOOR);
            Some((MarkOver::Door(building.id), at, MarkLook::of(*marker)))
        })
        .collect()
}

/// The Hall's gates in the stretch held that lead to planets with quests.
pub(super) fn gate_anchors(dungeon: &CurrentDungeon, gates: &GateQuests) -> Vec<Anchor> {
    let Some(info) = &dungeon.info else {
        return Vec::new();
    };
    let LevelKind::Hall { origin } = info.kind else {
        return Vec::new();
    };
    if gates.0.is_empty() {
        return Vec::new();
    }
    let (min_x, _, width, _) = info.bounds;
    let mut anchors = Vec::new();
    for y in LEVEL_TOP..LEVEL_TOP + LEVEL_ROWS {
        for x in min_x..min_x + width {
            let Some(gate) = GatePlace::at(x + origin, y) else {
                continue;
            };
            let Some(&ready) = gates.0.get(&gate.address().world().id()) else {
                continue;
            };
            let at = Vec3::new(x as f32, OVER_GATE, y as f32) * CELL_SIZE;
            let look = MarkLook::new(QuestKind::Normal, ready);
            anchors.push((MarkOver::Gate(x, y), at, look));
        }
    }
    anchors
}

/// The world map's pois in view with a quest step, over their tokens.
pub(super) fn poi_anchors(
    pois: &QuestPois,
    tokens: &Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    portals: &Query<&Portal>,
) -> Vec<Anchor> {
    if pois.0.is_empty() {
        return Vec::new();
    }
    (tokens.iter())
        .filter(|(.., shown)| shown.get())
        .filter_map(|(view, token, _)| {
            let poi = portals.get(view.owner).ok()?.poi()?;
            let look = *pois.0.get(&poi.chunk)?;
            let at = token.translation() + Vec3::Y * TOKEN_DIAMETER * OVER_POI;
            Some((MarkOver::Poi(poi.chunk), at, look))
        })
        .collect()
}

/// The quest bosses in sight with a sign, over their heads.
pub(super) fn boss_anchors(
    bosses: &BossMarks,
    tokens: &Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    others: &Query<&Other>,
) -> Vec<Anchor> {
    if bosses.0.is_empty() {
        return Vec::new();
    }
    (tokens.iter())
        .filter(|(.., shown)| shown.get())
        .filter_map(|(view, token, _)| {
            let other = others.get(view.owner).ok()?;
            let sign = *bosses.0.get(&other.id)?;
            let tall = other.kind.def().size.quarters() as f32 / 4.0;
            let at = token.translation() + Vec3::Y * CELL_SIZE * OVER_CREATURE * tall;
            Some((MarkOver::Creature(other.id), at, MarkLook::of_boss(sign)))
        })
        .collect()
}

/// The player's own quest item lying in sight, over where it lies.
pub(super) fn find_anchors(finds: &FindMarks, ground: &DrawnGround) -> Vec<Anchor> {
    let span = shared::spot::CELL_SPAN as f32;
    (finds.0.iter())
        .map(|find| {
            let cells = (find.at.x as f32 / span, find.at.y as f32 / span);
            let at = over_ground(ground, cells, OVER_FIND);
            (MarkOver::Find(find.at), at, MarkLook::of_boss(find.sign))
        })
        .collect()
}
