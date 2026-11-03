# client: system_menu

The system menu: Esc or the gear button top right (the way in on touch devices) opens a small panel in three sections: Continue; Settings and Credits; Logout and Quit. While it is open, single player is paused. The same menu, with Play for Continue, is the startup menu over the title, until Play is pressed (`common::Session`). Native and web.

## Files

- `crates/client/src/system_menu/mod.rs`: `SystemMenuPlugin`, `SystemMenu`, the gear icon (drawn in code), the panels, `take_input` (`PreUpdate`, `WidgetSystems::Front`, while `takes_input`: at the startup menu or once there is a world), `show_menu`, `show_buttons` (Play, Continue and Logout by `Session` and `ServerClock::local_server`), `ask_for_pause`, `show_ended_page`. Unit tests. Beside it: `state.rs` (`SystemMenu`, `Action`, `MenuButton`), `spawn.rs` (gear, layers, requester, Settings, `spawn_title_row`), `panel.rs` (the menu's sections, `SECTIONS`), `input.rs`, `show.rs`.
- `crates/client/src/system_menu/credits.rs`: the Credits layer (`CREDITS`: what was done and by whom), a scroll area that `start_credits` restarts from the top with `widgets::AutoScroll` each time it opens. Test: every bold label on `docs/credits.md` is in it.
- `crates/client/src/system_menu/settings.rs`: Settings' tabs and pages (`spawn_pages`; hidden pages take no room): Gameplay (empty) and Interface, with the interface size dropdown (Auto, then `widgets::INTERFACE_PERCENTS`) and "Enable touch" (on and disabled on a touch-only device, `widgets::TouchOnly`). `fill_interface_page` rebuilds it when `Preferences` change; `use_pages` switches pages and writes the choices into `Preferences`. Unit tests.

## Public API

- `SystemMenuPlugin`.
- `SystemMenu` (resource): `Closed`, `Menu`, `Confirming` (the requester above the menu), `Settings`, `Credits` (above the menu); `is_open()`. Set by `dev_tools` (`HOW_DEBUG_SYSTEM_MENU`).

## Depends on

- `common` (`GameView`, `ServerClock`, `Session`, `TouchGesture`, `Preferences`, `InterfaceSize`), `connection` (`ToServer`), `debug_menu` (`DebugMenu::open`), `minimap` (`FullMap::open`), `widgets`, `shared::protocol`.

## Used by

- `dev_tools`.

## Notes

- Startup menu: while `Session::AtStart` the menu is always open, and Esc, outside presses and Continue do nothing; Play sets `Playing` and closes it. Logout (shown only while playing on a server that isn't built in: always on the web) sets `AtStart` and keeps the menu open, and `connection` hangs up. Credits are kept by hand beside `docs/credits.md` (the owner's name is only in the game's).
- Owner's design: no close button or title on the menu; "Paused" in a space above the first button that is always there, so nothing moves when it appears, and multiplayer has the same layout with it empty. Quit asks "Are you sure?" in a small requester, whose "No" closes only that. Settings is a 90% layer with tabs, breaking the rule that a layer above is smaller (owner's exception).
- Claude's choices: Esc or a press outside closes what is on top, one layer at a time (touch has no Esc, and the menu no close button). Settings closes back to the menu. The gear sits in the corner outside the round minimap, so a low sun or moon can pass behind it, and is hidden while connecting, as the warning button is.
- Esc: text boxes and dropdowns use it first (`Presses`). `take_input` runs before the debug menu and full map, leaving it to them if they are open. While open, the menu takes all keys, the wheel, gestures and right button, so backtick and M do nothing.
- Pausing: `ask_for_pause` sends `SetPaused` as the menu opens and closes, if the server said it can (`ServerClock::can_pause`). Only the server's answer stops anything (`connection`, see there), and "Paused" shows only then. In multiplayer nothing pauses: keys are blocked, but click travel carries on.
- Quit: `AppExit`, which `connection` hangs up on. In a browser `show_ended_page` also shows the `#ended` screen in `index.html`, a copy of the loading screen with a "Click to reload" button, over the stopped canvas. Its colours and sizes are shared with `#loading` there.
