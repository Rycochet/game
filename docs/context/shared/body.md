# shared: body

Where things are on a standing body, in millimetres: the frame every worn thing's model is built on, so one model makes both its own picture and a character wearing it. Not rules for wearing (`creature`, `wear`).

## Files

- `Hair { style: HairStyle, beard: Beard }`, `Hair::NONE`: hair to draw on a head (`item_art`).
- `crates/shared/src/body.rs`: `Frame` (the figures for each body shape: `HUMAN`, the person every frame is measured against, and `GIANT`; the first peoples' come from their proportions), `Frame::of`, `Frame::joint`, `Joint`, `Side`. Tests.
- `crates/shared/src/body/hair.rs`: `HairStyle` (Bald, Short, Long, TiedBack, Braids, Topknot, Crest, Curly), `Beard` (None, Short, Full, Long, Braided, Forked), each with `name`, and `Hair { style, beard }` (`NONE`): the shapes hair is drawn in. Which a creature has is a look (`sheet`).
- `crates/shared/src/body/parts.rs`: `Parts` (what a body has beyond a person's: `Snout`, ears, `Horns`, tusks, tail, `Feet`, `Wings`, a mane and tail hair (a horse's mane and long tail, a cow's switch; a centaur keeps its horse's tail hair); shapes as kinds, sizes as millimetres) and `NONE`.
- `crates/shared/src/body/bipeds.rs`: `Biped` (a name, usual height, proportions and parts), `FIRST_BIPEDS` (the first peoples from the research, each a `CreatureKind` of the same name), `Biped::of(kind)`, `Parts::of(kind)`.
- `crates/shared/src/body/layout.rs`: `Layout`, any body's layout: `Spine` (from the hips' middle to the neck base, `pitch` 0 upright to 90 level, `Station`s at hips, waist, chest), `Neck`, `Head`, up to `MAX_LIMBS` `Limb` slots (`Role`, `Girdle`, side, up to `MAX_BONES` `Bone`s, foot) and `Parts`; `Layout::of`, `limb`, `root`, `joints`, `Spine::at`. Tests in `layout/tests.rs`.
- `crates/shared/src/body/beasts.rs`: `Beast` (a name, a `Plan` and `Parts`), `Fourfoot` and `Bird` (a few measurements in millimetres), `BEASTS` (horse, wolf, cattle, chicken), `Beast::of(kind)`, `Beast::layout`. In `beasts/build.rs`: layouts from them: a level back over four legs, or a bird's leaning back over two, legs as shares of the height bent at each joint.
- `crates/shared/src/body/biped_view.rs`: `Layout::biped(frame, parts)` and `Layout::frame()`: a `Frame` is the view of an upright layout with two arms and two legs.
- `crates/shared/src/body/taurs.rs`: `Taur` (a name, the `Lower` body: one of `BEASTS` by name or an `Arthropod`; the person's `Proportions` above; `Parts`), `TAURS` (the centaur on the horse, the drider on a spider, the scorpion man on a scorpion), `Taur::of(kind)`, `Taur::layout`; on `Layout`: `upper_body` (the upper body as a layout of its own, in place, arms only) and `worn_frame` (the frame worn things are built on and how far to move them).
- `crates/shared/src/body/taurs/arthropod.rs`: `Arthropod` (height, front body's length and halves, legs' reach and thickness) and its layout: a level front body with four pairs of `JointedLeg`s spread along it (`Girdle::Along`), each rising to its knee, falling out and down to the ground.
- `crates/shared/src/body/blend.rs`: `Layout::blend(a, b, t)`, one body turning into another (`t` thousandths of the way, `BLEND_WHOLE` = 1000); spine, neck, head and a taur's upper body (`grown` out of the back when the other has none). In `blend/`: `limbs.rs` (pairing on each side by role, then what each becomes, then slot order; bones, ways and girdles blended; limbs with no partner from or to a point), `parts.rs` (sizes blended, snout and horn kinds swapped at nothing halfway, feet, wings and sting at halfway; `scaled`), `frame.rs` (`zip`, `scaled`: frames number by number, for upper bodies). Tests in `blend/tests.rs`.
- `crates/shared/src/body/scaled.rs`: `Layout::scaled(share)` (the same body this many thousandths of its size, on the same ground) and `Layout::at_size(kind)` (a biped's layout scaled from a person's height to its kind's usual one; anything else as it is). Tests.
- `crates/shared/src/body/trig.rs`: sine and cosine by whole degrees from a table, in ten-thousandths, so tilted points agree on both sides.
- `crates/shared/src/body/proportions.rs`: `Proportions` (heads tall, breadth, limbs, arms, legs, beside a person's) and `PERSON`; `Proportions::frame` works a whole frame out from them. Research: `docs/context/research-creatures.md`.

## Public API

- `has_hands(kind)` (`body/limbs.rs`): arms (not wings), or a person's upper body on a beast's; a kind whose body isn't drawn has none. Mobs with hands open doors.

- `Layout::of(kind) -> Option<Layout>`: kinds drawn so far: on two legs, and the horse and chicken from `BEASTS`; and taurs (centaur, drider, scorpion man) from `TAURS`. The human's (and beggar's) is built from `PERSON.frame()`, the giant's from `GIANT`, the first peoples' from their `Biped`, with `Parts::of(kind)`.
- `Layout { spine, neck, head, limbs, parts, upper }`; `Upper { frame, at }`: a person's upper body (its frame, from the hips up) standing with the middle of its hips at `at` on the lower body; a taur's lower body has no neck or head (length and head 0), the upper body's being its own. `Layout::worn_frame() -> Option<(Frame, [i32; 3])>`: a biped's own frame unmoved, or a taur's upper frame and its offset. `Layout::upper_body()`. `Layout::limb(role, side)`, `limbs()`, `chin()` (the neck's top, by its tilt), `root(limb)` (where it hangs from), `joints(limb)` (root, then each bone's end); `Spine::at(along)`. `Bone { span, way, radius, end_radius }`: its end is `way` (mm per metre, x outwards) × `span` / 1000 on. `Role::{Arm, WingArm, Leg, Foreleg, Hindleg, JointedLeg}`, `Girdle::{Hips, Shoulders, Along(mm)}`.
- `Layout::blend(&a, &b, t) -> Layout`, `BLEND_WHOLE`: for shapeshifting; each end is exactly its own body. Blend layouts from `Layout::at_size(kind)`, so sizes change too. `Layout::scaled(share)`.
- `Layout::biped(&Frame, Parts)`, `Layout::frame() -> Option<Frame>` (upright, with arms or wing-arms and legs).
- `Frame::of(kind) -> Option<Frame>`: `Layout::of(kind)?.frame()`.
- `Proportions { heads, breadth, limbs, arms, legs }`, `PERSON`, `Proportions::frame()`: `heads` in tenths (a person is 80), the rest in thousandths of a person's.
- `Frame` fields: heights of the main levels from the ground, half widths and depths, limb radii, limb lengths, `arm_spread`.
- `Frame::usual_height(kind)`: how tall a kind on two legs usually stands, in millimetres (its frame is always a person's height).
- `Parts { snout, snout_length, ears, horns, horn_length, tusks, tail, sting, abdomen, feet, wings }` (`abdomen`: a spider's or scorpion's rear body's length; `sting`: the tail curls up over the back), `NONE`, `Snout::{Flat, Muzzle, Long, Beak}`, `Horns::{None, Back, Bull}`, `Feet::{Plantigrade, Digitigrade, Hooves, Talons}`, `Wings::{None, Arms, Folded}`.
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
- A taur's upper body is at a person's scale, as a biped's frame is, standing on its beast at the beast's own size: a centaur is a person on a 1.58 m horse, its hips at the withers less a third of the barrel, a third of the barrel in front of the shoulder joints. A frame is only for an upright body without an upper one; armour on a taur goes by `worn_frame`.
- Blending (shapeshifting, owner 2026-11-04 and 2026-11-06): limbs are paired per side, so a blend never needs more slots than the side with more limbs; one with no partner hangs from its girdle and grows from nothing there, left out at size 0, as are bones a partner lacks (grown at the limb's end). A blended bone's way keeps the blended length of the two ways, so a leg swinging forwards doesn't shorten on the way. A taur's upper body grows about its hips, which keep their place over the blended body's shoulders.
