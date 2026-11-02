# client: entities

Others on the map: mobs and other players, as tokens, for as long as the server says they can be seen. Also keeps the player's own token showing their kind of creature.

## Files

- `crates/client/src/entities/mod.rs`: `EntitiesPlugin`, `Others` (id to token), `read_server`, `show_moving`, `show_own_kind`, `offer_to_kill`, `kill_clicked`, `picture`, `picture_size`. Unit tests.

## Public API

- `EntitiesPlugin`.

## Depends on

- `common` (`Clickable`, `GameView`, `Icon`, `KillMob`, `MapPosition`, `SmoothPosition`, `Token`, `TokenSize`), `connection` (`FromServer`), `inventory` (`Carried`), `player` (`Player`, `PlayerSystems`), `view3d` (`ClickedToken`, `View3dSystems`), `shared::creature`, `shared::id`, `shared::protocol`.

## Used by

- Through `common::Other`: `inventory`.

## Notes

- `EnterLevel`, `OnWorldMap`, `Teleported` and `Welcome` forget everyone, as the server's `Client::changed_map` does, and it says again who is there.
- Others glide towards where the server says at an ordinary walking pace, and jump if `TOO_FAR_BEHIND`. They have the same tokens as anything else, so `view3d` and the views draw them and hide them where nobody can see.
- Pictures are the Gervais tiles, with no centaur among them, so a centaur looks like a human. Appearance doesn't change anything drawn yet. A token's colour is how it feels about this player (`EntityLook::feeling`), told when it comes into sight and again when it changes. `common::Other` is on each token, for `inventory`'s Nearby panel.
- Others can't be clicked, except mobs while the debug menu's Kill Mob is on. Other players never can be: their `feeling` is `None`.
