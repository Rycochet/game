# client: inventory

The Character panel (what the character is like, what they wear and what they carry, left to right), a moveable panel for each open bag, and picking items up, holding them and putting them down. `C`, `I` or the person button at the bottom left opens it. Native and web.

## Files

- `crates/client/src/inventory/mod.rs`: `InventoryPlugin`, `Carried`, `InventoryDebug` (native), the module list. Beside it: `holding.rs` (`Holding`, `HeldBy`, `Hand`, `cancel_held`), `server.rs` (`read_server`), `finger.rs` (`pick_up_by_finger`, `PreUpdate`, before widgets), `early_pictures.rs` (`ask_early`: pictures of everything carried and worn asked for as soon as `Carried` changes, so they are drawn before a panel shows them), `debug.rs` (`run_debug`, native).
- `crates/client/src/inventory/hold_items.rs`: `hold_items` (`PreUpdate`, after `WidgetSystems::Modal`), `is_click`, `worth_wearing`, and its `SystemParam`s `Under` (what the pointer may be over) and `Outbox` (what letting go writes). It runs three steps in `hold_items/`, in order: `press.rs` (presses: a double click opens a bag, a press picks up), `aim.rs` (turning, following the pointer or finger, and the `Target`), `let_go.rs` (taking, wearing, dropping, giving, binning or moving).
- `crates/client/src/inventory/hold.rs`: without Bevy: `Held`, `Target`, `target`, `drop_move`. Unit tests.
- `crates/client/src/inventory/body.rs`: `spawn_rows`, `BodyNode`, `show_clothes`, `show_body_aim`, `worn_notes`.
- `crates/client/src/inventory/give.rs`: `NearbyPanel`, `GiveRow`, `show_nearby`, `show_give_aim`.
- `crates/client/src/inventory/ground.rs`: `Ground`, `spawn_section`, `read_server`, `show_section`, `show_aim`.
- `crates/client/src/inventory/panels.rs`: `OpenGrids` and the panels' components, the module list. In `panels/`: `fixed.rs` (`spawn_fixed_parts`), `icons.rs` (the person, bin and ground icons, drawn in code), `grids.rs` (`OpenGrids::open`), `pockets.rs` (`toggle_pockets`, `forget_closed`), `bins.rs` (`show_bins`, `show_drop`), `show.rs` (`show_grids`, weights), `aim.rs` (`show_aim`), `held.rs` (`show_held`, `show_tooltip`, `show_button`).

## Public API

- `InventoryPlugin`.
- `OpenCharacter` (message): opens the Character panel, or brings it to the front (`ground`).
- `Carried(Inventory)` (resource): what is carried, as the server last said plus any move since.
- `InventoryDebug` (message, native): `OpenPockets`, `OpenBags`, `PickUp(index)`, `DropSome`. Used by `dev_tools`.

## Depends on

- `common` (`GameView`, `PointerPosition`, `PointerSystems`, `TouchGesture`, `GROUND`, `SpentEnergy`), `connection` (`ToServer`, `FromServer`), `player` (`Player`), `widgets`, `shared::item`, `shared::id`, `shared::protocol`.

## Used by

- `dev_tools`.

## Notes

