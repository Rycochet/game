# client: item_text

Finding out what items written into text are (`${item:<uuid>}`, `shared::item_text`), for whatever shows them. Not how they are drawn: that is `widgets` (inline pictures, `ItemText`).

## Files

- `crates/client/src/item_text/mod.rs`: `ItemTextPlugin`; `look_up_items` (`PostUpdate`, after everything showing items has said which it wants: what the player carries is filled in at once from `Carried` (`Inventory::find_within`, the item alone), the rest asked of the server with `LookUpItems` in batches of `MOST_LOOKED_UP`), `read_answers` (`ItemsLookedUp` into `KnownItems`).

## Public API

- `ItemTextPlugin`. What it fills is `common::KnownItems` (see `common.md`).

## Depends on

- `common` (`KnownItems`, `KnownItem`), `connection` (`ToServer`, `FromServer`), `inventory` (`Carried`), `shared::protocol`.

## Used by

- Nothing directly: `widgets` reads `KnownItems`.

## Notes

- Items follow the real item, not a copy kept with the text (owner): an answer is kept `ITEM_KEPT_FOR` (60 s) while on show, then asked again; an unanswered ask is repeated after 10 s. The server only looks in what it has loaded, so an item in nothing loaded reads as unknown.
