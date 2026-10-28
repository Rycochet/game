# client: common

Shared building blocks for the client modules. Keep it small.

## Files

- `crates/client/src/common/mod.rs`: `CommonPlugin`; black `ClearColor`, frame-rate cap (`WinitSettings`), world resource, clock, weather and view stats.

## Public API

- `CurrentWorld` (resource): `params: WorldParams`, `surface: Surface`. Fixed seed `12345` until a server supplies one.
- `GameClock` (resource): `hour` `0..24`; 1 real minute = 1 game hour; only changes on the hour (use `is_changed()`).
- `ViewStats` (resource): `summary` line written by `world_map`, shown by `ui` (so `ui` needn't depend on the map).
- `Token` (component): `shape: TokenShape` (`Circle`, `Triangle`, `Square`, `Hexagon`, `Octagon`) and `attitude: Attitude` (`None` white, `Friendly` green, `Neutral` blue, `Unfriendly` red; `Attitude::colour()`), and `completed: bool` (secondary state: drawn ~30% darker and slightly see-through). How an entity is shown on maps; set by its owning module (`player`), drawn by `world_map`. Both enums have `ALL` and `next()`.
- `Icon(String)` (component): a 2D picture shown above an entity's token, always facing the camera; asset path such as `monsters/humanarcher.png` (54×54 art, main picture in the central 32×32). Set by the owning module, drawn by `world_map`.
- `Weather` (resource): `snow_line` (`NORMAL_SNOW_LINE` = 128 normally, 0 = frozen). Changed by `demo`; read by `world_map` and `ui`.

## Depends on

- `shared` (`Seed`, `WorldParams`, `Surface`).

## Used by

- All client modules.

## Notes

- The camera belongs to `world_map`, not here.
- Frame rate: `WinitSettings` reactive mode, about 30 fps (`FOCUSED_FPS`) while focused and 1 fps unfocused; input wakes it immediately. Not a twitch game, so this saves GPU and power.
- The clock will become server-driven.
