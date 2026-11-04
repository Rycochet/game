# client: hall_view

The Hall of Worlds' own looks on top of what `dungeon_map` draws (its marble floor, slab edges and light are `dungeon_map`'s, by `LevelKind::Hall`): the World Gates' models on their ledges, with their codes in runes (at any zoom) and water running down their openings; the camera reaching 25% further and tilting no lower than 18°, so the floor always runs off the top of the screen and the Hall seems endless. `GateModel` builds the gates, for World Gate pois on the world map too.

## Files

- `crates/client/src/hall_view/mod.rs`: `HallViewPlugin` (in `GameView::Dungeon`: `place_gates`, `run_water`; `remove_gates` on leaving it; after `View3dSystems::Place`: `hall_reach`), `in_hall`, `pub use GateModel`.
- `hall_view/place.rs`: `Gates` (resource: the gate entities standing, by cell as the client counts it, their shared unlit material and the water's), `place_gates` (when `CurrentDungeon` changes: one per `shared::hall::GatePlace` in the stretch, with the cells' `x + origin`, runes and water as children; anywhere else, none), `run_water` (slides the water's picture down, `FLOW` pictures a second, by its material's `uv_transform`), `remove_gates`.
- `hall_view/sight.rs`: `hall_reach` (in the Hall `OrbitCamera::set_reach(REACH)` 1.25 and `set_lowest_pitch(LOWEST_PITCH)` 18°: at the furthest zoom the top of the screen then meets the floor about 72 tiles past the player, within the 75 `site` holds each way; at the usual 10° the top of the screen is level and no number of tiles would do. Usual values elsewhere). Owner: this rather than a fade (tried as an overlay, fog, darkening and transparency).
- `hall_view/gate.rs`: `GateModel` (triangles with a normal and colour each: `frame(design)`, `runes(design, code)`, `size(design)`, `triangles()`, `mesh(scale)`, light baked into the colours for unlit materials; `water(design, scale)` and `water_picture()`: the opening's sheet with picture coordinates, half see-through blue with paler streaks, tiling every half cell, drawn from both sides). In `gate/`: `profile.rs` (each `GateDesign`'s opening: jambs, then its top as points round from the left), `frame.rs` (the slab round it, filled along rays from the jambs' tops; `band`, `outline`), `water.rs` (the sheet: a fan over the opening from the jambs' tops), `runes.rs` (the code inlaid on both faces from `shared::hall::RUNE_STROKES`, mirrored on the back, each stroke on a black outline, owner).

## Public API

- `HallViewPlugin`.
- `GateModel`: units are cells, `x` along the frame, `y` up, `z` through it, standing on the origin; the frame 1.14 cells wide (`HALF_OPENING` 0.45 and `POST` 0.12, owner), up to about 1.6 cells tall by design. For the world map's World Gate pois when they come (`triangles()` suits `poi_model`'s pictures, `mesh(scale)` its meshes).

## Depends on

- `common` (`CurrentDungeon`, `GameView`, `SmoothPosition`), `dungeon_map` (`CELL_SIZE`), `view3d` (`OrbitCamera`, `ViewCamera`, `View3dSystems`), `player` (`Player`), `shared::hall`, `shared::protocol` (`LevelKind`).

## Used by

- Nothing else yet.

## Notes

- Runes are never a font (owner): every rune anywhere comes from `RUNE_STROKES`, so they always look the same. An interface showing codes should draw the same strokes.
- The frame's faces are filled between the opening and the outline along rays from `(0, SPRING)`, so every design's top must go round monotonically as seen from there; points are added where rays meet the outline's corners (exactly, by intersection). Tests check every design stays in its outline, faces point the way they say, the front's area is the outline less the opening, and the runes fit the band.
