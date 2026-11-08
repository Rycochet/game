# client: touch

Turns fingers into what the rest of the game reads, so no other module needs to follow fingers itself. It decides what a touch means, not what that does.

## Files

- `crates/client/src/touch/mod.rs`: `TouchPlugin`. `read_touches` (`PreUpdate`, in `common::PointerSystems`) feeds Bevy's `Touches` to the private `Tracker`, which follows one contact (first finger down to last finger up) and is unit tested without Bevy. Constants `TAP_SLOP` (10 logical pixels), `MIN_FINGER_GAP`.

## Public API

- `TouchPlugin`. What it writes is in `common`: `PointerPosition`, `UiPointer` (the same in interface pixels, by `UiScale`), `TouchGesture` (window pixels), `PointerSystems`.

## Depends on

- `common`.

## Used by

- Nothing directly. Through `common`: `view3d` (camera, token clicks), `world_map` and `dungeon_map` (clicks), `minimap`, `inventory`, `widgets` (hold buttons: `first_finger`, `tap`).

## Notes

- A tap (one finger, lifted within `TAP_SLOP` of where it landed, no second finger in the contact, not cancelled) is a left press in `ButtonInput<MouseButton>` for one frame, on lifting, so the start of a pinch never walks the player. Holding a finger still and lifting it is still a tap.
- `PointerPosition` is the finger while exactly one is down and on the tap's frame, otherwise `Window::cursor_position`. So a held finger names what is under it, and sliding away cancels. After a tap the pointer stays there for `TAP_SHOWS_SECONDS`, or until a finger lands, so what was tapped shows its name; that is also how a button's name can be read by touch. Don't fake the cursor with `Window::set_cursor_position`: it moves the real mouse on native and fails with a logged error in browsers.
- `TouchGesture`: `tap` (this frame's left press came from a finger), `still_at` (where the one finger down landed, once it has stayed `HOLD_SECONDS`), `drag` (one finger past `TAP_SLOP`), `drag_start` (where it landed), `pinch`, `twist`, `tilt` (only what both fingers share in moving up or down, so pinches and twists barely tilt), `centre`, `first_finger` (where the contact's first finger is while it stays down, whoever has joined it), `other_finger_tapped` (another finger touched and lifted while it did). Once a second finger has joined, the last one left neither drags nor taps until all are lifted. Three or more fingers do nothing.
- The camera ignores `drag`: one finger on the main view only taps. `minimap` takes the whole gesture while the full map is open.
- `index.html` sets `touch-action: none` and stops Safari's `gesturestart`, or the browser would take pinches for itself and cancel the touches.
- Not yet tuned on a device: `TAP_SLOP` and how sensitive two-finger gestures feel. There is no dead zone before twisting or tilting starts.
