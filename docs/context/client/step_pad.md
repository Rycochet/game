# client: step_pad

The movement pad (owner's design, 2026-11-08): a grip panel at the bottom right, open by default, with hold buttons in three rows round the middle like a d-pad, Down · Step Up · Up, Step Left · gap · Step Right, gap · Step Down · gap, and the small d-pad button in the corner that opens and closes it. Whether it is open and where it was dragged to are kept in `Preferences::step_pad`. It only spawns, shows and reads the pad: what Up and Down do, and when they are enabled, is `site`'s, and steps are `player`'s.

## Files

- `crates/client/src/step_pad/mod.rs`: `StepPadPlugin`; sizes for the tests (`TOGGLE` 30, `BUTTON` 44 for fingers, `GAP`, `CORNER_GAP`: the markup says them), form `step_pad`.
- `crates/client/src/step_pad/spawn.rs`: `spawn_pad` (Startup): names the pictures and spawns two documents. `TOGGLE_BUTTON` is the toggle (`hud/step_pad`, a `button` with its own `form`, placed from the bottom right on a `bottom_strip()` above the maps, hidden until on a map). `PAD` is the pad: a `panel grip` (`step_pad/panel`), by its `right` and `bottom` just above the toggle, or at the place saved, which `spawn_pad` sets as `left` and `top` in `%` (a share of the window); in it three rows of buttons, each holding an `img` of its picture, `picture:pad_<name>`, with gaps a button in size. `BUTTONS` (its buttons `step_pad/up`, `step_up`, `down`, `step_left`, `step_down`, `step_right`), `bind_pad` (each, once built, gets its `common::PadButton`, a `widgets::HoldButton` and its `HoverTip`; the panel gets `StepPad`, the toggle `PadToggle` and its tip). Unit tests.
- `crates/client/src/step_pad/show.rs`: `toggle_pad` (flips `Preferences::step_pad.closed`), `remember_place` (on `widgets::PanelMoved` for the pad, its share of the window into `Preferences::step_pad.at`: only when a drag lets go, so the file isn't saved every frame), `show_pad` (both only in `GameView::WorldMap` / `Dungeon`, by their elements' `hidden` through `Dom`, told only when it changes; hidden, the pad keeps its place; the toggle `Active` while open), `tint_icons` (white pictures in `INK`, faint when disabled), `read_pad` (`PreUpdate`, after `dom::DomInput::Presses`: held arrows into `common::PadSteps`).
- `crates/client/src/step_pad/icons.rs`: the pictures, drawn in code: `dpad` (a cross with rounded ends and a triangle cut into each arm), `arrow(Pointing)`, `ladder(climbing)` (rails closing in towards the far end, rungs evenly spaced in distance so they crowd together there: narrow at the top for Up, at the bottom for Down). `picture.rs`: `picture` (white, edges smoothed by 4 × 4 samples), `on_line`, `outside_box`.

## Public API

- `StepPadPlugin`. What others read is in `common`: `PadButton` (on each pad button), `PadSteps` (resource: held arrows as screen `dx`, `dy`).

## Depends on

- `common` (`GameView`, `PadButton`, `PadSteps`, `Preferences::step_pad`), `dom` (`Template`, `document`, `Dom`, `Pictures`, the strips), `widgets` (`PanelMoved`, `HoldButton`, `HoverTip`, `ButtonTooltip`, `ButtonPressed`, `ButtonState`).

## Used by

- Through `common`: `player` (`PadSteps` in `move_player`, as held keys), `site` (`down_or_up` enables Up and Down by its rules, keeps their tips, and sends `Up` / `Down` on their `ButtonPressed`, sent once when one becomes held).

## Notes

- The arrows are screen directions, as the step keys, turned by the camera in `player`: on a level or in the Hall they walk freely that way (at whatever angle the camera is turned), on the world map they step. Only one finger holds at a time, so no diagonals by touch; the mouse and a finger could hold two.
- Up and Down fire once when they become held; staying held does nothing more.
