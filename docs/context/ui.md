# ui

On-screen interface: HUD text, and later menus and panels. Uses Bevy UI (`Node`, `Text`).

## Files

- `src/ui/mod.rs`: `UiPlugin`; `spawn_controls_hint` (Startup) shows the controls text top-left.

## Public API

- `UiPlugin`: registered in `main.rs`.

## Depends on

- Nothing beyond Bevy.

## Used by

- Nothing.

## Notes

- UI should read game state through other modules' public components/resources and never mutate their internals.
