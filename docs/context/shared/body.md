# shared: body

Where things are on a standing body, in millimetres: the frame every worn thing's model is built on, so one model makes both its own picture and a character wearing it. Not rules for wearing (`creature`, `wear`).

## Files

- `crates/shared/src/body.rs`: `Frame` (the figures for each body shape: `HUMAN`, `GIANT`, `KOBOLD`), `Frame::of`, `Frame::joint`, `Joint`, `Side`. Tests.

## Public API

- `Frame::of(kind) -> Option<Frame>`: kinds with arms and legs (human, beggar, giant, kobold); none for centaurs and animals yet.
- `Frame` fields: heights of the main levels from the ground, half widths and depths, limb radii, limb lengths, `arm_spread`.
- `Frame::joint(Joint) -> [i32; 3]`: x towards the body's left, y up, z forwards, origin on the ground between the feet.
- `Joint::{Crown, Skull, Chin, NeckBase, Shoulder, Elbow, Wrist, Fingertips, Chest, Waist, Hip, Knee, Ankle, Toes, Heel}`, `Side::{Left, Right}` (`BOTH`, `sign`).

## Depends on

- `shared::creature` (`CreatureKind`).

## Used by

- client `item_art` (the body's model), `debug_menu` (Item Pictures). Armour pieces will be built on it.

## Notes

- Every frame is at an ordinary person's scale (1750 mm tall), as blade lengths are: an item's size goes on top. So a kobold's frame is its proportions, not its height.
- Arms hang a little out from the sides (`arm_spread` per metre down), so sleeves and gauntlets show in a picture.
- Whole millimetres, so weights worked out from it agree on both sides. Figures are first guesses from artists' proportions.
