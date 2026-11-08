# client: step_pad

The movement pad (owner's design, 2026-11-08): a grip panel at the bottom right, open by default, with hold buttons in two rows, Up · Step Up · Down and Step Left · Step Down · Step Right, and the small keypad button in the corner that opens and closes it. It only spawns, shows and reads the pad: what Up and Down do, and when they are enabled, is `site`'s, and steps are `player`'s.

## Files

- `crates/client/src/step_pad/mod.rs`: `StepPadPlugin`; sizes (`TOGGLE` 30, `BUTTON` 44 for fingers, `GAP`, `CORNER_GAP`), form `step_pad`.
- `crates/client/src/step_pad/spawn.rs`: `spawn_pad` (Startup): the toggle (`hud/step_pad`, tip "Movement Pad", hidden until on a map) and the pad (`widgets::spawn_grip_panel` placed `BottomRight` just above the toggle; `step_pad/panel`), its buttons `step_pad/up`, `step_up`, `down`, `step_left`, `step_down`, `step_right`, each a `widgets::HoldButton` with a `common::PadButton` and a `HoverTip`.
- `crates/client/src/step_pad/show.rs`: `PadOpen` (true at start, not saved), `toggle_pad`, `show_pad` (both only in `GameView::WorldMap` / `Dungeon`; the pad hidden by `Display::None` so it keeps its place; the toggle `Active` while open), `tint_icons` (white pictures in `INK`, faint when disabled), `read_pad` (`PreUpdate`, after `WidgetSystems::Presses`: held arrows into `common::PadSteps`).
- `crates/client/src/step_pad/icons.rs`: the pictures, drawn in code: `keypad`, `arrow(Pointing)`, `ladder(climbing)` (rails closing in towards the far end, rungs evenly spaced in distance so they crowd together there: narrow at the top for Up, at the bottom for Down). `picture.rs`: `picture` (white, edges smoothed by 4 × 4 samples), `on_line`, `outside_box`.

## Public API

- `StepPadPlugin`. What others read is in `common`: `PadButton` (on each pad button), `PadSteps` (resource: held arrows as screen `dx`, `dy`).

## Depends on

- `common` (`GameView`, `PadButton`, `PadSteps`), `widgets` (`spawn_grip_panel`, `HoldButton`, `spawn_icon_button`, `HoverTip`, `ButtonPressed`).

## Used by

- Through `common`: `player` (`PadSteps` in `move_player`, as held keys), `site` (`down_or_up` enables Up and Down by its rules, keeps their tips, and sends `Up` / `Down` on their `ButtonPressed`, sent once when one becomes held).

## Notes

- The arrows are screen directions, as the step keys, turned by `ViewFacing` in `player`. Only one finger holds at a time, so no diagonals by touch; the mouse and a finger could hold two.
- Up and Down fire once when they become held; staying held does nothing more.
