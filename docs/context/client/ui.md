# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; spawns and updates the status text (controls and demo keys with their state, snow line, position, hour, `ViewStats` line).

## Public API

- `UiPlugin` only.

## Depends on

- `common` (`GameClock`, `ViewStats`, `Weather`), `demo` (`AutoWalk`, `TemperatureCycle`), `player` (`MapPosition`, `Player`).

## Used by

- Nothing.

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on the world map's camera (the only one).
