# client: hover

The box near the bottom middle of the window naming what the pointer is over: another creature (a mob by its kind, another player by their character's name), a world map poi or World Gate, a building's door (a shop's by what it sells), stairs, a bag on the ground, or a sun or moon by the minimap. It only shows names: each module that knows what is under the pointer says so through `common`.

## Files

- `crates/client/src/hover/mod.rs`: `HoverPlugin`; `spawn::spawn_box` (Startup), `name::show_name` (PostUpdate, not while connecting).
- `crates/client/src/hover/spawn.rs`: `HOVER` (the box as markup: `hover/row`, an `inert` row across the window placed from its bottom, holding `hover/box`, a `span class="tip"` with `hover/mark` (the staff mark, a gold laurel crown: `img class="gold"`) and `hover/words`; the form `fills` the window, or the row would be placed from the top), `spawn_box` (the document, on a node of its own over the whole window, above the maps), `LIFT` (12 px above `common::BottomHud`), `LOWEST` (32 px).
- `crates/client/src/hover/name.rs`: `Found` (what each module found), `name_of` (which wins), `mark_of` (the hovered player's `StaffMark`, when their name is what shows), `show_name` (through `Dom`, changing only what differs from what the box last showed: places, fills and shows or hides it; a marked player's name reads "Brin (Admin)" with the mark before it).

## Public API

- `HoverPlugin`.

## Depends on

- `common` (`HoverName`, `StaffMark`, `HoveredOnMap`, `HoveredInSky`, `HoveredBag`, `Portal`, `poi_name`, `CurrentWorld`, `KnownPois`, `BottomHud`, `GameView`), `view3d` (`HoveredToken`), `minimap` (`FullMap`), `widgets` (`spawn_tooltip_with_picture`).

## Used by

- Nothing: it only shows.

## Notes

- Who finds what: `view3d` the hovered token (clickable, or with a `common::HoverName`: `entities` gives every other creature one); `ground` the bag (`HoveredBag`); `dungeon_map` the door or stairs (`HoveredOnMap`); `minimap` the sun or moon (`HoveredInSky`). World map pois and gates are named from their `Portal`; a town, castle or fort whose name the player has learned by its name ("Ruined" before a ruin's) (`CurrentWorld`'s `KnownPois::name_of`, passed in `Found::pois`), never before (owner).
- Order when several are under the pointer: the sky (drawn over everything), then a token, then a bag, then the map's door or stairs.
- Hidden while the full map is open: it has its own box (`minimap`).
- Lifted above whatever the interface keeps at the bottom (`BottomHud`: the vitals circle). By touch, a name shows while a finger is down, as the pointer is there only then.
