# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; spawns and updates the status text (controls including M to switch views, right-drag, wheel and Home for the camera, demo keys with their state, snow line, position, hour, `ViewStats` line).

## Public API

- `UiPlugin` only.

## Depends on

- `common` (`GameClock`, `ViewStats`, `Weather`, `MapPosition`), `demo` (`AutoWalk`, `TemperatureCycle`), `player` (`Player`).

## Used by

- Nothing.

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on `view3d`'s camera (the only one).