- Kinds with models show their pictures (`item_art`) in grids, in the hand, on the clothes panel and in the bin, with the same edges. Other kinds keep their colour and label.
- An item's quality colours its edge (2 px, wider than a plain item's) in grids, worn and held, and its name, on a dark tooltip so every colour can be read (`widgets::spawn_item_cells`, `ItemTooltip`, which draw items anywhere).
- Owner's design: press on an item to pick it up. It follows the pointer at its true size, held by the cell it was pressed on, and letting go drops it; a quick click (`is_click`) instead keeps it held until the next click (`Hand::Clicked`), which goes to nothing else under it. Anywhere it can't go, it snaps back, and Esc always puts it back (`cancel_held`, before the menus see Esc). A held item isn't drawn where it was but nothing has changed until the drop, which is one `MoveItem`, so a lost connection can't lose it.
- The cells it would take show green, or red if refused. Dropped on a closed bag, the bag's own cells show this, and it goes in the first place it fits (`Inventory::first_place_for`), turning if it must.
- Right-click or R turns what is held; by touch, a tap with a second finger (owner agreed to try this, to adjust after testing). While a finger holds something, its two-finger gestures don't reach the camera, and the right button doesn't turn it.
- The bin at the bottom right of the pockets is a slot (owner's design): what is dropped on it stays there (`Inventory::bin`, whatever its shape, drawn as one cell), weighs nothing and can be picked up again, until something else is dropped there, which destroys it and all in it. Green if empty, red if it would destroy, with the name saying so. It only counts if nothing is in front of it at the pointer. A bag in it can't be opened. Panels of bags destroyed close themselves.
- What is secret isn't known here (`shared::item`): dropping something refused only for size or fingers on the clothes still asks the server (green), and it snaps back if refused. Something worn with a known curse can't be picked up, unless in `common::GodMode`; one not yet known can, and the answer puts it back, now glowing.
- The three parts share a panel, in the owner's order, with `character` filling in `common::SheetText`. Clothes (owner's design: dragged on and off): a row per place on the body and a column per layer (Claude's layout). Dropping anything on it wears it (`WearItem`), with the rows green or red first. Dragging off into a grid takes it off (`MoveItem`); dropped elsewhere, it snaps back. Something covered is paler and can't be picked up, and its name says what it is under, or that it is loose. Rings on several fingers share a slot, with a count. A worn bag opens with a double press as anywhere.
- Giving: a Nearby panel opens with the Character panel, to become a trading panel (owner), listing who is within `mob::GIVE_REACH`, by `common::Other`. Dropping a held item on a row sends `GiveItem`; the server also checks they are in sight, and `Inventory` puts it right if not. Clicking people on the map to give is for later, as dropping on tokens needed more than this.
- Anything with `common::TakesGifts` counts as a row: `inventory` says what is held over it and let go on it (`HeldOverGifts`, `GaveItem`), for `trade` to show.
- Something picked up from a trading panel is held in `Holding::stock`, still theirs until dropped (`hold::target_for_new`, `TakeItem`). It can't go on the bin, the clothes or someone's row from there.
- Double press on a bag opens it: a second press within `DOUBLE_PRESS_SECONDS`, which by mouse puts back what the first picked up. A press on anything else always picks up.
- By touch (owner's design): a finger staying still on an item picks it up (`TouchGesture::still_at`), and it follows `first_finger`; a finger sliding at once scrolls instead. A tap (`TouchGesture::tap`) never picks up: the pointer stays on it, so its name shows, and two on a bag open it. Only what is in front at the point counts, the same as for presses.
- Moves are shown at once and `Inventory` follows every one, so a refused move goes back. Client-only choices: a panel per grid, a bag's panel closing when it is no longer carried.
- Items are plain coloured cells with the first letters of their names, until there are pictures.
- The ground (owner's design; sites only, never the world map): what is dropped on the Drop icon, left of the bin, goes on the cell under the player (`DropItem`), red on the world map, for what is covered and for anything held from the ground or a shop. While the player stands on something, a section to the right of the pockets shows it in a scrolling grid 8 wide (`widgets::StockGrid`, with `common::GROUND` standing for who it is with, so `hold_items` treats it as stock and gifts). What is dropped on the grid goes on the ground. From the ground, things can go to grids, the clothes (`WearFromGround`) or the bin (`BinFromGround`), but not the Drop icon, someone's row or a shop. Worn things can go straight to the ground if nothing covers them. A bag on the ground can't be opened until picked up.
- Moving things between containers takes time, by weight (`Inventory::move_cost`, `put_down_cost`, `energy::handling_cost`); sorting one grid doesn't. `SpentEnergy` tells `player`, so it doesn't run ahead of the server.
