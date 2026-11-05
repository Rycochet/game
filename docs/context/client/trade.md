# client: trade

Trading panels. For now a town shop's: clicking its door opens it from beside the door, or walks to the cell in front of it and opens it on arriving. Each panel shows what its keeper has in a grid. Dropping something on the grid gives it to the keeper; there is no buying or payment yet.

## Files

- `crates/client/src/trade/mod.rs`: `TradePlugin`; private `OpenTrades`, `Keepers`, `Heading`, `follow_clicked_door`, `open_on_arrival`, `note_gifts`, `read_server`, `draw_grids`, `show_gift_aim`, `close_on_escape`, `close_when_left`. Unit tests. Grids are `widgets::StockGrid`. Beside it: `state.rs`, `open.rs`, `stock.rs`, `close.rs`.

## Public API

- `TradePlugin`.

## Depends on

- `common` (`CurrentDungeon`, `GameView`, `MapPosition`, `PointerSystems`, `TakesGifts`, `HeldOverGifts`, `GaveItem`), `inventory` (`Carried`), `connection` (`ToServer`, `FromServer`), `dungeon_map` (`DoorClicked`), `player` (`Player`, `Arrived`, `TakeStep`, `TravelTo`, `PlayerSystems`), `widgets`, `shared::shop`, `shared::item`, `shared::protocol`.

## Used by

- Nothing.

## Notes

- A panel belongs to who it is with, an `Id`: a shop's keeper for now, from `ServerMessage::Keepers`. There is never more than one for the same one (a second click brings it to the front), but two shops side by side can both be open.
- A panel only opens on `TradeStock`, so one the server refused never shows. The server checks `DOOR_REACH` itself.
- A door of a building that sells nothing walks to it, as it always did.
- A step of the player's own, or a journey elsewhere, drops the wish to open it, and stopping short never opens it (`Arrived`).
- Grid (owner's design): 12 wide, as many rows as needed, oldest first, each item in the first place it fits along each row from the left. Below it all are as many empty rows as the longest side of any kind at its largest size. Worked out at each opening; places are not kept.
- Selling (owner: giving, for now): all of a grid has `TakesGifts`, so `inventory` treats dropping there as giving (`GiveItem`). Green shows where it will land: where it is let go if that is free, or else the first place. It stays there until the panel next opens, and the grid grows to keep its empty rows. `TradeStock` for an open panel keeps everything where it is.
- Each grid is in a `widgets::spawn_container` frame that grows, with no weight (stock is no one's to carry); taller than `MOST_HEIGHT` of the window, it scrolls, keeping its place by who the stock is with (`widgets::ScrollKey`).
- Taking (owner, free for now): stock is picked up as carried things are (`common::StockCell`, held by `inventory`, which says so in `HeldStock`). Dropped on its own grid it moves, on the client only. Dropped in the inventory or a bag it goes by the same rules (`TakeItem`), shown at once with `TookItem`; `TradeStock` puts it right if refused. Anywhere else it snaps back.
- Esc closes the panel opened last, ahead of the system menu. The close button, walking out of reach, a new level and leaving the map also close panels.
- A tap arrives as a click, so all this works by touch.
- The stock doesn't change while it is open: it is only caught up on opening.
