//! The markers over the 3D view: a "!" or "?" in its colour, ringed in black,
//! kept over what it marks by projecting it through the view's camera each
//! frame (as health bars are). They take no presses.

use bevy::prelude::*;
use bevy::text::FontWeight;

use super::anchors::{
    Anchor, MarkOver, boss_anchors, door_anchors, find_anchors, gate_anchors, poi_anchors,
};
use super::{BossMarks, DoorMarkers, FindMarks, GateQuests, MarkLook, QuestPois};
use crate::common::{CurrentDungeon, DrawnGround, GameView, Other, Portal};
use crate::view3d::{TokenView, ViewCamera};

/// How tall a marker's sign is, in interface pixels.
const SIZE: f32 = 30.0;
/// How far its black ring reaches round it.
const RING: f32 = 2.0;

/// A marker: what it is over and how it looks.
#[derive(Component)]
pub(super) struct OverMark(MarkOver, MarkLook);

/// Places a marker over everything with one in view, and takes away the rest.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_marks(
    view: Res<State<GameView>>,
    (dungeon, ground): (Res<CurrentDungeon>, Res<DrawnGround>),
    (doors, gates, pois): (Res<DoorMarkers>, Res<GateQuests>, Res<QuestPois>),
    (bosses, finds): (Res<BossMarks>, Res<FindMarks>),
    ui_scale: Res<UiScale>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    tokens: Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    (portals, others): (Query<&Portal>, Query<&Other>),
    mut marks: Query<(Entity, &OverMark, &mut Node)>,
    mut commands: Commands,
) {
    let anchors: Vec<Anchor> = match view.get() {
        GameView::Dungeon => {
            let mut all = door_anchors(&dungeon, &ground, &doors);
            all.extend(gate_anchors(&dungeon, &gates));
            all.extend(boss_anchors(&bosses, &tokens, &others));
            all.extend(find_anchors(&finds, &ground));
            all
        }
        GameView::WorldMap => poi_anchors(&pois, &tokens, &portals),
        _ => Vec::new(),
    };
    let (camera, eye) = *camera;
    let scale = ui_scale.0.max(0.01);
    let mut placed: Vec<(MarkOver, MarkLook, Node)> = (anchors.into_iter())
        .filter_map(|(over, at, look)| {
            let on_screen = camera.world_to_viewport(eye, at).ok()? / scale;
            Some((over, look, place(on_screen)))
        })
        .collect();
    for (entity, mark, mut node) in &mut marks {
        let same = (placed.iter()).position(|(over, look, _)| (*over, *look) == (mark.0, mark.1));
        match same {
            Some(index) => *node = placed.swap_remove(index).2,
            None => commands.entity(entity).despawn(),
        }
    }
    for (over, look, node) in placed {
        spawn_mark(&mut commands, OverMark(over, look), node);
    }
}

/// A marker's box, the foot of its sign on the point.
fn place(at: Vec2) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(at.x - SIZE / 2.0),
        top: px(at.y - SIZE),
        width: px(SIZE),
        height: px(SIZE),
        ..default()
    }
}

fn spawn_mark(commands: &mut Commands, mark: OverMark, node: Node) {
    let glyph = mark.1.glyph();
    let colour = mark.1.colour();
    let font = TextFont::from_font_size(SIZE).with_font_weight(FontWeight::BLACK);
    let sign = |colour: Color, (x, y): (f32, f32)| {
        (
            Text::new(glyph),
            font.clone(),
            TextColor(colour),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                left: px(x),
                top: px(y),
                width: percent(100.0),
                ..default()
            },
        )
    };
    let mut parent = commands.spawn((mark, node, Pickable::IGNORE));
    // A black ring: the sign over copies of itself moved each way.
    for offset in [(-RING, 0.0), (RING, 0.0), (0.0, -RING), (0.0, RING)] {
        parent.with_child(sign(Color::BLACK, offset));
    }
    parent.with_child(sign(colour, (0.0, 0.0)));
}
