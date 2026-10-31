# client: minimap

The maps: the round minimap always in the window's top-right corner, and the full parchment map that M or a click on the minimap opens. Both show only what is known (lighting doesn't matter), painted on the CPU into a picture shown by Bevy UI, redrawn only when something it shows changes. On the world map they show the land's plain colours from above and features' pictures; in sites, remembered tiles as a grid, coloured by their ground.

## Files

- `crates/client/src/minimap/mod.rs`: `MinimapPlugin`, `FullMap`. `spawn_maps` (Startup). `take_input` (`PreUpdate`, after Bevy's input systems: M, Esc, clicks on the minimap, close button and outside the parchment, dragging, wheel; takes what it uses out of the input resources, so the game's systems never see it). `draw_maps` (`Update`, after `View3dSystems::Place`: sizes the UI, decides whether to redraw, paints). `name_what_is_hovered` and `hovered_name` (the shop door or feature under the mouse, in the raised box). `location_title`, `words`. Sizes are constants at the top, in logical pixels (multiplied by the window's scale factor when painting), since the owner expects to tweak them: `MINIMAP_OF_HEIGHT`, `BORDER`, `NOTCH`, `CELL_PIXELS`, `SIGN_PIXELS`, `ICON_PIXELS`, `STAIRS_PIXELS`, `COMPLETED_OPACITY`, `DOT_PIXELS`, `DOT_OUTLINE`, `ARROW_PIXELS`, `FULL_MAP_OF_WINDOW`, `FULL_MAP_MARGIN`, `TITLE_HEIGHT`, `CLOSE_PIXELS`, `HOVER_LIFT`, `OPENING_ZOOM`, `MIN_ZOOM`, `MAX_ZOOM`, `ZOOM_PER_NOTCH`, `COMPASS_LINE`, `COMPASS_OF_HEIGHT`, `COMPASS_MIN_LENGTH`, `COMPASS_INSET`, `KEEP_KNOWN_INSIDE`. Unit tests drive `take_input`.
- `crates/client/src/minimap/canvas.rs`: `Canvas` (an image's bytes: `fill`, `set`, `blend`, `shape` by a distance function, `disc`, `sprite`), `Sprite`, `polygon_distance`.
- `crates/client/src/minimap/projection.rs`: `Projection` (`to_grid`, `to_picture`, `grid_box`: turned and scaled), `slide` (how far a drag can go while staying allowed).
- `crates/client/src/minimap/grid.rs`: site levels. `Looks::of` works out each cell in view as a `CellLook` (`Empty`, `Floor` with its colour from `ground_colour`, `Wall` with the `Sides` it is hatched along, `Door`, `HouseDoor`, `Sign`); `colour_at` is the drawing of a cell (grid line, hatching, door); `draw` samples it per pixel (four samples when turned).
- `crates/client/src/minimap/world.rs`: world map. `WorldTiles` keeps a chunk's colours (one per point, averaged by `draw` where a pixel covers more, from `SurfaceStyle::ground_colour`), worked out for chunks in view, nearest first, within a time budget per redraw; `shows_known`.
- `crates/client/src/minimap/frame.rs`: `RoundFrame` (cuts the picture round, ridged border and notch), `compass` (the same ridge, through `ridged`), `stairs_icon`, `player_dot`, `arrow_place`, `arrow`.
- `crates/client/src/minimap/sprites.rs`: `Sprites`: small copies of pictures, averaged down once per size, read from loaded `Image`s.

## Public API

- `MinimapPlugin`.
- `FullMap` (resource): `open`, `centre` (`None` follows the player), `zoom`. Used by `dev_tools`.

## Depends on

- `common` (`CurrentWorld`, `KnownFeatures::is_known`, `CurrentDungeon`, `GameClock`, `GameView`, `Portal`, `SmoothPosition`, `Token`, `Weather`), `surface_style` (`SurfaceStyle`), `view3d` (`OrbitCamera`, `View3dSystems`), `player` (`Player`, `PlayerSystems`), `world_map` (`HALF_SIZE`, `feature_icon`), `dungeon_map` (`TOWN_ATLAS`, `door_picture`), `ui` (`raised_box`), `shared::dungeon`, `shared::feature`, `shared::site` (`BuildingKind`).

## Used by

- `dev_tools` (`GAME_MAP`).

## Notes

- Minimap (owner's design): about a fifth of the window's height across, top right, over everything; contents and notch turn with the camera so the notch points north. World map scale: its diameter shows the same distance as the 3D window's width, so zooming changes it. Sites: fixed `CELL_PIXELS`.
- Full map (owner's design, plus Claude's choices marked): a square of 90% of the window's shorter side, a 20 px bare margin, a centred title (location, floor or level), an X button, and a compass. Closes by M, Esc, the button or a click outside the parchment. It opens at twice the minimap's scale; the wheel zooms it from 1x to 4x towards the mouse, with pictures and the player's dot scaling too (the grid lines and the arrow stay their size). Dragging moves it; it can't leave what is known (at least `KEEP_KNOWN_INSIDE` px inside its edges must show something known). An arrow on its edge points to the player when off the map. North is always at the top, like a sheet of paper (Claude's choice). The minimap is hidden while it is open (Claude's choice). The game keeps running and the keyboard still walks; the mouse belongs to the map, so right-drag and the wheel do nothing to the camera (Claude's choice). Esc closes it rather than going back to the start. Hovering over a shop door or feature names it in a box near the bottom of the parchment.
- Clicks on the minimap's circle open the full map, and don't reach the land under it.
- Known: on the world map, the chunks the server has sent (`KnownFeatures`), saved with the character; in sites, `CurrentDungeon::remembered`. A wall is hatched towards a cell only if that is remembered too.
- Pictures stay upright when the map turns (the main view's icons face the camera too), so they are stamped on after the contents are turned: this is why the whole map is repainted rather than a north-up picture being turned by a shader.
- Grid squares are `CELL_PIXELS` (11) apart: 10 px of white and the 1 px line shared with the next.
- Tile colours (owner's): white floors, pale colours by road type, pale green outside and in gardens (Claude's choice), blue water and red lava. The world map doesn't show rivers.
- Stairs (owner's): a flat icon of four steps on whatever tile they are on, highest on the left going down and on the right going up, so it stays upright when the map turns. Smaller than the player's dot, leaving more of a gap; remembered stairs only.
- Hatching (owner's): inside remembered wall cells, reaching a quarter of a cell in from each side that meets something open, including a dungeon door (so its sides don't look open), or a wall of a different building (so touching buildings are hatched on both sides). Town doors, house or shop, have none, and nothing hatches towards them.
- A shop's sign is its door picture from the town atlas (`door_picture`); a house has its own look, a red diamond on the line of its wall (the diamond is Claude's choice), so it differs from a dungeon's door.
- Features' completed look reads `Token::completed` on their entities, which exist only near the player, and nothing marks them completed yet. It should move to the server's per-player feature state once something does.
- `dungeon_map` adds to `remembered` without marking it changed, so maps redraw at least every `REDRAW_SECONDS`.
