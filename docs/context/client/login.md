# client: login

The login screen and the account's corner. It only shows `common::LoggingIn` and sends `common::LoginChoice`; `connection` does the logging in (`connection/log_in.rs`), and `system_menu`'s Logout forgets an account at the startup menu.

## Files

- `crates/client/src/login/mod.rs`: `LoginPlugin` (registered apart in `main.rs`, as the plugin tuples are full).
- `crates/client/src/login/screen.rs`: the "Log in" dialog (layer 0, over the loading screen, as making a character is), rebuilt whenever `LoggingIn` changes: `Choosing` shows "Log in with <label>" for each provider (the first `Main`), Play as Guest if the server takes guests, Cancel, and the reason in red after a failure; `InBrowser` says to finish in the browser, with Open Again (once the server has sent the page) and Back. `press_buttons` turns presses into `LoginChoice`, or opens the page again (`common::open_in_browser`). Ids `login/provider.<n>`, `login/guest`, `login/cancel`, `login/open_again`, `login/back`.
- `crates/client/src/login/corner.rs`: "Logged in as <email>" at the bottom left (`AccountEmail`), at the startup menu and while making a character, never in game (nor on Character Select).

## Public API

- `LoginPlugin`.

## Depends on

- `common` (`LoggingIn`, `LoginChoice`, `AccountEmail`, `MakingCharacter`, `Session`, `open_in_browser`), `widgets`.

## Used by

- Nothing: other modules see `common`'s types.

## Notes

- The email is shown only here, and never written to a log: `AccountEmail`'s `Debug` hides it (see CLAUDE.md's core rules).
- `loading_screen` lets go of input and hides its bar while `LoggingIn::shows()`, as for `MakingCharacter`.
