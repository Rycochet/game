# client: hover

The box near the bottom middle of the window naming what the pointer is over: another creature (a mob by its kind, another player by their character's name), a world map poi or World Gate, a building's door (a shop's by what it sells), stairs, a bag on the ground, or a sun or moon by the minimap. It only shows names: each module that knows what is under the pointer says so through `common`.

## Files

- `crates/client/src/hover/mod.rs`: `HoverPlugin`; `spawn::spawn_box` (Startup), `name::show_name` (PostUpdate, not while connecting).
- `crates/client/src/hover/spawn.rs`: `HoverBox`, `HoverText`, `spawn_box` (a `widgets` tooltip in an absolute row across the window), `LIFT` (12 px above `common::BottomHud`), `LOWEST` (32 px).
- `crates/client/src/hover/name.rs`: `Found` (what each module found), `name_of` (which wins), `show_name` (places, fills and shows or hides the box).

## Public API

- `HoverPlugin`.

## Depends on

- `common` (`HoverName`, `HoveredOnMap`, `HoveredInSky`, `HoveredBag`, `Portal`, `poi_name`, `BottomHud`, `GameView`), `view3d` (`HoveredToken`), `minimap` (`FullMap`), `widgets` (`spawn_tooltip`).

## Used by

- Nothing: it only shows.

## Notes

- Who finds what: `view3d` the hovered token (clickable, or with a `common::HoverName`: `entities` gives every other creature one); `ground` the bag (`HoveredBag`); `dungeon_map` the door or stairs (`HoveredOnMap`); `minimap` the sun or moon (`HoveredInSky`). World map pois and gates are named from their `Portal`.
- Order when several are under the pointer: the sky (drawn over everything), then a token, then a bag, then the map's door or stairs.
- Hidden while the full map is open: it has its own box (`minimap`).
- Lifted above whatever the interface keeps at the bottom (`BottomHud`: the vitals circle). By touch, a name shows while a finger is down, as the pointer is there only then.
