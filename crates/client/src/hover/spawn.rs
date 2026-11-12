//! The hover box, as markup.

use bevy::prelude::*;

use crate::dom::{Template, bottom_strip, document};
use crate::html;

/// How far above the bottom of the window the box sits, or above whatever the
/// interface keeps there (`common::BottomHud`), in interface pixels.
pub(super) const LIFT: f32 = 12.0;

/// The lowest the box goes, in interface pixels (as the markup starts it).
pub(super) const LOWEST: f32 = 32.0;

/// The box: a tooltip's dark box in the middle of a row across the window,
/// with the staff mark (`common::StaffMark`: a gold laurel crown) before the
/// name of a marked player. The pointer is never on it (`inert`).
pub(super) static HOVER: Template = html!(
    "hover",
    r#"
<form id="hover">
  <div id="row" bottom="32" left="0" right="0" align="center" inert hidden>
    <span id="box" class="tip" valign="middle">
      <img id="mark" class="gold" src="icon:lorc/laurel-crown" width="20" height="20" hidden/>
      <p id="words"/>
    </span>
  </div>
</form>
"#
);

pub(super) fn spawn_box(mut commands: Commands) {
    // Along the bottom of the window, for the row to be placed from. Above
    // the maps.
    let over = GlobalZIndex(10);
    commands
        .spawn((document(&HOVER), over))
        .insert(bottom_strip());
}
