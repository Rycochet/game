//! The mark at the right end of Connect's address box: a spinner while the
//! address is being checked, then a green tick for a server this game can play
//! on or a red cross, with the reason on hovering.

use bevy::prelude::*;

use super::super::icons::ConnectIcons;
use super::check::{AddressCheck, Checked};
use crate::widgets::{HoverTip, Spinner};

/// The mark's size, in pixels.
pub(super) const MARK: f32 = 24.0;
/// Space kept clear for it at the end of the box.
pub(super) const MARK_ROOM: f32 = MARK + 12.0;

/// The mark, holding the spinner and the picture.
#[derive(Component)]
pub(in crate::system_menu) struct AddressMark;

#[derive(Component)]
pub(in crate::system_menu) struct MarkSpinner;

#[derive(Component)]
pub(in crate::system_menu) struct MarkPicture;

/// The mark, inside the box (`box_height` tall) at its right end.
pub(super) fn spawn_mark(address_box: &mut ChildSpawnerCommands, box_height: f32) {
    // Placed from the box's padding, inside its 2-pixel border.
    let top = (box_height - 4.0 - MARK) / 2.0;
    let node = Node {
        position_type: PositionType::Absolute,
        right: px(6),
        top: px(top),
        width: px(MARK),
        height: px(MARK),
        ..default()
    };
    let part = || Node {
        width: px(MARK),
        height: px(MARK),
        position_type: PositionType::Absolute,
        display: Display::None,
        ..default()
    };
    address_box
        .spawn((AddressMark, node, HoverTip(String::new())))
        .with_children(|mark| {
            mark.spawn((MarkSpinner, Spinner, part()));
            mark.spawn((MarkPicture, ImageNode::default(), part()));
        });
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
