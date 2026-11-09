# client: notices

Short notices that go by themselves (a snackbar), at the top middle of the window under the status text. For now "Found: <kind>" when the server says the player has found a poi in the wilderness, and "Locked" when a door the player tried is locked (`DoorRefused` with `locked`). Not for anything that needs an answer (that is a dialog).

## Files

- `crates/client/src/notices/mod.rs`: `NoticesPlugin`; `spawn_notice` (Startup: a row across the window, `FROM_TOP` (64) down, holding a parchment `panel` with the text, hidden), `show_notices` (Update: `ServerMessage::PoiFound` or a locked `DoorRefused` sets the text and shows it for `SHOWN_FOR` (4 s), the newest in place of any showing), `found` (what the notice says, the kind in words).

## Public API

- `NoticesPlugin`.

## Depends on

- `common` (`words`), `connection` (`FromServer`), `widgets` (`panel`, `PANEL_EDGE`, `INK`, `TextSize`), `shared::protocol`.

## Used by

- Nothing.

## Notes

- The notice takes no clicks (`Pickable::IGNORE`), so it never gets in the way of the map under it.
- The server sends `PoiFound` once per poi, the first time the character finds it, and never for towns, which everyone knows of.
