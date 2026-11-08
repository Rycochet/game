# shared: ability

Abilities, the named things a creature can do: active ones done when asked (and put on the action bar), toggles turned on and off there, passive ones always at work. It also holds the action bar's slots and the rules for changing them, which the client predicts with and the server checks by. How Attack works when used is `shared::combat`'s and the server's, Heal's `shared::magic`'s; abilities don't hook onto skills by name yet, Attack using the held weapon's node and Heal Healing's. Plan: `docs/context/abilities-skills.md`.

## Files

- `crates/shared/src/ability.rs`: `Ability` (`ALL`, `name`, `description` (a line for its tip), `leans` (the attributes it leans on, which colour it: Attack and Kick Strength and Agility, Trample Strength and Toughness, Flutter Agility, Night Vision Perception, Heal Empathy as Healing), `mana_cost`, `use_kind`, `is_active` (active or a toggle: it goes on the bar), `is_toggle`, `can_use`), `Use::{Active, Passive, Toggle}`, `known(kind)`. Tests.
- `crates/shared/src/ability/bar.rs`: `ActionBar`, `BarSlot`, `BarChange`, `BarRefusal`, `BAR_SLOTS`.

## Public API

- `Ability`: `Attack`, `Heal` and `Sneak` (every creature; Heal as there is no way to learn spells yet; Sneak a toggle, holding steps on levels to a sneaking pace, `energy::sneak_step_energy`), and the kinds' own `Flutter`, `Kick`, `NightVision` (passive), `Trample`. Also `sheet::Ability`. Messages carry it by its place, so new ones go at the end of the enum; `ALL` is the order players see (Attack first).
- `known(kind)`: Attack, Heal and the kind's own (`CreatureDef::abilities`), for the body it is in now, so shapeshifting changes them.
- `Ability::can_be_automatic()`: whether a slot may be made automatic, done whenever it can be (owner, 2026-11-07): Attack and Heal so far; `BarChange::ToggleAuto` refuses the rest (`BarRefusal::NotAutomatic`), though turning one off always works.
- `Ability::can_use(kind)`: active and known. The client greys out what fails it, and what costs more Mana (`mana_cost`) than there is.
- `ActionBar { slots: [Option<BarSlot { ability, auto }>; 10], given }`: `Default` is a new character's (Heal in the first slot and Attack in the second, so healing comes first if both are automatic: Claude's suggestion, 2026-11-07; also for saves from before the bar); `apply(change, kind)`, `slot_of(ability)`, `catch_up` (a bar saved before an ability was added to new bars gets it once, in its first empty slot, counted by `given`; the server calls it on loading). Automatic slots take turns from the first: the first whose ability can be done is (owner, 2026-11-07).
- `BarChange::{Place { slot, ability }, Clear { slot }, Swap { a, b }, ToggleAuto { slot }}`; `BarRefusal::{NoSlot, Passive, Unknown, Empty, NotAutomatic}`.

## Depends on

- `creature` (`CreatureKind::def().abilities`).

## Used by

- `sheet` (re-exports `Ability`), `creature` (kinds' abilities), `protocol` (`ClientMessage::ActionBar`, `ServerMessage::ActionBar`), server `connections/action_bar.rs` and `storage` (`CharacterSave::action_bar`), client `abilities` (the Abilities page and the action bar).

## Notes

- Bar rules (owner): an ability is in at most one slot; placing it from the Abilities page onto a filled slot replaces what was there, and placing one already on the bar moves it with its `auto`; swapping keeps each slot's `auto` with its ability; taking one off (dropping it on the Abilities page) forgets its `auto`. Only active, known abilities can be placed. One no longer known (after shapeshifting) stays in its slot, shown as unusable.
- Which of the kinds' abilities are active was Claude's guess, agreed by the owner for now (Night Vision passive, the rest active); see `decisions.md`.
