# client: inventory

What the character carries: a moveable panel for each open grid, and picking items up, holding them and putting them down. `I` or the bag button at the bottom left opens the pockets. Native and web.

## Files

- `crates/client/src/inventory/mod.rs`: `InventoryPlugin`, `Carried`, `Holding`, `read_server`, `pick_up_by_finger` (`PreUpdate`, before widgets), `hold_items` (`PreUpdate`, after `WidgetSystems::Modal`: presses, turning, following and letting go), `InventoryDebug` (native).
- `crates/client/src/inventory/hold.rs`: without Bevy: `Held`, `Target`, `target`, `drop_move`. Unit tests.
- `crates/client/src/inventory/panels.rs`: `OpenGrids`, `CELL` (32 px), `spawn_fixed_parts`, `toggle_pockets`, `forget_closed`, `show_grids`, `show_aim`, `show_held`, `show_tooltip`, `show_button`, the bag icon (drawn in code).

## Public API

- `InventoryPlugin`.
- `Carried(Inventory)` (resource): what is carried, as the server last said plus any move since.
- `InventoryDebug` (message, native): `OpenPockets`, `OpenBags`, `PickUp(index)`. Used by `dev_tools`.

## Depends on

- `common` (`GameView`, `PointerPosition`, `PointerSystems`, `TouchGesture`), `connection` (`ToServer`, `FromServer`), `widgets`, `shared::item`, `shared::id`, `shared::protocol`.

## Used by

- `dev_tools`.

## Notes

- Owner's design: press on an item to pick it up. It follows the pointer at its true size, held by the cell it was pressed on, and letting go drops it. Anywhere it can't go, it snaps back. A held item isn't drawn where it was but nothing has changed until the drop, which is one `MoveItem`, so a lost connection can't lose it.
- The cells it would take show green, or red if refused. Dropped on a closed bag, the bag's own cells show this, and it goes in the first place it fits (`Inventory::first_place_for`), turning if it must.
- Right-click turns what is held; by touch, a tap with a second finger (owner agreed to try this, to adjust after testing). While a finger holds something, its two-finger gestures don't reach the camera, and the right button doesn't turn it.
- The bin at the bottom right of the pockets' panel destroys what is dropped on it, from any grid, with all that is in it (owner's design), at once and for good: there is no requester, and items on the ground don't exist yet. It only counts if nothing is in front of it at the pointer; it shows red and the name reads "Destroy ..." first. Panels of bags destroyed close themselves.
- Double press on a bag opens it: a second press within `DOUBLE_PRESS_SECONDS`. A press on anything else always picks up.
- By touch: a finger sliding away from an item picks it up (`TouchGesture::first_finger`, `other_finger_tapped`). A tap arrives as a click, so it picks up and drops at once, doing nothing, and two make a double press. Only what is in front at the point counts, the same as for presses.
- Moves are shown at once and `Inventory` follows every one, so a refused move goes back. Client-only choices: a panel per grid, a bag's panel closing when it is no longer carried.
- Items are plain coloured cells with the first letters of their names, until there are pictures.
