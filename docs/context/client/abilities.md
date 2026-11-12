# client: abilities

What the character can do: the Character panel's Abilities page (B), every ability they know (`shared::ability::known`, for the body they are in) as ability widgets flowing in rows, Attack first; and the action bar, ten slots, five either side of the Health and Mana circle, with dragging between them. Not the panel or its tabs: those are `inventory`'s, which spawns the page as a `common::AbilitiesPage` node. Native and web.

## Files

- `crates/client/src/abilities/mod.rs`: `AbilitiesPlugin`, the module list.
- `crates/client/src/abilities/page.rs`: `fill_pages` (gives each new `AbilitiesPage` its `PageList`, a `dom::AbilityGridShown` of what the body knows, which draws itself; again when the body changes: `DrawnFor`), `PageAbility` (on each ability drawn there), `name_abilities` (says how every ability looks for this body, by its id, in `dom::KnownAbilities`, for an `<object type="ability">` anywhere), `look` (how an ability shows, also on the bar and while dragged), `ability_id` (widget ids `abilities/<id>`).
- `crates/client/src/abilities/icons.rs`: `icon_path`: each ability's game-icons.net icon in `assets/icons/` (listed in `xtask/icons.txt`; a test checks each is there).
- `crates/client/src/abilities/own_bar.rs`: `OwnBar` (the bar as the server last said, with changes since), `take_bar`, `OwnBar::change` (checked by `ActionBar::apply`, made here and sent as `ClientMessage::ActionBar`).
- `crates/client/src/abilities/bar.rs`: `BarSide`, `SlotNode`, `spawn_bar` (Startup: each slot a `dom::AbilitySlotShown`, which draws itself), `place_bar` (either side of `common::VitalsCircle`, hidden with it), `show_bar` (says what each slot shows again when the bar, body, keys, aim or having Heal's Mana change: its ability, the key's `Binding::short_label`, automatic use, a toggle being on, being queued, the shade for the wait, and green or red while something dragged is over it).
- `crates/client/src/abilities/spots.rs`: `Spot` (`Slot`, `Listed`, `Page`), `Spots` (what is in front at a point, `at`, and what a node in front is or is in, `of`).
- `crates/client/src/abilities/drag.rs`: `Drag`, `Aim`, `drop_change`, and two systems in `PreUpdate` either side of `dom::HoldObjects`, which moves, draws and lets go of what is dragged as it does items: `drag_abilities` (after `PointerSystems`, before `dom::DomInput::Presses`, so presses on abilities never reach panels or the map: a click uses a slot, a right-click or still finger turns automatic use on or off, and a drag picks the ability up with `HeldObject::pick_up`, unless something is already held) and `drop_abilities` (the `Aim` while one is held, and the change to the bar on `dom::Dropped`).
- `crates/client/src/abilities/held.rs`: `use_by_key`.
- `crates/client/src/abilities/mark.rs`: `mark_abilities` (`PostUpdate`, after `dom::DrawObjects`): `PageAbility` and the id on each ability the page's list drew, and the ids `bar/on.<slot>` and `bar/queued.<slot>` on those parts of a drawn slot.
- `crates/client/src/abilities/use_ability.rs`: `UseSlot` (message: a slot clicked, tapped or its key), `take_timing` (`AbilityStarted` and a cancel into `common::OwnAbility` and `Manual`, and the cost as `SpentEnergy`, so steps wait here as on the server and a journey stops), `ask` (`UseAbility`; target `None` is the player), `use_automatic` (each frame, in a site, never while `player::PlayerMoving` or `Manual::holds`: the automatic slots in bar order, the first whose time has come: Attack with an enemy targeted, in sight and in reach by the held weapon; Heal on the player at a quarter of their Health, with the Mana).
- `crates/client/src/abilities/vitals_now.rs`: `affords` (the Mana for an ability, from `common::OwnVitals` as refilled here), `wants_healing` (`magic::needs_healing`).
- `crates/client/src/abilities/sneak.rs`: `take_sneaking` (`common::Sneaking` from `ServerMessage::Sneaking`, off on `Welcome`). A toggle pressed on the bar (`use_slots`) is asked for at once with no target, never queued; its slot has a steady green edge while on (`bar/on` widget id).
- `crates/client/src/abilities/manual.rs`: `Manual` (abilities chosen by hand: queued in press order, each once, then sent, then winding up until it lands; `press` queues or takes out), `use_slots` (a `UseSlot` presses its ability), `ask_wanted` (asks for the first queued that can be done, at the target, once the player can act, passing over those that can't; an automatic ask still waiting on the server is taken back first with `WithdrawAbility`, never an earlier manual one).
- `crates/client/src/abilities/wait.rs`: `wait_left`, `show_wait` (each slot's shade, `dom::SlotPart::Shade`, drains from the top as the wait to act again ends; every ability waits alike, sharing energy).
- `crates/client/src/abilities/cancel.rs`: `cancel_by_escape` (`PreUpdate`, before `dom::DomInput::Front`: Esc while winding up calls it off instead of opening the menu, unless something Esc closes is open), `cancel_by_moving` (a `TakeStep` or `TravelTo` forgets what was chosen by hand, takes back an ask still waiting on the server with `WithdrawAbility`, and calls off a wind-up); calling off sends `CancelAbility` and `common::CancelledEnergy` (owner, 2026-11-07).

## Public API

- `AbilitiesPlugin`.

## Depends on

- `player` (`PlayerMoving`, `PlayerSystems`, `TakeStep`, `TravelTo`), `common` (`AbilitiesPage`, `Sneaking`, `VitalsCircle`, `OwnVitals`, `Other`, `Target`, `Action::UseSlot`, `Preferences`, `UiPointer`, `TouchGesture`, `PointerSystems`), `connection` (`FromServer`, `ToServer`), `inventory` (`Carried`, for the body), `dom` (`AbilityGridShown`, `AbilitySlotShown`, `AbilityThing`, `AbilityCell`, `SlotPart`, `HeldObject`, `Dropped`, `HoldObjects`, `DrawObjects`), `widgets` (`AbilityLook`, `attribute_colour`, `stops_presses`), `shared::ability`, `shared::magic`.

## Used by

- Nothing.

## Notes

- Owner's design: from the page onto a slot puts an ability there (replacing what was there; one already on the bar moves, with its automatic use); slot onto slot swaps; slot onto the page takes it off and forgets its automatic use; dropped anywhere else nothing changes, so nothing is lost by accident. Abilities can't be sorted on the page; filtering and searching come later.
- Mouse: press and drag to move; a click on a slot uses it; right-click on a slot turns automatic use on or off. Touch: drag to move; a tap uses; a finger held still on a slot turns automatic use on or off (a drag starts only when the finger moves). Keys: `Action::UseSlot(0..9)`, 1 to 0 by default, in Settings' Action Bar group. Using one asks the server for it at the target; an automatic one is asked for whenever it can be done (owner: target, in reach, an enemy for Attack; a quarter of the player's Health for Heal), so it can be left on. Automatic slots take turns in bar order, keys 1 to 0: the first whose time has come is asked for, so their order is the player's choice of how to fight (owner, 2026-11-07). The next is asked a quarter of a second before the last's recovery ends (`OwnAbility::is_busy`), as the server keeps it until then; otherwise slow frames would slow attacks. Only abilities that may be made automatic can be.
- Clicks come first (owner, 2026-11-07): while the player moves (a journey with way to go, steps waiting, a movement key held) or abilities chosen by hand are queued, asked for or winding up, automatic ones wait, starting again once it is over. Presses queue in order (owner: a second press of a queued one takes it out; pressing the one winding up queues it again), and moving empties the queue while Esc only calls off the wind-up; one asked for early and still waiting on the server is taken back (`WithdrawAbility`, at no cost) so the press or the move goes in its place. One already winding up finishes, unless Esc or a move calls it off. The Update systems run after `PlayerSystems::Travel`, so they know whether the player is moving this frame.
- The bar takes presses (each side's row is `dom::Opaque`), so clicks on it never walk the player; right-clicks on a slot never target.
- Each ability stands on the colour of what it leans on (`widgets::attribute_colour` of `Ability::leans`), dark when active and pastel when passive, on the page, the bar and while dragged.
- Heal by hand (owner, 2026-11-07): the target if it isn't an enemy, otherwise the player; never an enemy, though the server allows it (healing one, for taming, is to come from the right-press menu of choices).
- Passive abilities are drawn round and never greyed; active ones the player can't use now are greyed with a dull red frame (also on the bar, after a shapeshift, and while there isn't the Mana). The tip names the ability (on the bar with its key in full) over its `description`.
- Ids: `abilities/<ability>` (`attack`, `heal`, `night_vision`, ...), `bar/slot.<0..9>`.
- Narrow screens will show only the first few slots and may move the circle: a later mobile task (owner).
