# client: shop

A town shop's panel. Clicking a shop's door opens it from beside the door, or walks to the cell in front of it and opens it on arriving. It lists what is for sale, best first, in quality colours. There is no buying yet.

## Files

- `crates/client/src/shop/mod.rs`: `ShopPlugin`; private `OpenShop`, `Heading`, `follow_clicked_door`, `open_on_arrival`, `show_stock`, `close_on_escape`, `close_when_left`. Unit tests.

## Public API

- `ShopPlugin`.

## Depends on

- `common` (`CurrentDungeon`, `GameView`, `MapPosition`, `PointerSystems`, `quality_colour`), `connection` (`ToServer`, `FromServer`), `dungeon_map` (`DoorClicked`), `player` (`Player`, `Arrived`, `TakeStep`, `TravelTo`, `PlayerSystems`), `widgets`, `shared::shop`, `shared::item`, `shared::protocol`.

## Used by

- Nothing.

## Notes

- The panel only opens on `ShopStock`, so one the server refused never shows. The server checks `DOOR_REACH` itself.
- A door of a building that sells nothing walks to it, as it always did.
- A step of the player's own, or a journey elsewhere, drops the wish to open it, and stopping short never opens it (`Arrived`).
- Esc closes the panel, ahead of the system menu; so does the close button, walking out of reach and leaving the map.
- A tap arrives as a click, so all this works by touch.
- The stock doesn't change while it is open: it is only caught up on opening.
