# client: entities

Others on the map: mobs and other players, as tokens, for as long as the server says they can be seen. Also keeps the player's own token showing their kind of creature.

## Files

- `crates/client/src/entities/mod.rs`: `EntitiesPlugin`, `Others` (id to token), `read_server`, `show_moving`, `show_own_kind`, `picture`, `picture_size`. Unit tests.

## Public API

- `EntitiesPlugin`.

## Depends on

- `common` (`GameView`, `Icon`, `MapPosition`, `SmoothPosition`, `Token`, `TokenSize`), `connection` (`FromServer`), `inventory` (`Carried`), `player` (`Player`, `PlayerSystems`), `shared::creature`, `shared::id`, `shared::protocol`.

## Used by

- Nothing: only `main.rs`.

## Notes

- `EnterLevel`, `OnWorldMap`, `Teleported` and `Welcome` forget everyone, as the server's `Client::changed_map` does, and it says again who is there.
- Others glide towards where the server says at an ordinary walking pace, and jump if `TOO_FAR_BEHIND`. They have the same tokens as anything else, so `view3d` and the views draw them and hide them where nobody can see.
- Pictures are the Gervais tiles, with no centaur among them, so a centaur looks like a human. Appearance doesn't change anything drawn yet. A token's colour is how it feels about this player (`EntityLook::feeling`), told once when it comes into sight.
- Others can't be clicked yet.
