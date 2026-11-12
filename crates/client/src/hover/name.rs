//! Deciding what the hover box says.

use bevy::prelude::*;

use super::spawn::{LIFT, LOWEST};
use crate::common::{
    BottomHud, CurrentWorld, HoverName, HoveredBag, HoveredInSky, HoveredOnMap, KnownPois, Portal,
    StaffMark, poi_name,
};
use crate::dom::Dom;
use crate::minimap::FullMap;
use crate::view3d::HoveredToken;

/// What the pointer is over, from what each module found, if anything.
#[derive(Clone, Copy, Default)]
pub(super) struct Found<'a> {
    /// A sun or moon by the minimap.
    pub(super) sky: Option<&'a str>,
    /// What the hovered token is.
    pub(super) token: Option<(Option<&'a Portal>, Option<&'a HoverName>)>,
    pub(super) bag: bool,
    /// A door or stairs on the site's map.
    pub(super) place: Option<&'a str>,
    /// The pois known, for the names learned of them.
    pub(super) pois: Option<&'a KnownPois>,
}

/// The name to show: the sky first, as it is drawn over the map, then a token
/// standing on the map, then a bag, then the map itself.
pub(super) fn name_of(found: Found) -> Option<String> {
    let token = found.token.and_then(|(portal, name)| match (portal, name) {
        (Some(Portal::Poi(poi)), _) => {
            let name = found.pois.and_then(|pois| pois.name_of(poi.chunk));
            Some(poi_name(poi, name))
        }
        (Some(Portal::Gate(_)), _) => Some("World Gate".to_owned()),
        (None, Some(name)) => Some(name.0.clone()),
        (None, None) => None,
    });
    found
        .sky
        .map(str::to_owned)
        .or(token)
        .or_else(|| found.bag.then(|| "Bag".to_owned()))
        .or_else(|| found.place.map(str::to_owned))
}

/// The role a hovered player is shown as, when their name is what the box
/// shows (nothing in the sky is in front of them).
pub(super) fn mark_of<'a>(found: Found, mark: Option<&'a StaffMark>) -> Option<&'a str> {
    let named = found.sky.is_none() && matches!(found.token, Some((None, Some(_))));
    mark.filter(|_| named).map(|mark| mark.0.as_str())
}

/// What the box last showed: its words, whether its mark shows, and how far
/// up it sits.
#[derive(Default, PartialEq)]
pub(super) struct Shown {
    name: Option<String>,
    marked: bool,
    lift: f32,
}

/// Shows the box with the name of what the pointer is over, or hides it. The
/// full map names things itself, so the box hides while it is open.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_name(
    full: Res<FullMap>,
    hovered: Res<HoveredToken>,
    tokens: Query<(Option<&Portal>, Option<&HoverName>)>,
    marks: Query<&StaffMark>,
    (sky, bag, place, world): (
        Res<HoveredInSky>,
        Res<HoveredBag>,
        Res<HoveredOnMap>,
        Option<Res<CurrentWorld>>,
    ),
    bottom_hud: Res<BottomHud>,
    mut dom: Dom,
    mut shown: Local<Option<Shown>>,
) {
    let found = Found {
        sky: sky.0.as_deref(),
        token: hovered.0.and_then(|token| tokens.get(token).ok()),
        bag: bag.0.is_some(),
        place: place.0.as_deref(),
        pois: world.as_deref().map(|world| &world.pois),
    };
    let mark = mark_of(found, hovered.0.and_then(|token| marks.get(token).ok()));
    let name = (!full.open).then(|| name_of(found)).flatten();
    let name = match (name, mark) {
        (Some(name), Some(role)) => Some(format!("{name} ({role})")),
        (name, _) => name,
    };
    let now = Shown {
        name,
        marked: mark.is_some(),
        lift: LOWEST.max(bottom_hud.0 + LIFT),
    };
    // Only what differs from what it shows is changed.
    let was = shown.as_ref();
    if was.is_none_or(|was| was.lift != now.lift) {
        dom.set("hover/row", "bottom", &now.lift.to_string());
    }
    if was.is_none_or(|was| was.marked != now.marked) {
        dom.set_flag("hover/mark", "hidden", !now.marked);
    }
    if was.is_none_or(|was| was.name.is_some() != now.name.is_some()) {
        dom.set_flag("hover/row", "hidden", now.name.is_none());
    }
    if let Some(name) = &now.name
        && was.is_none_or(|was| was.name.as_ref() != Some(name))
    {
        dom.set_text("hover/words", name.clone());
    }
    *shown = Some(now);
}
