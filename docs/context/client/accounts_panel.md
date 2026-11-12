# client: accounts_panel

The Accounts panel, a moveable panel opened from the debug menu's Accounts... under Admin Tools (`common::OpenAccounts`) for a role with something on it, and keeping the account's role (`common::AccountRole`) from `Welcome` and `RoleAnswer::Changed`. It only asks (`ClientMessage::Role(RoleAsk)`) and shows the answers: the server checks every permission.

## Files

- `mod.rs`: `AccountsPanelPlugin`, `FONT`; its systems in order: `answers::read_answers`, `open::notice_closed`, `open::open_panel`, `open::bind_panel`, the `input` ones, `fill::fill_page`.
- `state.rs`: `Panel` (resource: the panel's document, its built panel and page, tabs, everything shown and typed, `dirty` to build the page again), `Tab` (`Find`, `Roles`, `Audit`), `tabs_for(powers)` (by `LookUp`, `RenameRoles`, `ReadAudit`), `FOR` (Always, an hour, a day, a week, 30 days), `FORM` (`accounts`).
- `open.rs`: `ACCOUNTS` (the panel as markup, `dom`: a moveable `panel`, its `tab-bar` and the page's `div`), `open_panel` (spawns the document, gives the tab bar the role's tabs with `Dom::set_options` or hides it for one page, asks for the roles and the log), `bind_panel` (notes the built panel and page by id), `notice_closed` (despawns the document when the panel's button closed it, `UiValue::Closed`, or on leaving the game).
- `answers.rs`: `read_answers`: `AccountRole`; `Found`, `Audit`, `Roles` and `Refused` into `Panel`.
- `input.rs`: `choose_tab` (from `UiEvent`; the log is asked for again on its tab), `keep_typing`, `keep_choices`, `press_buttons` (Find, Apply, Save, Newer, Older into `RoleAsk`s).
- `fill.rs`: `fill_page` (the chosen tab's page, and why the last ask was refused in `WARNING_TEXT`), `words`, `labelled`, `control`, `ROW`.
- `find.rs`, `roles.rs`, `audit.rs`: each page and its widgets' markers.

## Public API

- `AccountsPanelPlugin`.

## Depends on

- `common` (`AccountRole`, `OpenAccounts`, `Session`), `connection` (`ToServer`, `FromServer`), `dom` (`Dom`, `Template`, `document`, `UiEvent`), `widgets` (`RaisePanel`, `UiId`, and for the pages still built by code: text boxes, dropdowns, buttons, scroll area), `shared::protocol` (`RoleAsk`, `RoleAnswer`, `FoundAccount`), `shared::role`.

## Used by

- `debug_menu` (its Accounts... entry, through `common`).

## Notes

- Widget ids: `accounts/panel`, `accounts/tabs`, `accounts/page` (the page's place), `accounts/name`, `find`, `give_role`, `give_for`, `reason`, `apply`, `role`, `role_name`, `role_title`, `save`, `audit`, `newer`, `older`.
- The shell is markup; the page inside it is still built again by code after every answer (the pages move to markup next); what was typed is kept in `Panel`, so it survives.
- Giving a role to someone else can only be tried with a second account (the server's tests cover it); single player has only its own, which can't be changed.
