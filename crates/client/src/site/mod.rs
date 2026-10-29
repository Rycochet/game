//! Moving between maps: entering a world map feature's site, taking stairs
//! between its levels, and leaving again. See `docs/context/client/site.md`.
//!
//! Clicking a `Portal` token (a world map feature) sends the player
//! travelling to it; arriving there enters its site. Inside, arriving (by a
//! click) on a cell with stairs takes them, and arriving on outside ground at
//! the very edge of the level leaves for the world map. The levels themselves
//! come from `shared::site`; `dungeon_map` draws them, stairs included.

use bevy::prelude::*;
use shared::feature::Feature;
use shared::site::{Exit, Site, SiteLevel, leads_out};

use crate::common::{CurrentDungeon, GameView, MapPosition, Portal};
use crate::player::{Arrived, Player, PlayerSystems, TravelTo};
use crate::view3d::{ClickedToken, View3dSystems};

pub struct SitePlugin;

impl Plugin for SitePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EnterFeature>()
            .add_message::<TakeExit>()
            .init_resource::<Heading>()
            .init_resource::<WorldReturn>()
            .add_systems(
                Update,
                follow_clicked_portal
                    .in_set(PlayerSystems::Input)
                    .after(View3dSystems::Click),
            )
            // After everything else this frame, so the view being left
            // never sees the new position (the view changes next frame).
            .add_systems(PostUpdate, (use_on_arrival, change_map).chain());
    }
}

/// Enter a feature's site, from the world map (or anywhere: dev tools).
#[derive(Message, Clone, Copy, Debug)]
pub struct EnterFeature(pub Feature);

/// Leave the current level: to another level of the site, or to the world
/// map.
#[derive(Message, Clone, Copy, Debug)]
pub struct TakeExit(pub Exit);

/// The portal the player is travelling to, to use on arrival.
#[derive(Resource, Default)]
struct Heading(Option<Entity>);

/// Where the player stood on the world map before entering a site.
#[derive(Resource, Default)]
struct WorldReturn(MapPosition);

/// A click on a portal's token: travel to it, to use it there.
fn follow_clicked_portal(
    clicked: Res<ClickedToken>,
    portals: Query<&MapPosition, With<Portal>>,
    mut heading: ResMut<Heading>,
    mut travel: MessageWriter<TravelTo>,
) {
    let Some(entity) = clicked.0 else {
        return;
    };
    if let Ok(position) = portals.get(entity) {
        heading.0 = Some(entity);
        travel.write(TravelTo {
            x: position.x,
            y: position.y,
        });
    }
}

/// When a journey ends: on the portal it was heading for, enter it; in a
/// site, on stairs take them, and on outside ground at the edge of the level
/// leave for the world map.
fn use_on_arrival(
    mut arrivals: MessageReader<Arrived>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    portals: Query<(&MapPosition, &Portal)>,
    mut heading: ResMut<Heading>,
    mut enter: MessageWriter<EnterFeature>,
    mut exits: MessageWriter<TakeExit>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    let target = heading.0.take().and_then(|entity| portals.get(entity).ok());
    match target {
        Some((position, Portal::Feature(feature))) if (position.x, position.y) == (x, y) => {
            enter.write(EnterFeature(*feature));
        }
        _ => {
            if *view.get() != GameView::Dungeon || dungeon.site.is_none() {
                return;
            }
            if let Some(stairs) = dungeon.stairs.iter().find(|s| (s.x, s.y) == (x, y)) {
                exits.write(TakeExit(stairs.to));
            } else if leads_out(&dungeon.level, x, y) {
                exits.write(TakeExit(Exit::WorldMap));
            }
        }
    }
}

/// Enters sites and moves between levels: generates the level and puts the
/// player where they arrive.
fn change_map(
    mut enter: MessageReader<EnterFeature>,
    mut exits: MessageReader<TakeExit>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut world_return: ResMut<WorldReturn>,
    mut next: ResMut<NextState<GameView>>,
    view: Res<State<GameView>>,
    mut player: Single<&mut MapPosition, With<Player>>,
) {
    // Entering comes first, so dev tools can enter and change level at once.
    let mut changes: Vec<Change> = enter.read().map(|e| Change::Enter(e.0)).collect();
    changes.extend(exits.read().map(|e| Change::Exit(e.0)));
    if changes.is_empty() {
        return;
    }
    for change in changes {
        match change {
            Change::Enter(feature) => {
                if *view.get() == GameView::WorldMap {
                    world_return.0 = **player;
                }
                dungeon.site = Some(Site::new(feature.kind, feature.seed));
                load_level(&mut dungeon, 0, Exit::WorldMap, &mut player);
                if *view.get() != GameView::Dungeon {
                    next.set(GameView::Dungeon);
                }
            }
            Change::Exit(Exit::WorldMap) => {
                dungeon.site = None;
                dungeon.stairs.clear();
                **player = world_return.0;
                if *view.get() != GameView::WorldMap {
                    next.set(GameView::WorldMap);
                }
            }
            Change::Exit(Exit::Level(index)) => {
                let from = Exit::Level(dungeon.index);
                load_level(&mut dungeon, index, from, &mut player);
            }
        }
    }
}

enum Change {
    Enter(Feature),
    Exit(Exit),
}

/// Makes level `index` of the current site current, with the player where
/// they arrive coming from `from`.
fn load_level(dungeon: &mut CurrentDungeon, index: u32, from: Exit, player: &mut MapPosition) {
    let Some(site) = &dungeon.site else {
        return;
    };
    let index = index.min(site.levels() - 1);
    let site_level = site.level(index);
    let (x, y) = arrival(&site_level, from);
    *player = MapPosition { x, y };
    dungeon.index = index;
    dungeon.level = site_level.level;
    dungeon.stairs = site_level.stairs;
}

/// Where the player arrives from `from`; if nothing leads there (a jump
/// between levels, from dev tools), the first stairs or the entrance.
fn arrival(site_level: &SiteLevel, from: Exit) -> (i64, i64) {
    site_level
        .arrival(from)
        .or_else(|| site_level.stairs.first().map(|s| (s.x, s.y)))
        .or(site_level.entrance)
        .unwrap_or((0, 0))
}
