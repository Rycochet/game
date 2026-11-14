//! Where the maps are on screen: which shows, their sizes and the full map's
//! title, told to their elements only when they change.

use bevy::prelude::*;

use super::{
    FULL_MAP_MARGIN, Room, Sources, full_map_area, full_map_corner, full_map_side, location_title,
    minimap_across, minimap_picture_size,
};
use crate::dom::Dom;
use crate::widgets::CLOSE_PIXELS;

/// How far down the minimap's top edge is, as in its markup.
const MINIMAP_TOP: f32 = 8.0;

/// Where the minimap is, in interface pixels from the window's top right
/// corner (`right` 8, as in its markup), for what goes under it: its picture's
/// bottom edge and size, and whether it shows (not while the full map is
/// open).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct MinimapArea {
    pub bottom: f32,
    pub across: f32,
    pub shown: bool,
}

/// What the maps' elements were last told.
#[derive(Clone, Default, PartialEq)]
pub(super) struct Placed {
    open: bool,
    /// The minimap's picture, across, and its round face.
    minimap: f32,
    face: f32,
    /// The parchment: its top left corner, and how far across.
    corner: Vec2,
    side: f32,
    /// The full map's picture.
    area: Vec2,
    title: String,
}

/// Shows the minimap or the full map, and keeps their sizes, the full map's
/// place in the middle of the window and its title.
pub(super) fn place_maps(
    sources: Sources,
    mut dom: Dom,
    mut area: ResMut<MinimapArea>,
    mut was: Local<Option<Placed>>,
) {
    let window = Room::of(&sources.window, &sources.ui_scale);
    let open = sources.full.open;
    let mut now = was.clone().unwrap_or_default();
    now.open = open;
    now.minimap = minimap_picture_size(&window);
    now.face = minimap_across(&window);
    now.corner = full_map_corner(&window);
    now.side = full_map_side(&window);
    now.area = full_map_area(&window);
    // (Only while it shows: the world map's names the land the player is on.)
    if open {
        now.title = location_title(
            *sources.view.get(),
            sources.world.as_deref(),
            sources.levels.shown(&sources.dungeon),
            *sources.first_floor,
            &sources.player,
        );
    }
    if was.as_ref() == Some(&now) {
        return;
    }
    let shown = MinimapArea {
        bottom: MINIMAP_TOP + now.minimap,
        across: now.minimap,
        shown: !now.open,
    };
    if *area != shown {
        *area = shown;
    }
    tell(&mut dom, was.as_ref(), &now);
    *was = Some(now);
}

/// Changes the elements, only where `now` differs from what they were told.
fn tell(dom: &mut Dom, was: Option<&Placed>, now: &Placed) {
    let differs = |pick: fn(&Placed) -> Vec2| was.is_none_or(|was| pick(was) != pick(now));
    let size = |dom: &mut Dom, id: &str, size: Vec2| {
        dom.set(id, "width", &size.x.to_string());
        dom.set(id, "height", &size.y.to_string());
    };
    let place = |dom: &mut Dom, id: &str, at: Vec2| {
        dom.set(id, "left", &at.x.to_string());
        dom.set(id, "top", &at.y.to_string());
    };
    if was.is_none_or(|was| was.open != now.open) {
        dom.set_flag("minimap/map", "hidden", now.open);
        dom.set_flag("minimap/disc", "hidden", now.open);
        dom.set_flag("full_map/parchment", "hidden", !now.open);
        dom.set_flag("full_map/close", "hidden", !now.open);
    }
    if differs(|placed| Vec2::new(placed.minimap, placed.face)) {
        size(dom, "minimap/map", Vec2::splat(now.minimap));
        // The face, in the middle of the picture.
        size(dom, "minimap/disc", Vec2::splat(now.face));
        let inset = (MINIMAP_TOP + (now.minimap - now.face) / 2.0).to_string();
        dom.set("minimap/disc", "top", &inset);
        dom.set("minimap/disc", "right", &inset);
    }
    if differs(|placed| Vec2::splat(placed.side)) {
        size(dom, "full_map/parchment", Vec2::splat(now.side));
    }
    if differs(|placed| placed.corner) || differs(|placed| Vec2::splat(placed.side)) {
        place(dom, "full_map/parchment", now.corner);
        // The close button, in the parchment's top right corner.
        let across = now.side - FULL_MAP_MARGIN - CLOSE_PIXELS;
        let close = now.corner + Vec2::new(across, FULL_MAP_MARGIN);
        place(dom, "full_map/close", close);
    }
    if differs(|placed| placed.area) {
        size(dom, "full_map/picture", now.area);
    }
    if was.is_none_or(|was| was.title != now.title) {
        dom.set_text("full_map/title", now.title.clone());
    }
}
