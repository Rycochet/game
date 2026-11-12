# client: login

The login screen and the account's corner. It only shows `common::LoggingIn` and sends `common::LoginChoice`; `connection` does the logging in (`connection/log_in.rs`), and `system_menu`'s Logout forgets an account and the chosen server at the startup menu.

## Files

- `crates/client/src/login/mod.rs`: `LoginPlugin` (registered apart in `main.rs`, as the plugin tuples are full).
- `crates/client/src/login/screen.rs`: the "Log in" dialog as markup (`dom`: `LOGIN`, layer 0, over the loading screen, as making a character is), set through `Dom` whenever `LoggingIn` changes (`asked` says what: `Choosing` shows "Log in with <label>" for each provider (copies of a template), Play as Guest if the server takes guests, Cancel, and the reason in red after a failure; `InBrowser` says to finish in the browser, with Open Again (once the server has sent the page) and Back). The buttons that don't go with what is asked are `hidden`, and the first on show has `class="main"`. `press_buttons` finds what a pressed button does from its id and `LoggingIn` (`choice_of`): a `LoginChoice`, or the page again (`common::open_in_browser`). Ids `login/provider.<n>`, `login/guest`, `login/cancel`, `login/open_again`, `login/back`. Tests.
- `crates/client/src/login/corner.rs`: "Playing on <server>" (`ChosenServer`, by `SavedServer::label`, so only while making a character on a server chosen in Connect; none for this computer's own) and "Logged in as <email>" (`AccountEmail`) at the bottom left, at the startup menu and while making a character, never in game (nor on Character Select). Its place is markup (`CORNER`: an `inert` `canvas`, `login/corner`, on a `bottom_strip()` over the startup menu's layer), but its words are a `Text` this module puts in that place (`bind_corner`) and are never given to `dom`: what `dom` holds can be read back through the debug API (`client.ui.html`), which must never answer with an email address.

## Public API

- `LoginPlugin`.

## Depends on

- `common` (`LoggingIn`, `LoginChoice`, `AccountEmail`, `MakingCharacter`, `Session`, `open_in_browser`), `widgets`, `dom`.

## Used by

- Nothing: other modules see `common`'s types.

## Notes

- The email is shown only here, and never written to a log: `AccountEmail`'s `Debug` hides it (see CLAUDE.md's core rules).
- `loading_screen` lets go of input and hides its bar while `LoggingIn::shows()`, as for `MakingCharacter`.
