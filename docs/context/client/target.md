# client: target

Targeting creatures: a right click on one (or a finger held on it), Tab and Shift+Tab through them from nearest to furthest, and targets chosen for the player. The target gets a red reticule turning slowly on top of its token. Attacks go at the target (`abilities`); mobs' own targets are the server's and are never shown.

## Files

- `crates/client/src/target/mod.rs`: `TargetPlugin`; system order (`eat_tap` before `PlayerSystems::Input`; picking, keys and automatic picks after `View3dSystems::Click`, not while connecting; `place_reticule` in `PostUpdate` before transforms propagate).
- `crates/client/src/target/choose.rs`: the rules, apart from Bevy: `Seen`, `in_order` (Tab's list: enemies nearest first, ties by id; everyone if no enemies are in sight), `cycle` (next out or in, round again; from the nearest or furthest if the target isn't among them), `nearest_enemy`.
- `crates/client/src/target/seen.rs`: `seen_now` (everyone the server says is in sight, from `common::Other`).
- `crates/client/src/target/pick.rs`: `pick_by_right_click` (a right press and release less than `CLICK_SLOP`, 5 px, apart, on a creature's token; a drag turns the camera), `pick_by_long_press` (a finger still on a creature for `LONG_PRESS`, 0.5 s, from `TouchGesture::still_at`), `LongPress`, `eat_tap` (the tap lifting that finger makes is taken away, so it doesn't walk there).
- `crates/client/src/target/keys.rs`: `cycle_by_keys` (`Action::NextTarget`, Tab; `Action::PreviousTarget`, Shift+Tab).
- `crates/client/src/target/auto.rs`: `pick_attacker` (no target in sight: whoever begins an attack on the player, `AbilityStarted` aimed at them), `pick_after_death` (the target dies in sight, `EntityGone { died: true }`: the nearest other enemy in sight, or none).
- `crates/client/src/target/reticule.rs`: `Reticule`, `spawn_reticule` (red ring with four ticks pointing in, over a slightly larger black copy as its outline, both unlit), `place_reticule` (on the target's `TokenView`, a little above its top, under its picture; hidden while the target isn't in sight; one turn in `TURN_SECONDS`, 8, still with Reduce Motion), `reticule_mesh`.

## Public API

- `TargetPlugin`. The target itself is `common::Target`.

## Depends on

- `common` (`Target`, `Other`, `MapPosition`, `PointerPosition`, `TouchGesture`, `Preferences` for controls and Reduce Motion, `Action`, `GameView`), `connection` (`FromServer`), `player` (`Player`, `PlayerSystems`), `view3d` (`HoveredToken`, `View3dSystems`, `TokenView`, `TOKEN_DIAMETER`, `TOKEN_HALF_THICKNESS`), `shared::protocol`, `shared::sheet` (`Feeling`).

## Used by

- `dev_tools` (`client.state` reads `common::Target`).

## Notes

- Owner's rules: an enemy is a creature whose feeling is `Unfriendly`. Tab and Shift+Tab go through enemies only, unless none are in sight; a right click or long press can target anyone. A target going out of sight stays the target, its reticule hidden until it is seen again (after changing level too); a death out of sight is never told, so the id simply stays. Automatic picks treat a target out of sight as no target.
- Being attacked with no target in sight makes the attacker the target (owner), so an automatic Attack answers them.
- A right press is to open a menu of choices later, targeting among them (owner), so only a press and release that hardly moved counts.
