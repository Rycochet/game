# client: character_select

The Character Select screen: the account's characters in a grid on the left, each standing to the same scale with their name and when they were last played, the latest first, and a "+" to make another while the server allows; the chosen one described on the right (as the Character tab would, from their save) with when they were made, and Play, a red Delete (after a requester) and Cancel; Change User at the bottom left while logged in to an account (`common::AccountEmail`), sending `common::LoginChoice::ChangeUser`. It only shows `common::CharacterSelect` and sends `common::CharacterChoice`; `connection` talks to the server. Making a character is `character_create`'s. Native and web.

## Files

- `crates/client/src/character_select/mod.rs`: `CharacterSelectPlugin` (registered with `login`'s in `main.rs`, as the plugin tuples are full); its systems run only while `CharacterSelect::open`.
- `screen.rs`: `spawn_screen` (a full-screen layer, depth 0, hidden: "Characters" over a filling scroll area holding the `Grid`, wrapping; the right column 380 px wide: name, words, a gap, Created, then Play, Delete and Cancel), `show_screen`, `FORM`, `Button`.
- `cells.rs`: `fill_cells` (again whenever the list changes: a button per character, `CharacterSelect::sorted`, with its `item_art::portrait_look` picture 128 × 192, name and `common::local_date_time` of last played; then the "+" cell if `can_create`), `show_chosen` (the chosen cell `Selected`, its words light), `Cell`.
- `chosen.rs`: `Chosen` (resource: the character chosen, and whether Delete is being asked about), `keep_chosen` (on a new list: the same one if still there, or else the latest).
- `info.rs`: `show_info` (name, `character::describe_listed`, "Created ..."; Play and Delete disabled with nothing chosen; "No characters yet" when there are none).
- `change_user.rs`: `spawn_change_user` (under the grid, hidden), `show_change_user` (shown while `AccountEmail` is kept: never for a guest or single player), `ChangeUserRow`.
- `presses.rs`: `press_buttons` (a cell chooses its character, "+" sends `CharacterChoice::New`, Play `Play(id)`, Delete opens the requester, Cancel `Cancel`, Change User `LoginChoice::ChangeUser`; while the requester is up, only its answers count).
- `requester.rs`: "Delete <name>? This can't be undone.", a dialog at depth 1 with a red Delete (`CharacterChoice::Delete(id)`) and Cancel; a press outside it is Cancel.

## Public API

- `CharacterSelectPlugin`. Nothing else: others talk to it through `common::CharacterSelect` and `common::CharacterChoice`.

## Depends on

- `common` (`CharacterSelect`, `CharacterChoice`, `LoginChoice`, `AccountEmail`, `local_date_time`), `item_art` (`ItemPicture`, `portrait_look`), `character` (`describe_listed`), `widgets` (full-screen layer, dialog, buttons with `ButtonStyle::Danger`, filling scroll area, `UiId`), `shared::protocol`.

## Used by

- Nothing directly.

## Notes

- Widget ids: `character_select/character.<n>` (in the grid's order, latest first), `/new`, `/play`, `/delete`, `/cancel`, `/change_user`, and the requester's `/really_delete` and `/keep`.
- Pictures are to one scale: `item_art`'s portrait draws every kind with its frame at its usual height (`Frame::usual_height`), the tallest playable kind filling 85% of the picture's height, feet on the floor.
- Health and Mana aren't shown: their most is worked out on joining, from playtime the client isn't told. Speed isn't either, as it goes by what is carried.
