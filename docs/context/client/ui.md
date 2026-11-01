# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; the status text (position, hour and ping (a dash in single player) with symbols for labels, the snow line while `debug_menu::TemperatureCycle` is running, the mode clicks are in (`common::ClickMode`, in red), `ViewStats` line; beside the warning button, only the mode while `GameView::Connecting` or with `common::DebugInfo` off). Connecting and refusals are on `loading_screen`.

## Public API

- `UiPlugin`.

## Depends on

- `common` (`ClickMode`, `DebugInfo`, `GameView`, `GameClock`, `ViewStats`, `Weather`, `MapPosition`, `ServerClock`), `player` (`Player`), `debug_menu` (`TemperatureCycle`), `fonts` (`with_icons`).

## Used by

- Nothing.

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on `view3d`'s camera (the only one).
