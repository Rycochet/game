# client: notices

Short notices that go by themselves (a snackbar), at the top middle of the window under the status text. For now "Found: <kind>" when the server says the player has found a poi in the wilderness, and "Locked" when a door the player tried is locked (`DoorRefused` with `locked`). Not for anything that needs an answer (that is a dialog).

## Files

- `crates/client/src/notices/mod.rs`: `NoticesPlugin`; `NOTICES` (the notice as markup: `notices/row`, a row across the window 64 down, `inert` and hidden, holding a parchment `panel` with `notices/words`), `spawn_notice` (Startup: the document, on a node of its own over the whole window), `show_notices` (Update, through `Dom`: `ServerMessage::PoiFound` or a locked `DoorRefused` sets the words and shows the row for `SHOWN_FOR` (4 s), the newest in place of any showing), `found` (what the notice says, the kind in words).

## Public API

- `NoticesPlugin`.

## Depends on

- `common` (`words`), `connection` (`FromServer`), `dom` (`Template`, `Dom`, `document`), `shared::protocol`.

## Used by

- Nothing.

## Notes

- The notice takes no clicks (its row is `inert`, and the document's own node isn't an element), so it never gets in the way of the map under it.
- A document spawned on its own (`document`) is laid out by the node it is given: here one over the whole window, stacked, so the row placed in it is as wide as the window. Left as a row, the form in it has no width and the notice sits off the left edge.
- The server sends `PoiFound` once per poi, the first time the character finds it, and never for towns, which everyone knows of.
