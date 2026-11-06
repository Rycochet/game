# shared: body

Where things are on a standing body, in millimetres: the frame every worn thing's model is built on, so one model makes both its own picture and a character wearing it. Not rules for wearing (`creature`, `wear`).

## Files

- `crates/shared/src/body.rs`: `Frame` (the figures for each body shape: `HUMAN`, the person every frame is measured against, and `GIANT`; the first peoples' come from their proportions), `Frame::of`, `Frame::joint`, `Joint`, `Side`. Tests.
- `crates/shared/src/body/parts.rs`: `Parts` (what a body has beyond a person's: `Snout`, ears, `Horns`, tusks, tail, `Feet`, `Wings`; shapes as kinds, sizes as millimetres) and `NONE`.
- `crates/shared/src/body/bipeds.rs`: `Biped` (a name, usual height, proportions and parts), `FIRST_BIPEDS` (the first peoples from the research, each a `CreatureKind` of the same name), `Biped::of(kind)`, `Parts::of(kind)`.
- `crates/shared/src/body/layout.rs`: `Layout`, any body's layout: `Spine` (from the hips' middle to the neck base, `pitch` 0 upright to 90 level, `Station`s at hips, waist, chest), `Neck`, `Head`, up to `MAX_LIMBS` `Limb` slots (`Role`, `Girdle`, side, up to `MAX_BONES` `Bone`s, foot) and `Parts`; `Layout::of`, `limb`, `root`, `joints`, `Spine::at`. Tests in `layout/tests.rs`.
- `crates/shared/src/body/beasts.rs`: `Beast` (a name, a `Plan` and `Parts`), `Fourfoot` and `Bird` (a few measurements in millimetres), `BEASTS` (horse, wolf, cattle, chicken), `Beast::of(kind)`, `Beast::layout`. In `beasts/build.rs`: layouts from them: a level back over four legs, or a bird's leaning back over two, legs as shares of the height bent at each joint.
- `crates/shared/src/body/biped_view.rs`: `Layout::biped(frame, parts)` and `Layout::frame()`: a `Frame` is the view of an upright layout with two arms and two legs.
- `crates/shared/src/body/trig.rs`: sine and cosine by whole degrees from a table, in ten-thousandths, so tilted points agree on both sides.
- `crates/shared/src/body/proportions.rs`: `Proportions` (heads tall, breadth, limbs, arms, legs, beside a person's) and `PERSON`; `Proportions::frame` works a whole frame out from them. Research: `docs/context/research-creatures.md`.

## Public API

- `Layout::of(kind) -> Option<Layout>`: kinds drawn so far: on two legs, and the horse and chicken from `BEASTS`; none for centaurs yet. The human's (and beggar's) is built from `PERSON.frame()`, the giant's from `GIANT`, the first peoples' from their `Biped`, with `Parts::of(kind)`.
- `Layout { spine, neck, head, limbs, parts }`; `Layout::limb(role, side)`, `limbs()`, `chin()` (the neck's top, by its tilt), `root(limb)` (where it hangs from), `joints(limb)` (root, then each bone's end); `Spine::at(along)`. `Bone { span, way, radius, end_radius }`: its end is `way` (mm per metre, x outwards) × `span` / 1000 on. `Role::{Arm, WingArm, Leg, Foreleg, Hindleg, JointedLeg}`, `Girdle::{Hips, Shoulders}`.
- `Layout::biped(&Frame, Parts)`, `Layout::frame() -> Option<Frame>` (upright, with arms or wing-arms and legs).
- `Frame::of(kind) -> Option<Frame>`: `Layout::of(kind)?.frame()`.
- `Proportions { heads, breadth, limbs, arms, legs }`, `PERSON`, `Proportions::frame()`: `heads` in tenths (a person is 80), the rest in thousandths of a person's.
- `Frame` fields: heights of the main levels from the ground, half widths and depths, limb radii, limb lengths, `arm_spread`.
- `Frame::usual_height(kind)`: how tall a kind on two legs usually stands, in millimetres (its frame is always a person's height).
- `Parts { snout, snout_length, ears, horns, horn_length, tusks, tail, feet, wings }`, `NONE`, `Snout::{Flat, Muzzle, Long, Beak}`, `Horns::{None, Back, Bull}`, `Feet::{Plantigrade, Digitigrade, Hooves, Talons}`, `Wings::{None, Arms, Folded}`.
- `Beast { name, plan, parts }`, `Plan::{Fourfoot, Bird}`, `BEASTS`: each a `CreatureKind` of the same name (a test checks). `Parts::of(kind)` covers beasts too.
- `Biped { name, height, proportions, parts }`, `FIRST_BIPEDS`: halfling, dwarf, elf, orc, goblin, kobold, lizardfolk, minotaur and birdfolk. `Biped::of(kind)` finds a kind's by name; `Parts::of(kind)` is its parts, `NONE` for any other kind.
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
- Frames from `Proportions`: the head's height follows `heads`, the hips (and knees, ankles, feet) `legs`, and the shoulder, chest and waist levels spread over what lies between the neck and the hips, so every level stays in order. A person's proportions give `HUMAN` exactly (tested). The giant keeps its own figures; the kobold's frame comes from its proportions since the first peoples became kinds.
- The general layout (owner, 2026-11-04) is what other body plans, taurs and morphing build on: a spine that tilts, limb slots with roles, head, parts as numbers. A `Frame` is its two-legged view, so the ~290 places armour reads `Frame` stay as they are; a test checks every kind's frame through its layout is the one worked out before. Bipeds are still described by their frame figures (from `Proportions`) and their layout built from them; later body plans build layouts directly. Limbs hang down in the standing body's directions whatever the spine's tilt, so legs stand on the ground. Tail and wing-arms are still in `Parts`.
- Beasts' layouts are at their own usual size in millimetres, not a person's scale as bipeds' frames are; their pictures fill themselves. Measurements are first guesses from the research (owner to tweak, as with the bipeds). Necks lean forwards by `Neck::pitch`; a frame needs the spine and neck upright.
