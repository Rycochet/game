# shared: body

Where things are on a standing body, in millimetres: the frame every worn thing's model is built on, so one model makes both its own picture and a character wearing it. Not rules for wearing (`creature`, `wear`).

## Files

- `crates/shared/src/body.rs`: `Frame` (the figures for each body shape: `HUMAN`, the person every frame is measured against, then `GIANT`, `KOBOLD`), `Frame::of`, `Frame::joint`, `Joint`, `Side`. Tests.
- `crates/shared/src/body/bipeds.rs`: `Biped` (a name and its proportions) and `FIRST_BIPEDS`, the first set from the research, drawn on Item Pictures' Bodies page but not kinds yet.
- `crates/shared/src/body/proportions.rs`: `Proportions` (heads tall, breadth, limbs, arms, legs, beside a person's) and `PERSON`; `Proportions::frame` works a whole frame out from them. Research: `docs/context/research-creatures.md`.

## Public API

- `Frame::of(kind) -> Option<Frame>`: kinds with arms and legs (human, beggar, giant, kobold); none for centaurs and animals yet. The human's (and beggar's) is `PERSON.frame()`.
- `Proportions { heads, breadth, limbs, arms, legs }`, `PERSON`, `Proportions::frame()`: `heads` in tenths (a person is 80), the rest in thousandths of a person's.
- `Frame` fields: heights of the main levels from the ground, half widths and depths, limb radii, limb lengths, `arm_spread`.
- `Biped { name, proportions }`, `FIRST_BIPEDS`: halfling, dwarf, elf, orc, goblin, kobold, lizardfolk, minotaur and birdfolk.
- `Frame::joint(Joint) -> [i32; 3]`: x towards the body's left, y up, z forwards, origin on the ground between the feet.
- `Joint::{Crown, Skull, Chin, NeckBase, Shoulder, Elbow, Wrist, Fingertips, Chest, Waist, Hip, Knee, Ankle, Toes, Heel}`, `Side::{Left, Right}` (`BOTH`, `sign`).

## Depends on

- `shared::creature` (`CreatureKind`).

## Used by

- client `item_art` (the body's model), `debug_menu` (Item Pictures). Armour pieces will be built on it.

## Notes

- Every frame is at an ordinary person's scale (1750 mm tall), as blade lengths are: an item's size goes on top. So a kobold's frame is its proportions, not its height.
- Arms hang a little out from the sides (`arm_spread` per metre down), so sleeves and gauntlets show in a picture; the elbows are slightly bent, so the forearms hang nearer straight down and a little forwards (`forearm_spread`, `forearm_forward`), and the wrists and fingertips follow the forearm (owner, 2026-11-05, with the shoulders lowered a little).
- Whole millimetres, so weights worked out from it agree on both sides. Figures are first guesses from artists' proportions.
- Frames from `Proportions`: the head's height follows `heads`, the hips (and knees, ankles, feet) `legs`, and the shoulder, chest and waist levels spread over what lies between the neck and the hips, so every level stays in order. A person's proportions give `HUMAN` exactly (tested). Giant and kobold keep their own figures until their kinds move to proportions with the next bipeds.
- Coming (owner, 2026-11-04): a general body layout (a spine that can tilt, limb slots with roles, head, tail, wings, parts as numbers) that `Frame` becomes the biped view of, so other body plans and shapeshifting can be built on it. See the research file.
