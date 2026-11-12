# client: accounts_panel

The Accounts panel, a moveable panel opened from the debug menu's Accounts under Admin Tools (`common::OpenAccounts`) for a role with something on it, and keeping the account's role (`common::AccountRole`) from `Welcome` and `RoleAnswer::Changed`. It only asks (`ClientMessage::Role(RoleAsk)`) and shows the answers: the server checks every permission. It is markup (`dom`), changed in place.

## Files

- `mod.rs`: `AccountsPanelPlugin`; its systems in order: `answers::read_answers`, `open::notice_closed`, `open::open_panel`, `open::bind_panel`, `input::read_events`, `show::show_panel`.
- `markup.rs`: `ACCOUNTS`, the whole panel: a moveable `panel`, its `tab-bar`, a page per tab (`find_page`, `roles_page`, `audit_page`), and the lines for a role with nothing here and for a refusal.
- `state.rs`: `Panel` (resource: the panel's document and built panel, its tabs, everything shown and typed; `dirty` to show it again, and `new_found`, `new_roles`, `new_words` for what `show` must put into controls), `Tab` (`Find`, `Roles`, `Audit`; `ALL`, `label`, `page`: its page's id), `tabs_for(powers)` (by `LookUp`, `RenameRoles`, `ReadAudit`), `FOR` (Always, an hour, a day, a week, 30 days).
- `open.rs`: `open_panel` (spawns the document, gives the tab bar the role's tabs with `Dom::set_options` or hides it for one page, asks for the roles and the log), `bind_panel` (notes the built panel, for bringing it to the front), `notice_closed` (despawns the document when the panel's button closed it, `UiValue::Closed`, or on leaving the game).
- `answers.rs`: `read_answers`: `AccountRole`; `Found`, `Audit`, `Roles` and `Refused` into `Panel`.
- `input.rs`: `read_events`: the panel's `UiEvent`s by id: tabs (the log is asked for again on its tab), typing and choices kept in `Panel`, and Find, Apply, Save, Newer, Older into `RoleAsk`s.
- `show.rs`: `show_panel` (when `dirty`: the chosen tab's page shown and the others hidden, each page, and why the last ask was refused, `p class="warning"`), `numbered` (options chosen by their place), `capitalised`.
- `find.rs`, `roles.rs`, `audit.rs`: `show_find`, `show_roles`, `show_audit`: each page's words, what is hidden, and its dropdowns' options, through `Dom`; `find::about`; `audit::PAGE`.

## Public API

- `AccountsPanelPlugin`.

## Depends on

- `common` (`AccountRole`, `OpenAccounts`, `Session`), `connection` (`ToServer`, `FromServer`), `dom` (`Dom`, `Template`, `document`, `UiEvent`, `UiValue`), `widgets` (`RaisePanel`, `UiId`), `shared::protocol` (`RoleAsk`, `RoleAnswer`, `FoundAccount`), `shared::role`.

## Used by

- `debug_menu` (its Accounts entry, through `common`).

## Notes

- Ids, all in the form `accounts`: `panel`, `tabs`, the pages `find_page`, `roles_page`, `audit_page`; `name`, `find`, `found` (with `about`, `others`, `cannot`), `give` (with `give_role`, `give_for`, `reason`, `apply`); `asking`, `roles` (with `role`, `role_name`, `role_title`, `save`); `no_audit`, `audit` (the scroll area), `lines`, `newer`, `older`; `nothing`, `refused`.
- Nothing is built again after an answer: controls keep what was typed by themselves. `show` writes into a control only what is new (a new account's roles, the roles told again, another role's name and title), so an answer about something else never writes over what is being typed. `UiEvent`s arrive the frame after the widget changed.
- A dropdown's options are numbered (`value` is the place in the list), as two roles may be shown by the same words.
- The times a role is given for are written in the markup; a test checks them against `FOR`.
- The audit log is one piece of small writing, a line of the log to a line.
- Giving a role to someone else can only be tried with a second account (the server's tests cover it); single player has only its own, which can't be changed.
