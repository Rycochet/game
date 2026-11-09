# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; the status text, for everyone (owner, 2026-11-09): position, frame rate (🎬: the icon font has no stopwatch, owner's choice) and ping (a dash in single player) with symbols for labels; with `common::DebugInfo` (Show Debug Info) also the hour and the `ViewStats` line; a ghost for `common::GodMode`, eyes for `common::Truesight` (the font has no single eye) and the mode clicks are in (`common::ClickMode`), both in red, always; beside the warning button, only those while `GameView::Connecting`. Connecting and refusals are on `loading_screen`.
- `crates/client/src/ui/frame_rate.rs`: `FrameTimes` (when each of the last 2 seconds' frames began, real time; `rate`, frames a second over them), `note_frame` (before the text).

## Public API

- `UiPlugin`.

## Depends on

- `common` (`ClickMode`, `DebugInfo`, `GameView`, `GodMode`, `Truesight`, `GameClock`, `ViewStats`, `MapPosition`, `ServerClock`), `player` (`Player`), `fonts` (`with_icons`).

## Used by

- Nothing.

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on `view3d`'s camera (the only one).
