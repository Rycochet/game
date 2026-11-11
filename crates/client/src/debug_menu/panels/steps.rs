//! Drawing States: a row each for stairs, fields and herbs, with buttons moving
//! what is drawn of them on or back a step, greyed where the level has none.

use crate::widgets::UiId;
use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::site::StairsStyle;

use crate::common::{CurrentDungeon, GameView, HerbGrowthShift};
use crate::dom::Template;
use crate::dungeon_map::{CROP_STAGES, HERB_STAGES};
use crate::html;
use crate::widgets::{ButtonPressed, ButtonState};

/// What a row of the panel steps through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Drawn {
    Stairs,
    Fields,
    Herbs,
}

impl Drawn {
    const ALL: [Self; 3] = [Self::Stairs, Self::Fields, Self::Herbs];

    /// Its buttons' ids: `stairs_previous`, `stairs_next` and so on.
    fn id(self, forward: bool) -> &'static str {
        match (self, forward) {
            (Self::Stairs, false) => "stairs_previous",
            (Self::Stairs, true) => "stairs_next",
            (Self::Fields, false) => "fields_previous",
            (Self::Fields, true) => "fields_next",
            (Self::Herbs, false) => "herbs_previous",
            (Self::Herbs, true) => "herbs_next",
        }
    }
}

/// A button that moves what its row is for on or back a step.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct StepButton {
    drawn: Drawn,
    forward: bool,
}

/// The panel's contents, as markup: a table, so the buttons line up.
pub(super) static DRAWING_STATES: Template = html!(
    "debug_menu.drawing_states",
    r#"
<form id="debug.drawing_states">
  <table cellspacing="4">
    <tr>
      <td valign="middle">Stairs</td>
      <td><button id="stairs_previous">Previous</button></td>
      <td><button id="stairs_next">Next</button></td>
    </tr>
    <tr>
      <td valign="middle">Fields</td>
      <td><button id="fields_previous">Previous</button></td>
      <td><button id="fields_next">Next</button></td>
    </tr>
    <tr>
      <td valign="middle">Herbs</td>
      <td><button id="herbs_previous">Previous</button></td>
      <td><button id="herbs_next">Next</button></td>
    </tr>
  </table>
</form>
"#
);

/// Which row a built button steps, and which way, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    for drawn in Drawn::ALL {
        for forward in [false, true] {
            if drawn.id(forward) == id.parts().1 {
                bound.insert(StepButton { drawn, forward });
            }
        }
    }
}

/// Whether a row's buttons can do anything here.
pub(super) fn can_step(drawn: Drawn, view: GameView, dungeon: &CurrentDungeon) -> bool {
    view == GameView::Dungeon
        && match drawn {
            Drawn::Stairs => !dungeon.stairs.is_empty(),
            Drawn::Fields => !dungeon.fields.is_empty(),
            Drawn::Herbs => dungeon.level_has(Cell::HerbBed),
        }
}

/// The style before this one, going round.
pub(super) fn previous_style(style: StairsStyle) -> StairsStyle {
    let all = StairsStyle::ALL;
    let at = all.iter().position(|&other| other == style).unwrap_or(0);
    all[(at + all.len() - 1) % all.len()]
}

/// Moves what the pressed button's row is for on or back a step: only what is
/// drawn, and the next level goes back to its own.
pub(in crate::debug_menu) fn press_panel_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&StepButton>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut shift: ResMut<HerbGrowthShift>,
) {
    for press in pressed.read() {
        let Ok(&StepButton { drawn, forward }) = buttons.get(press.0) else {
            continue;
        };
        match drawn {
            Drawn::Stairs => {
                for stairs in &mut dungeon.stairs {
                    stairs.style = if forward {
                        stairs.style.next()
                    } else {
                        previous_style(stairs.style)
                    };
                }
            }
            Drawn::Fields => {
                for field in &mut dungeon.fields {
                    // As crops, which alone go through every stage.
                    field.livestock = false;
                    let by = if forward { 1 } else { CROP_STAGES as u32 - 1 };
                    field.variety = field.variety.wrapping_add(by);
                }
            }
            Drawn::Herbs => {
                // Back is nearly all the way round, as the shift can't go below 0.
                let by = if forward { 1 } else { HERB_STAGES - 1 };
                shift.0 = (shift.0 + by) % HERB_STAGES;
            }
        }
    }
}

/// Greys the buttons where they can do nothing; the panel stays open.
pub(in crate::debug_menu) fn update_panel_buttons(
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut buttons: Query<(&StepButton, &mut ButtonState)>,
) {
    for (button, mut state) in &mut buttons {
        state.set_if_neq(if can_step(button.drawn, *view.get(), &dungeon) {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}
