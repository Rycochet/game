# shared: item_text

Items written into text as `${item:<uuid>}`: writing one, and reading text back as plain words and items. Not how an item is shown (client) or found (server).

## Files

- `crates/shared/src/item_text.rs`: `ITEM_START`, `ITEM_END`, `Piece`, `item_token`, `item_at`, `pieces`, `items_in`.

## Public API

- `item_token(id) -> String`: `${item:<uuid>}`, the UUID hyphenated in lower case.
- `item_at(text) -> Option<(Id, bytes)>`: the item written at the very start of `text`.
- `pieces(text) -> Vec<Piece>`: `Piece::Text(&str)` and `Piece::Item(Id)` in order; neighbouring words are one piece, none empty.
- `items_in(text) -> Vec<Id>`: each item once, in the order first written (for asking the server, `ClientMessage::LookUpItems`).
- `ITEM_START` (`${item:`), `ITEM_END` (`}`).

## Depends on

- `shared::id` (`Id`), `uuid`.

## Used by

- Nothing yet: the client's text boxes and text display will (Text box editing, PR 3).

## Notes

- Only the exact shape counts: a hyphenated UUID (either case) between `${item:` and `}`. Anything else, a misspelt one included, is plain text, and a broken one doesn't hide a good one after it.
- The text says only which item: what it is called and looks like comes from the server (`ItemsLookedUp`), live, as the reader may know it. `@name` is kept for mentioning players later.
