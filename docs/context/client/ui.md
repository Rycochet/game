# client: ui

On-screen interface: currently a status text top-left. Uses Bevy UI (`Node`, `Text`).

## Files

- `crates/client/src/ui/mod.rs`: `UiPlugin`; the status text (position, hour and ping (a dash in single player) with symbols for labels, a ghost for `common::GodMode`, eyes for `common::Truesight` (the font has no single eye) and the mode clicks are in (`common::ClickMode`), both in red, `ViewStats` line; beside the warning button, only those while `GameView::Connecting` or with `common::DebugInfo` off). Connecting and refusals are on `loading_screen`.

## Public API

- `UiPlugin`.

## Depends on

- `common` (`ClickMode`, `DebugInfo`, `GameView`, `GodMode`, `Truesight`, `GameClock`, `ViewStats`, `MapPosition`, `ServerClock`), `player` (`Player`), `fonts` (`with_icons`).

## Used by

- Nothing.

## Notes

- Reads other modules' public resources/components; never mutates them. Views report through `common::ViewStats` so `ui` doesn't depend on any view module.
- Bevy UI draws on `view3d`'s camera (the only one).
