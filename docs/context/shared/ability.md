# shared: ability

Abilities, the named things a creature can do: active ones done when asked (and put on the action bar), passive ones always at work. It also holds the action bar's slots and the rules for changing them, which the client predicts with and the server checks by. Not how an ability works when used: nothing is used yet (combat is a later task), and abilities don't hook onto skills yet (after the skill tree, owner). Plan: `docs/context/abilities-skills.md`.

## Files

- `crates/shared/src/ability.rs`: `Ability` (`ALL`, `name`, `use_kind`, `is_active`, `can_use`), `Use`, `known(kind)`. Tests.
- `crates/shared/src/ability/bar.rs`: `ActionBar`, `BarSlot`, `BarChange`, `BarRefusal`, `BAR_SLOTS`.

## Public API

- `Ability`: `Attack` (every creature), and the kinds' own `Flutter`, `Kick`, `NightVision` (passive), `Trample`. Also `sheet::Ability`. Messages carry it by its place, so new ones go at the end of the enum; `ALL` is the order players see (Attack first).
- `known(kind)`: Attack and the kind's own (`CreatureDef::abilities`), for the body it is in now, so shapeshifting changes them.
- `Ability::can_use(kind)`: active and known. It will gain costs and waiting times; the client greys out what fails it.
- `ActionBar { slots: [Option<BarSlot { ability, auto }>; 10] }`: `Default` is a new character's (Attack in the first slot, also for saves from before the bar); `apply(change, kind)`, `slot_of(ability)`.
- `BarChange::{Place { slot, ability }, Clear { slot }, Swap { a, b }, ToggleAuto { slot }}`; `BarRefusal::{NoSlot, Passive, Unknown, Empty}`.

## Depends on

- `creature` (`CreatureKind::def().abilities`).

## Used by

- `sheet` (re-exports `Ability`), `creature` (kinds' abilities), `protocol` (`ClientMessage::ActionBar`, `ServerMessage::ActionBar`), server `connections/action_bar.rs` and `storage` (`CharacterSave::action_bar`), client `abilities` (the Abilities page).

## Notes

- Bar rules (owner): an ability is in at most one slot; placing it from the Abilities page onto a filled slot replaces what was there, and placing one already on the bar moves it with its `auto`; swapping keeps each slot's `auto` with its ability; taking one off (dropping it on the Abilities page) forgets its `auto`. Only active, known abilities can be placed. One no longer known (after shapeshifting) stays in its slot, shown as unusable.
- Which of the kinds' abilities are active was Claude's guess, agreed by the owner for now (Night Vision passive, the rest active); see `decisions.md`.
