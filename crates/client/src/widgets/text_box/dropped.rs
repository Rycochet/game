//! Items dropped on a box that takes them (`common::DroppedOnText`, from
//! `inventory`): written in at its caret, in place of anything selected. The
//! item itself goes back where it was.

use bevy::prelude::*;
use shared::item_text::item_token;

use super::keys::{current, keep};
use super::{Cursor, TextBox, TextEdited};
use crate::common::DroppedOnText;

pub(in crate::widgets) fn take_dropped_items(
    mut dropped: MessageReader<DroppedOnText>,
    mut boxes: Query<(&mut TextBox, &mut Cursor)>,
    mut edited_boxes: MessageWriter<TextEdited>,
    time: Res<Time<Real>>,
) {
    for drop in dropped.read() {
        let Ok((mut text_box, mut cursor)) = boxes.get_mut(drop.on) else {
            continue;
        };
        if !text_box.accepts_items {
            continue;
        }
        let mut edited = current(&text_box, &cursor);
        edited.insert(&item_token(drop.item));
        if keep(edited, &mut text_box, &mut cursor, time.elapsed_secs()) {
            edited_boxes.write(TextEdited(drop.on));
        }
    }
}
