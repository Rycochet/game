# client: accounts_panel

The Accounts panel, a moveable panel opened from the system menu's Accounts (`common::OpenAccounts`) for a role with something on it, and keeping the account's role (`common::AccountRole`) from `Welcome` and `RoleAnswer::Changed`. It only asks (`ClientMessage::Role(RoleAsk)`) and shows the answers: the server checks every permission.

## Files

- `mod.rs`: `AccountsPanelPlugin`, `FONT`; its systems in order: `answers::read_answers`, `open::open_panel`, the `input` ones, `open::notice_closed`, `fill::fill_page`.
- `state.rs`: `Panel` (resource: the panel's entities, tabs, everything shown and typed, `dirty` to build the page again), `Tab` (`Find`, `Roles`, `Audit`), `tabs_for(powers)` (by `LookUp`, `RenameRoles`, `ReadAudit`), `FOR` (Always, an hour, a day, a week, 30 days), `FORM` (`accounts`).
- `open.rs`: `open_panel` (`widgets::spawn_moveable_panel`, tabs when more than one, asks for the roles and the log), `notice_closed` (forgets it once closed; closes it on leaving the game), `WIDTH`.
- `answers.rs`: `read_answers`: `AccountRole`; `Found`, `Audit`, `Roles` and `Refused` into `Panel`.
- `input.rs`: `choose_tab` (the log is asked for again on its tab), `keep_typing`, `keep_choices`, `press_buttons` (Find, Apply, Save, Newer, Older into `RoleAsk`s).
- `fill.rs`: `fill_page` (the chosen tab's page, and why the last ask was refused in `WARNING_TEXT`), `words`, `labelled`, `control`, `ROW`.
- `find.rs`, `roles.rs`, `audit.rs`: each page and its widgets' markers.

## Public API

- `AccountsPanelPlugin`.

## Depends on

- `common` (`AccountRole`, `OpenAccounts`, `Session`), `connection` (`ToServer`, `FromServer`), `widgets` (moveable panel, tabs, text boxes, dropdowns, buttons, scroll area), `shared::protocol` (`RoleAsk`, `RoleAnswer`, `FoundAccount`), `shared::role`.

## Used by

- `system_menu` (its Accounts entry, through `common`).

## Notes

- Widget ids: `accounts/panel`, `accounts/page` (tabs), `accounts/name`, `find`, `give_role`, `give_for`, `reason`, `apply`, `role`, `role_name`, `role_title`, `save`, `audit`, `newer`, `older`.
- The page is built again after every answer; what was typed is kept in `Panel`, so it survives.
- Giving a role to someone else can only be tried with a second account (the server's tests cover it); single player has only its own, which can't be changed.
