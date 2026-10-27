# common

Shared building blocks every module may use. Keep it small: only things that genuinely need sharing.

## Files

- `src/common/mod.rs`: `CommonPlugin`; spawns the single `Camera2d` at startup.

## Public API

- `CommonPlugin`: registered in `main.rs`.

(No shared types yet. When two modules need the same component/resource/message, put it here.)

## Depends on

- Nothing (Bevy only).

## Used by

- All modules (implicitly: they rely on the camera existing).

## Notes

- The camera does not follow the player yet. When it does, that logic belongs in the module that owns the behaviour (likely `player` or a future `camera` module), reading the `Player` component.
