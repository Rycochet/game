//! Deciding what the hover box says.

use bevy::prelude::*;

use super::spawn::{HoverBox, HoverMark, HoverText, LIFT, LOWEST};
use crate::common::{
    BottomHud, CurrentWorld, HoverName, HoveredBag, HoveredInSky, HoveredOnMap, KnownPois, Portal,
    StaffMark, poi_name,
};
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

/// Shows the box with the name of what the pointer is over, or hides it. The
/// full map names things itself, so the box hides while it is open.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_name(
    full: Res<FullMap>,
    hovered: Res<HoveredToken>,
    tokens: Query<(Option<&Portal>, Option<&HoverName>)>,
    marks: Query<&StaffMark>,
    mut picture: Single<&mut Node, (With<HoverMark>, Without<HoverBox>)>,
    (sky, bag, place, world): (
        Res<HoveredInSky>,
        Res<HoveredBag>,
        Res<HoveredOnMap>,
        Option<Res<CurrentWorld>>,
    ),
    bottom_hud: Res<BottomHud>,
    mut holder: Single<(&mut Visibility, &mut Node), With<HoverBox>>,
    mut text: Single<&mut Text, With<HoverText>>,
) {
    let (shown_now, node) = &mut *holder;
    let lift = LOWEST.max(bottom_hud.0 + LIFT);
    if node.bottom != px(lift) {
        node.bottom = px(lift);
    }
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
    let display = if mark.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    if picture.display != display {
        picture.display = display;
    }
    shown_now.set_if_neq(if name.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if let Some(name) = name
        && text.0 != name
    {
        text.0 = name;
    }
}
