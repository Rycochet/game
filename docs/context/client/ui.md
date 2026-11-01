# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; the status text (position, hour, ping, snow line, `ViewStats` line; below the warning button, empty while `GameView::Connecting` or with `common::DebugInfo` off), and the mode text beside the warning button (`common::ClickMode`, in red). Connecting and refusals are on `loading_screen`.

## Public API

- `UiPlugin`.

## Depends on

- `common` (`ClickMode`, `DebugInfo`, `GameView`, `GameClock`, `ViewStats`, `Weather`, `MapPosition`), `player` (`Player`).

## Used by

- Nothing.

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on `view3d`'s camera (the only one).
