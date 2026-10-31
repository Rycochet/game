# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; spawns and updates the status text (controls including clicking places and stairs, right-drag, wheel and Home for the camera, ctrl+click and Esc (dev tools), M for the map, demo keys with their state, snow line, position, hour, `ViewStats` line), empty while `GameView::Connecting`; and a red line bottom left from `common::ServerStatus`: "Connecting..." while not connected, or why the server refused.

## Public API

- `UiPlugin`.
- `raised_box()`: a bundle for a box standing up from what is behind it (lit top and left, in shade bottom and right); put its contents in as children. Used by `minimap`.

## Depends on

- `common` (`GameView`, `GameClock`, `ViewStats`, `Weather`, `MapPosition`), `demo` (`AutoWalk`, `TemperatureCycle`), `player` (`Player`).

## Used by

- `minimap` (`raised_box`).

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on `view3d`'s camera (the only one).
