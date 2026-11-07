# client: abilities

What the character can do: the Character panel's Abilities page (B), every ability they know (`shared::ability::known`, for the body they are in) as ability widgets flowing in rows, Attack first, under a Skills button opening the Skills page (`common::SkillsPageOpen`); and the action bar, ten slots, five either side of the Health and Mana circle, with dragging between them. Not the panel or its tabs: those are `inventory`'s, which spawns the page as a `common::AbilitiesPage` node. Native and web.

## Files

- `crates/client/src/abilities/mod.rs`: `AbilitiesPlugin`, the module list.
- `crates/client/src/abilities/page.rs`: `fill_pages` (fills each new `AbilitiesPage`, and again when the body changes: `DrawnFor`), `PageAbility` (on each ability there), `look` (how an ability shows, also on the bar), `open_skills` (the Skills button sets `common::SkillsPageOpen`), `ability_id` (widget ids `abilities/<id>`).
- `crates/client/src/abilities/icons.rs`: `icon_path`: each ability's game-icons.net icon in `assets/icons/` (listed in `xtask/icons.txt`; a test checks each is there).
- `crates/client/src/abilities/own_bar.rs`: `OwnBar` (the bar as the server last said, with changes since), `take_bar`, `OwnBar::change` (checked by `ActionBar::apply`, made here and sent as `ClientMessage::ActionBar`).
- `crates/client/src/abilities/bar.rs`: `BarSide`, `SlotNode`, `spawn_bar` (Startup), `place_bar` (either side of `common::VitalsCircle`, hidden with it), `show_bar` (each slot drawn again when the bar, body, keys or aim change: its ability or an empty frame, the key's `Binding::short_label` bottom right in outlined words, a `set_edge` for automatic use, green or red while something dragged is over it).
- `crates/client/src/abilities/spots.rs`: `Spot` (`Slot`, `Listed`, `Page`), `Spots` (what is in front at a point).
- `crates/client/src/abilities/drag.rs`: `Drag`, `Aim`, `drag_abilities` (`PreUpdate`, after `PointerSystems`, before `WidgetSystems::Presses`, so presses on abilities never reach panels or the map), `drop_change`.
- `crates/client/src/abilities/held.rs`: `spawn_held`, `show_held` (the dragged icon at the pointer), `use_by_key`.
- `crates/client/src/abilities/use_ability.rs`: `UseSlot` (message: a slot clicked, tapped or its key), `take_timing` (`AbilityStarted` and a cancel into `common::OwnAbility`, and the cost as `SpentEnergy`, so steps wait here as on the server and a journey stops), `use_slots` (its ability at the target, if it can be done and nothing is waiting: `UseAbility`), `use_automatic` (each frame: an automatic Attack slot, an enemy targeted, in sight and in reach by the held weapon, in a site).
- `crates/client/src/abilities/wait.rs`: `WaitShade` (spawned over every active ability on the bar), `wait_left`, `show_wait` (the shade drains from the top as the wait to act again ends; every ability waits alike, sharing energy).
- `crates/client/src/abilities/cancel.rs`: `cancel_by_escape` (`PreUpdate`, before `WidgetSystems::Front`: Esc while winding up calls it off instead of opening the menu, unless something Esc closes is open), `cancel_by_moving` (a `TakeStep` or `TravelTo` while winding up); both send `CancelAbility` and `common::CancelledEnergy` (owner, 2026-11-07).

## Public API

- `AbilitiesPlugin`.

## Depends on

- `common` (`AbilitiesPage`, `SkillsPageOpen`, `VitalsCircle`, `Action::UseSlot`, `Preferences`, `UiPointer`, `TouchGesture`, `PointerSystems`), `connection` (`FromServer`, `ToServer`), `inventory` (`Carried`, for the body), `widgets` (`spawn_ability`, `set_edge`, `spawn_outlined_text`, `stops_presses`, buttons), `shared::ability`.

## Used by

- Nothing.

## Notes

- Owner's design: from the page onto a slot puts an ability there (replacing what was there; one already on the bar moves, with its automatic use); slot onto slot swaps; slot onto the page takes it off and forgets its automatic use; dropped anywhere else nothing changes, so nothing is lost by accident. Abilities can't be sorted on the page; filtering and searching come later.
- Mouse: press and drag to move; a click on a slot uses it; right-click on a slot turns automatic use on or off. Touch: drag to move; a tap uses; a finger held still on a slot turns automatic use on or off (a drag starts only when the finger moves). Keys: `Action::UseSlot(0..9)`, 1 to 0 by default, in Settings' Action Bar group. Using one asks the server for it at the target; an automatic one is asked for whenever it can be done (owner: target, in reach, an enemy for Attack), so it can be left on. The next is asked a quarter of a second before the last's recovery ends (`OwnAbility::is_busy`), as the server keeps it until then; otherwise slow frames would slow attacks. Only abilities that may be made automatic can be.
- The bar takes presses (`stops_presses`), so clicks on it never walk the player; right-clicks on a slot never target.
- Passive abilities are drawn round and never greyed; active ones the player can't use now are greyed with a dull red frame (also on the bar, after a shapeshift). The slot's tip names the ability and its key in full.
- Ids: `abilities/skills`, `abilities/<ability>` (`attack`, `night_vision`, ...), `bar/slot.<0..9>`.
- Narrow screens will show only the first few slots and may move the circle: a later mobile task (owner).
