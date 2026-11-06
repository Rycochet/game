//! Deciding what the hover box says.

use bevy::prelude::*;

use super::spawn::{HoverBox, HoverText, LIFT, LOWEST};
use crate::common::{
    BottomHud, HoverName, HoveredBag, HoveredInSky, HoveredOnMap, Portal, poi_name,
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
}

/// The name to show: the sky first, as it is drawn over the map, then a token
/// standing on the map, then a bag, then the map itself.
pub(super) fn name_of(found: Found) -> Option<String> {
    let token = found.token.and_then(|(portal, name)| match (portal, name) {
        (Some(Portal::Poi(poi)), _) => Some(poi_name(poi)),
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

/// Shows the box with the name of what the pointer is over, or hides it. The
/// full map names things itself, so the box hides while it is open.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_name(
    full: Res<FullMap>,
    hovered: Res<HoveredToken>,
    tokens: Query<(Option<&Portal>, Option<&HoverName>)>,
    (sky, bag, place): (Res<HoveredInSky>, Res<HoveredBag>, Res<HoveredOnMap>),
    bottom_hud: Res<BottomHud>,
    mut holder: Single<(&mut Visibility, &mut Node), With<HoverBox>>,
    mut text: Single<&mut Text, With<HoverText>>,
) {
    let (shown_now, node) = &mut *holder;
    let lift = LOWEST.max(bottom_hud.0 + LIFT);
    if node.bottom != px(lift) {
        node.bottom = px(lift);
    }
    let name = (!full.open)
        .then(|| {
            name_of(Found {
                sky: sky.0.as_deref(),
                token: hovered.0.and_then(|token| tokens.get(token).ok()),
                bag: bag.0.is_some(),
                place: place.0.as_deref(),
            })
        })
        .flatten();
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
