# client: common

Shared building blocks for the client modules. Keep it small.

## Files

- `crates/client/src/common/mod.rs`: `CommonPlugin`; black `ClearColor`, frame-rate cap (`WinitSettings`), world and dungeon resources, `GameView` state, clock, weather, view stats, tokens, lights.

## Public API

- `CurrentWorld` (resource): `params: WorldParams`, `surface: Surface`. Fixed seed `12345` until a server supplies one.
- `GameView` (Bevy state): `WorldMap` or `Dungeon` (the default for now, while the dungeon view is tested). Each view shows its scene and runs its systems only while current (`OnEnter`/`OnExit`, `in_state`). Switched by `demo` (M) and `dev_tools` (`GAME_PLACE`).
- `CurrentDungeon` (resource): `level: shared::dungeon::Level`, the test room for now. Read by `player` (walls block steps) and `dungeon_map`.
- `LightSource` (component): `radius` (cells) and `colour`; lights dungeons. The player has one (6 cells, white).
- `SharesSight` (component, marker): its sight is added to the player's in dungeons. Nothing has it yet.
- `GameClock` (resource): `hour` `0..24`; 1 real minute = 1 game hour; only changes on the hour (use `is_changed()`).
- `ViewStats` (resource): `summary` line written by the current view, shown by `ui` (so `ui` needn't depend on the map).
- `Token` (component): `shape: TokenShape` (`Circle`, `Triangle`, `Square`, `Hexagon`, `Octagon`) and `attitude: Attitude` (`None` white, `Friendly` green, `Neutral` blue, `Unfriendly` red; `Attitude::colour()`), and `completed: bool` (secondary state: drawn ~30% darker and slightly see-through). How an entity is shown on maps; set by its owning module (`player`), drawn by `view3d`. Both enums have `ALL` and `next()`.
- `Icon(String)` (component): a 2D picture shown above an entity's token, always facing the camera; asset path such as `monsters/humanarcher.png` (54×54 art, main picture in the central 32×32). Set by the owning module, drawn by `view3d`.
- `ViewFacing(u8)` (resource): which compass direction is nearest the top of the screen, in quarter turns clockwise from north (0 north, 1 east, 2 south, 3 west). Written by `view3d` from its camera; `screen_to_grid(dx, dy)` turns a screen direction into a grid step (used by `player` for keys).
- `Weather` (resource): `snow_line` (`NORMAL_SNOW_LINE` = 128 normally, 0 = frozen). Changed by `demo`; read by `world_map` and `ui`.

## Depends on

- `shared` (`Seed`, `WorldParams`, `Surface`, `dungeon::Level`).

## Used by

- All client modules.

## Notes

- The camera belongs to `view3d`, not here.
- `MapPosition` is the position in the current view's grid; the world map and dungeon grids are unrelated (`demo` keeps the other view's position when switching).
- Frame rate: `WinitSettings` reactive mode, about 30 fps (`FOCUSED_FPS`) while focused and 1 fps unfocused; input wakes it immediately. Not a twitch game, so this saves GPU and power.
- The clock will become server-driven.
