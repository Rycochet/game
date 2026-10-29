# client: common

Shared building blocks for the client modules. Keep it small.

## Files

- `crates/client/src/common/mod.rs`: `CommonPlugin`; black `ClearColor`, frame-rate cap (`WinitSettings`), world and dungeon resources, `GameView` state, clock, weather, view stats, tokens, lights.

## Public API

- `CurrentWorld` (resource): `params: WorldParams`, `surface: Surface`, `features: KnownFeatures`; `CurrentWorld::new(&WorldInfo)`. Only exists once the server has welcomed the client (inserted by `connection`, which also adds the chunks the server sends, marking it changed).
- `KnownFeatures`: the world map chunks the server has sent and their features (`protocol::SeenFeature`, no map seed; `None`: known to have none). `insert(chunk, feature)`, `in_area(min, max)` (as `FeatureMap::in_area`: any planet-width frame). Unit test.
- `MapPosition { x: i64, y: i64 }` (component): position on the current view's grid; `y` increases southwards (towards the viewer). For the travelling player, the nearest grid point to `SmoothPosition`.
- `SmoothPosition { x: f64, y: f64 }` (component): exact position, fractional while the player travels, equal to `MapPosition` when still (`From<MapPosition>`). Anything with a `Token` needs one: `view3d` draws tokens there.
- `GameView` (Bevy state): `Connecting` (the default: waiting for the server, no `CurrentWorld` yet; `connection` moves on to `WorldMap`), `WorldMap` or `Dungeon` (inside a site). Each view shows its scene and runs its systems only while current (`OnEnter`/`OnExit`, `in_state`). Switched by `site`.
- `CurrentDungeon` (resource): the site level as far as the server has told: `info: Option<LevelInfo>` (kind, floor/level, bounds...), `level` (`shared::dungeon::Level`: unknown cells are wall), `known` (cells sent), `remembered` (cells seen now or before, drawn in grey when out of sight; includes what is in sight), `stairs: Vec<SeenStairs>` (known stairs, with the way they go down). `enter(info)`, `add(&LevelChunk)`, `describe()` for the HUD ("Castle, floor 2 of 3"). Changed by `site`; read by `player` (walkable cells) and `dungeon_map` (which adds to `remembered` without marking it changed).
- `Portal` (component): `Feature(SeenFeature)`: clicking it sends the player there and arriving enters its site (`site`). World map features have one; stairs are cells of the level, not entities.
- `Clickable` (component, marker): its token can be clicked (`view3d::ClickedToken`).
- `LightSource` (component): `radius` (cells) and `colour`; lights dungeons. The player has one (6 cells, white).
- `SharesSight` (component, marker): its sight is added to the player's in dungeons. Nothing has it yet.
- `GameClock` (resource): `hour` `0..24`; 1 real minute = 1 game hour; only changes on the hour (use `is_changed()`).
- `ViewStats` (resource): `summary` line written by the current view, shown by `ui` (so `ui` needn't depend on the map).
- `Token` (component): `shape: TokenShape` (`Circle`, `Triangle`, `Square`, `Hexagon`, `Octagon`) and `attitude: Attitude` (`None` white, `Friendly` green, `Neutral` blue, `Unfriendly` red; `Attitude::colour()`), and `completed: bool` (secondary state: drawn ~30% darker and slightly see-through), and `size: TokenSize` (how big its `Icon` is drawn; the base never changes: `Tiny` 0.5, `Small` 0.75, `Default` 1, `Large` 1.25, `Giant` 1.5, `Titan` 2; `scale()`). World map features are `Tiny`, the player `Default`. How an entity is shown on maps; set by its owning module (`player`), drawn by `view3d`. The enums have `ALL` and `next()`.
- `Icon(String)` (component): a 2D picture shown above an entity's token, always facing the camera; asset path such as `monsters/humanarcher.png` or `terrain/l2_terrain086.png` (54×54 art, main picture in the central 32×32). Set by the owning module, drawn by `view3d`.
- `ViewFacing(u8)` (resource): which compass direction is nearest the top of the screen, in quarter turns clockwise from north (0 north, 1 east, 2 south, 3 west). Written by `view3d` from its camera; `screen_to_grid(dx, dy)` turns a screen direction into a grid step (used by `player` for keys).
- `Weather` (resource): `snow_line` (`NORMAL_SNOW_LINE` = 128 normally, 0 = frozen). Changed by `demo`; read by `world_map` and `ui`.

## Depends on

- `shared` (`protocol::{WorldInfo, SeenFeature, LevelInfo, LevelChunk, SeenStairs}`, `WorldParams`, `Surface`, `feature::{ChunkPos, CHUNK_SIZE}`, `dungeon::{Level, Sight}`).

## Used by

- All client modules.

## Notes

- The camera belongs to `view3d`, not here.
- `MapPosition` is the position in the current view's grid; the world map and dungeon grids are unrelated (`site` keeps the world map position while inside a site).
- Frame rate: `WinitSettings` reactive mode, about 30 fps (`FOCUSED_FPS`) while focused and 1 fps unfocused; input wakes it immediately. Not a twitch game, so this saves GPU and power.
- The clock will become server-driven.
