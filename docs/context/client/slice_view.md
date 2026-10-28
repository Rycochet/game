# client: slice_view

The original world map look, ported faithfully from the prototype's `WorldMap.tsx`: the surface around the player seen from the south as height slices, drawn on the CPU into a 400×264 image shown at 2× with sharp pixels.

## Files

- `crates/client/src/slice_view/mod.rs`: `SliceViewPlugin`; on entering `MapView::Slices` spawns a `Camera2d` and the image sprite; redraws only when the player, clock or style changes (or on entering); reports timing to `ViewStats`.
- `crates/client/src/slice_view/render.rs`: `render(...)`: pure function filling the RGBA buffer. Constants `WIDTH` 400, `DEPTH` 400 rows (every 2nd drawn), `SKY` 64, `HEIGHT`.

## Public API

- `SliceViewPlugin` only.

## Depends on

- `common` (`CurrentWorld`, `GameClock`, `MapView`, `ViewStats`), `player` (`MapPosition`), `surface_style`.

## Used by

- Nothing.

## Notes

- Per column, slices go front (south) to back. Each slice paints only the part sticking up above the previous slice's top. Perspective: far slices are narrower (`shift = (DEPTH-row)/6`) and each pixel covers more ground.
- Colours: `SurfaceStyle::shade` with slope strength 1.0 (see `surface_style.md`), so snow line and ice apply. Redraws when the player, hour, `Weather` or style changes.
- Front slice: water depth drawn at half lightness, then `earth_face` down to the base line.
- Faithful quirk: `top` is replaced by every slice even when that slice is hidden, so a far slice can paint over part of a nearer one behind steep hills. Kept to match the original; fix by tracking the minimum `top` if it shows.
- Not ported: click-to-coordinate map (`coordMap`).
- ~37 ms per redraw natively (debug), ~65 ms in the browser.
