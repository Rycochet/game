# client: step_pad

The movement pad (owner's design, 2026-11-08): a grip panel at the bottom right, open by default, with hold buttons in three rows round the middle like a d-pad, Down · Step Up · Up, Step Left · gap · Step Right, gap · Step Down · gap, and the small d-pad button in the corner that opens and closes it. Whether it is open and where it was dragged to are kept in `Preferences::step_pad`. It only spawns, shows and reads the pad: what Up and Down do, and when they are enabled, is `site`'s, and steps are `player`'s.

## Files

- `crates/client/src/step_pad/mod.rs`: `StepPadPlugin`; sizes (`TOGGLE` 30, `BUTTON` 44 for fingers, `GAP`, `CORNER_GAP`), form `step_pad`.
- `crates/client/src/step_pad/spawn.rs`: `spawn_pad` (Startup): the toggle (`hud/step_pad`, tip "Movement Pad", hidden until on a map) and the pad (`widgets::spawn_grip_panel`, at `PanelPlace::Share` of the place saved, or else `BottomRight` just above the toggle; `step_pad/panel`), `PAD` (the pad's contents as markup, in the code-spawned grip panel: three rows of buttons, each holding an `img` of its picture, `picture:pad_<name>`, with gaps a button in size), `BUTTONS` (its buttons `step_pad/up`, `step_up`, `down`, `step_left`, `step_down`, `step_right`), `bind_pad` (each, once built, gets its `common::PadButton`, a `widgets::HoldButton` and its `HoverTip`). Unit test.
- `crates/client/src/step_pad/show.rs`: `toggle_pad` (flips `Preferences::step_pad.closed`), `remember_place` (on `widgets::PanelMoved` for the pad, its share of the window into `Preferences::step_pad.at`: only when a drag lets go, so the file isn't saved every frame), `show_pad` (both only in `GameView::WorldMap` / `Dungeon`; the pad hidden by `Display::None` so it keeps its place; the toggle `Active` while open), `tint_icons` (white pictures in `INK`, faint when disabled), `read_pad` (`PreUpdate`, after `WidgetSystems::Presses`: held arrows into `common::PadSteps`).
- `crates/client/src/step_pad/icons.rs`: the pictures, drawn in code: `dpad` (a cross with rounded ends and a triangle cut into each arm), `arrow(Pointing)`, `ladder(climbing)` (rails closing in towards the far end, rungs evenly spaced in distance so they crowd together there: narrow at the top for Up, at the bottom for Down). `picture.rs`: `picture` (white, edges smoothed by 4 × 4 samples), `on_line`, `outside_box`.

## Public API

- `StepPadPlugin`. What others read is in `common`: `PadButton` (on each pad button), `PadSteps` (resource: held arrows as screen `dx`, `dy`).

## Depends on

- `common` (`GameView`, `PadButton`, `PadSteps`, `Preferences::step_pad`), `dom` (`Template`, `document_in`, `Pictures`), `widgets` (`spawn_grip_panel`, `PanelPlace`, `PanelMoved`, `HoldButton`, `spawn_icon_button`, `HoverTip`, `ButtonPressed`).

## Used by

- Through `common`: `player` (`PadSteps` in `move_player`, as held keys), `site` (`down_or_up` enables Up and Down by its rules, keeps their tips, and sends `Up` / `Down` on their `ButtonPressed`, sent once when one becomes held).

## Notes

- The arrows are screen directions, as the step keys, turned by `ViewFacing` in `player`. Only one finger holds at a time, so no diagonals by touch; the mouse and a finger could hold two.
- Up and Down fire once when they become held; staying held does nothing more.
