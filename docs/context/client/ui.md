# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; spawns and updates the status text (controls including clicking places and stairs, right-drag, wheel and Home for the camera, demo keys with their state, snow line, position, hour, `ViewStats` line), or "Connecting to the server..." while `GameView::Connecting`.

## Public API

- `UiPlugin` only.

## Depends on

- `common` (`GameView`, `GameClock`, `ViewStats`, `Weather`, `MapPosition`), `demo` (`AutoWalk`, `TemperatureCycle`), `player` (`Player`).

## Used by

- Nothing.

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on `view3d`'s camera (the only one).
