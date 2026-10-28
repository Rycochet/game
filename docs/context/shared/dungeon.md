# shared: dungeon

Dungeons and buildings as rules: a grid of floor and wall cells, and what can be seen (or lit) from a cell. No drawing (that is client `dungeon_map`).

## Files

- `crates/shared/src/dungeon.rs`: `Cell`, `Level`, `Sight` and the private shadowcasting scan. Golden tests (test room layout, sight from its centre).

## Public API

- `Cell` (`Floor`, `Wall`).
- `Level`: rectangle of cells (`new(min_x, min_y, width, height)` all wall, `set`, `cell`, `is_open`, `bounds`). Outside the rectangle counts as wall. `test_room()`: 15 × 7 floor cells (east–west × north–south) walled round, centred on `(0, 0)`, with a 1 × 3 wall at `x = -2`, `y = -1..=1` (temporary, for testing the view).
- `Sight`: cells seen, sized to a level. `add_view(level, x, y, radius)` adds what is seen from a cell (`radius` in cells, straight-line between centres, or `None`); calls combine (shared sight, several lights). `is_seen(x, y)`.

## Depends on

- Nothing.

## Used by

- Client `common` (`CurrentDungeon`), `player` (walls block steps), `dungeon_map` (sight, light reach).

## Notes

- Walls block movement, sight and light; floors block nothing. Light uses the same `Sight` with a radius, so it is blocked by exactly the same rule.
- Sight is symmetric shadowcasting (Albert Ford's description) with exact integer fractions: deterministic, no floats. Floors are seen when their centre is inside the visible slopes (so seeing is mutual); walls are seen when any part is, so their faces show.
- Changing the algorithm changes what players can see: update the golden test deliberately.
