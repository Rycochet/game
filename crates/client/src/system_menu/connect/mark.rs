//! The mark just left of Connect's Connect button: a spinner while the
//! address is being checked, then a green tick for a server this game can play
//! on or a red cross, with the reason on hovering.

use bevy::prelude::*;

use super::super::icons::ConnectIcons;
use super::check::{AddressCheck, Checked};
use crate::widgets::{HoverTip, Spinner};

/// The mark's size, in pixels.
const MARK: f32 = 24.0;

/// The mark, holding the spinner and the picture.
#[derive(Component)]
pub(in crate::system_menu) struct AddressMark;

#[derive(Component)]
pub(in crate::system_menu) struct MarkSpinner;

#[derive(Component)]
pub(in crate::system_menu) struct MarkPicture;

/// The spinner and the picture, in the mark (a `canvas` in Connect's
/// markup, just left of its Connect button).
pub(super) fn spawn_mark_parts(mark: &mut ChildSpawnerCommands) {
    let part = || Node {
        width: px(MARK),
        height: px(MARK),
        position_type: PositionType::Absolute,
        display: Display::None,
        ..default()
    };
    mark.spawn((MarkSpinner, Spinner, part()));
    mark.spawn((MarkPicture, ImageNode::default(), part()));
}

/// Shows where the check has got.
#[allow(clippy::type_complexity)]
pub(in crate::system_menu) fn show_mark(
    check: Res<AddressCheck>,
    icons: Res<ConnectIcons>,
    mut tip: Single<&mut HoverTip, With<AddressMark>>,
    mut spinner: Single<&mut Node, (With<MarkSpinner>, Without<MarkPicture>)>,
    mut picture: Single<(&mut Node, &mut ImageNode), (With<MarkPicture>, Without<MarkSpinner>)>,
) {
    let (spinning, shown, words) = match &check.checked {
        Checked::Waiting => (false, None, String::new()),
        Checked::Checking => (true, None, "Checking for a server…".to_owned()),
        Checked::Found { version, .. } => (
            false,
            Some(&icons.tick),
            format!("Hall of Worlds {}", version.version),
        ),
        Checked::Failed(reason) => (false, Some(&icons.cross), reason.clone()),
    };
    let display = |on: bool| if on { Display::Flex } else { Display::None };
    if spinner.display != display(spinning) {
        spinner.display = display(spinning);
    }
    let (node, image) = &mut *picture;
    if node.display != display(shown.is_some()) {
        node.display = display(shown.is_some());
    }
    if let Some(shown) = shown
        && image.image != *shown
    {
        image.image = shown.clone();
    }
    if tip.0 != words {
        tip.0 = words;
    }
}
