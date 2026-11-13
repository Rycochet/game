# Plan: more accurate creature models

Agreed with the owner 2026-11-10 to 2026-11-12, in 6 PRs. PR 1 (the measuring tool, `cargo xtask shapes`), PR 2 (animal trunks), PR 3 (animal heads, ears, necks, tails) and PR 4 (humanoid trunks) are built. Each PR keeps this file up to date.

## Goal

The humanoids are recognisable; the animals (wolf, cattle, horse, chicken) are not recognisable as their exact kind. Make every creature's shape more accurate, and keep everything that is worn fitting.

## Why they look wrong now

- The trunk (`crates/client/src/item_art/body/trunk.rs`) is 6 oval rings centred on a straight spine, from the 3 `Station`s in `shared::body::Spine`. Back and belly are the same distance from the spine, so every animal is a tube: no deep chest, tucked belly, boxy cow.
- Heads (`body/features.rs`) are an oval with a cone for a snout; ears are one cone; tails differ only by length and hair.
- One colour per animal: no markings.

## The approach

- The skeleton stays: `shared::body::Layout` (spine, limbs, parts) and `Frame` keep their figures, so wearing rules and the ~290 places armour reads `Frame` don't change.
- Add an outline: a fixed number of rings along the spine (`shared::body::RINGS`, 10, settled in PR 2: `Spine::rings`, each a `Ring`), each a half width, how far the front (belly) is from the spine and how far the back is. Whole millimetres, in shared, as barding and armour weights will go by them (determinism rule).
- The same number of rings on every body, so `Layout::blend` (shapeshifting) blends ring by ring.
- Numbers are measured from MakeHuman meshes by an offline tool. The game never loads a mesh and gains no dependency.
- Each animal keeps our own size (withers 740 mm wolf, 1350 mm cattle, 1580 mm horse); only proportions come from the meshes, whose size is arbitrary.
- Creature models on the maps (`crates/client/src/item_art/model.rs`, free positions PR 8, merged) are built from the same triangles as the pictures, each vertex tagged by body part: keep the tagging right as shapes change.

## MakeHuman sources

- Site: https://static.makehumancommunity.org/assets/index.html (asset packs under `/assets/assetpacks/`). The old asset repository links (`www.makehumancommunity.org/node/...`) were dead on 2026-11-11 after a PHP exploit; the owner expects them back.
- Base mesh `base.obj` (hm08): CC0. From `https://raw.githubusercontent.com/makehumancommunity/makehuman/master/makehuman/data/3dobjs/base.obj`. 19158 vertices; the first 13380 are the skin, the rest helpers and joints. Units are decimetres; y up, z forwards, x across.
- A target is text: `<vertex index> <dx> <dy> <dz>` per line, `#` comments. Applying it is adding the offsets to the base vertices.
- `animal04` (CC-BY, JALdMIC): full four-legged bodies `jaldmic_vaca_cow`, `jaldmic_perro_dog`, `jaldmic_horse`, `jaldmic_donkey` (and a cartoon bear to skip). `https://files2.makehumancommunity.org/asset_packs/animal04/animal04_ccby.zip` (mirror: `files.` in place of `files2.`).
- `animal03` (CC-BY, JALdMIC): heads on a person's shoulders: `jaldmic_wolf_headv3`, `jaldmic_vaca_cow_head`, `jaldmic_perro_dog_head`, bear, cat, lion, tiger, rabbit, squirrel (and cartoon ones to skip). `https://files2.makehumancommunity.org/asset_packs/animal03/animal03_ccby.zip`.
- `animal02` (CC-BY, JALdMIC): ears (canine, feline, rabbit, dropped) and mouths (canine, feline, reptile, shark). `animal01` (CC0): faun face, pig nose, donkey and horse heads, cat ears.
- The files in use are in `spare_assets/makehuman/` (PR 1): the base mesh; the cow, dog and horse bodies; the wolf, cow and dog heads; pointed and dropped ears, a dog's and a reptile's muzzle. Its `README.md` has each file's source, author, licence and download date. Another target is fetched from its pack and added there.
- The meshes have no tails or horns (the human mesh has neither) and there is no chicken: ours stay as drawn.
- MakeHuman's code is AGPL (MPFB's GPL): never copy it. Reading their file formats with our own code is fine.

## The owner's decisions

1. Commit the few source files we use to `spare_assets/makehuman/` (about 5 MB), with a README giving each file's source link, author, licence and download date, so they can be refreshed when the site is back.
2. The tool is Rust: `cargo xtask shapes`, in `xtask/src/shapes/`, no new dependencies (as `cargo xtask icons` works). No Python.
3. Markings stay in this plan as PR 6, kept simple; more detail is a later task.
4. Animals first, as the most dramatic improvement.
5. The base human mesh is the reference for most of the humanoids too.
6. Credits in `docs/credits.md`: JALdMIC (CC-BY, required) and MakeHuman (CC0, as a courtesy).

## The PRs

| PR | What | Needs | Kind |
| --- | --- | --- | --- |
| 1 | Measuring tool and source files (built) | | blocking |
| 2 | Trunk outlines for the four-legged animals (built) | 1 | blocking |
| 3 | Animal heads, ears, necks, tails (built) | 2 | independent |
| 4 | Humanoid trunk and limbs from the base human (built) | 2 | independent |
| 5 | Beast peoples' heads | 3 | waits for 3 |
| 6 | Markings | 2 | independent |

### PR 1: measuring tool (built)
- `cargo xtask shapes [rings=10]` (`xtask/src/shapes.rs`, `shapes/`): applies each beast's target to the base mesh, stands it level, cuts the skin with planes and prints tables. Our own readers, no new dependencies. `spare_assets/makehuman/` with the files and README (decision 1), `docs/credits.md`.
- Nothing changed in game. No figures are committed yet, so there is no check in `cargo xtask test`: PR 2 adds one, like the icons check, that the committed figures still match the source files.

What it prints for each of the cow, dog and horse, all in thousandths of the height at the withers (so a figure × our animal's withers / 1000 is millimetres):
- The hip joints' height, and how far in front of them the shoulder joints are (`Fourfoot::body`).
- Trunk rings, evenly from the rump to the neck's base (a third of the chest's half depth in front of the shoulder joints, as `beasts/build.rs` places it): how far along from the hip joints (the rump is behind them, so the first are negative), half width, and the belly's and back's heights from the ground.
- The left foreleg and hind leg every 50 up from the ground while they stand clear of the trunk: half thickness side to side and front to back, and where the leg's middle is (out from the middle line, along from the hip joints), which shows how the leg bends.

How it measures, and what to know when using the figures:
- A cut is the outlines where a plane meets the skin's faces (`shapes/cut.rs`). A ring (`shapes/ring.rs`) is the outline crossing the body's middle line: belly and back are read on the middle line, and the half width only between them, so where a leg joins the trunk the leg below the belly is left out, but the shoulder and thigh muscle beside the trunk is counted (the dog's rings over its hind legs are wider than its waist for this reason).
- "Withers" is the top of the back over the forefeet (the `l-hand` joint: the animals are the person reshaped, so forelegs are arms and hind legs legs). The horse's neck rises from there, so its last two rings are already climbing into the neck.
- The meshes stand with their forefeet about a twentieth of their height above their hind feet; `shapes/level.rs` slides them level first.
- The first ring is the rump's tip (next to no width), and the last is at the neck's base, where the underside is already the neck's.
- A leg's top row or two, where it begins to join the trunk, can take in part of the trunk and read large.
- It checks out: the dog's belly rises behind the ribs (about 520 at the chest to 770 at the loin), the cow's barely (about 390 to 500), the horse's a little (about 500 to 590).
- Room left for later PRs: `ring()` takes any plane facing along the body's middle (`Plane::facing`), so a neck or head (PR 3) is rings along its own line, and an upright person (PR 4) rings up the spine, where "back" is backwards. The head targets are a person's body with an animal's head: measure them from the `neck` and `head` joints. Each new kind of measuring gets its own file beside `fourfoot.rs`, and its table in `table.rs`.

### PR 2: animal trunks (built)
- Shared: `Spine::rings`, `RINGS` (10) `Ring`s (`along`, `half`, `belly`, `back`; `body/rings.rs`). The tool's tables are copied unchanged into `body/measured.rs` (`MEASURED`: `COW`, `DOG`, `HORSE`, in thousandths) and `Measured::rings` scales them to our beasts: along the spine by our hips-to-shoulders length over the measured one, everything else by our withers, belly and back from our own spine (a ring's `belly` is negative when it is wholly above a level spine, as the rump's tip). Bodies with no measurements get `rings::from_stations`: their six old trunk levels with rings halfway between, which the client leaves out, so they are drawn exactly as before (checked pixel by pixel in Item Pictures).
- Client: `body/trunk.rs` lofts the rings, each an oval from its front to its back. The neck is drawn from the middle of the last ring (`trunk::neck_root`; the spine's end for other bodies, so unchanged), as the measured trunks already climb into the neck; the mane and the model's neck bone start there too. A level back's tail starts at the first ring.
- `blend.rs` mixes rings one by one; the existing test that each end of a blend is its own body covers them. `scaled.rs` scales them.
- Legs: `Fourfoot::fore` and `hind` (radius where each hangs from and at each joint) and `foot`, from the leg tables at our joints' heights, in place of one `leg` radius.
- `cargo xtask test` checks `MEASURED` still matches what the tool measures (`xtask/src/shapes/check.rs`); `ATTRIBUTION.md` credits JALdMIC for the figures.
- Left for PR 3: our necks start low on the spine while the measured trunks climb into theirs, so the wolf keeps a small rise where its neck meets its back; PR 3's necks should follow the last rings (done in PR 3).

### PR 3: animal heads and details (built)
- Shared: `Head::shape`, an optional `HeadShape` (`body/head_shape.rs`): `pitch` (nose down) and `HEAD_RINGS` (8) `Ring`s along the head's own line, from the neck's top (`Layout::chin`) forwards, `belly` below the line and `back` above; the snout is in the rings. The tool measures heads from the `head` joint to the nose tip (`xtask/src/shapes/head.rs`), each ring cut off at the jaw's underside (the head targets go on down a person's neck), its half width below the top quarter (leaving out the ears). Figures in `body/measured_heads.rs` (`WOLF_HEAD`, `COW_HEAD`, `DOG_HEAD`, `HORSE_HEAD`, in thousandths of the head's length; checked by `xtask test`), scaled by `MeasuredHead::shape(length, pitch)` to our own head length and carriage (`Fourfoot::head`). The hen's is drawn by hand in the same terms (`beasts.rs`, `CHICKEN_HEAD`).
- Heads without a shape (all bipeds) are drawn as before, so helmets are untouched; blends use `HeadShape::plain` (an oval and its snout as rings) for them, each end exactly its own head.
- `Layout::chin` starts from `Spine::neck_root` (the middle of the last ring; unchanged for bodies from stations), and the client lofts a shaped head's neck from the trunk's last ring to the head (`item_art/body/head.rs`), which also took away the wolf's rise at the neck join. Carriage and thickness by animal: a cow's head low (neck 75°, head 40° nose down), a wolf's level with its back, a horse's 45° nose down on an arched neck.
- `Parts`: `Ears::{Swept, Upright, Sideways, Dropped}` (`ear_shape`), `Tail::{Plain, Bushy, Thin, Tufted}` (`tail_shape`), `dewlap`, `udder`, `comb`. Wolf: upright ears, a bushy hanging tail, a thicker neck and a 25 mm coat on its trunk (`Fourfoot::coat`). Cattle: sideways ears under the horns, a thin tail with its switch, dewlap and udder (`item_art/body/extras.rs`). Horse: upright ears. Hen: comb and wattles.
- For PR 5: a beast people's head is a `HeadShape` too (`DOG_HEAD` is measured for it), placed on its frame's chin; helmets (`armour/head/face.rs`) then need checking against it, as the bipeds' oval is what they fit now.

### PR 4: humanoid trunks (built)
- The tool measures the base mesh standing upright (`xtask/src/shapes/upright.rs`): level rings from the crotch to the `neck` joint, front and back from a line straight up through the hip joints (the spine of our bipeds), and each limb bone's half thickness across and front to back at five cuts square to it. Figures in `body/measured_person.rs` (`PERSON_TRUNK`, `MeasuredUpright`, thousandths of the height; checked by `xtask test`).
- `MeasuredUpright::rings(frame)` puts them on any frame, so every people (and the giant, on its own figures) is shaped from its frame as before: the measured levels (crotch, hip joints, shoulders, neck) go to the frame's; widths are still the frame's (hips, waist, chest, shoulders, neck); depth is scaled to the frame's hips, waist and chest, split front and back as measured; the trunk leans back towards the top so the neck stays over the spine. The shape fades in from the hip joints (round, as before) to the waist, and the crotch ring (the hips', round) and the neck's base ring are as before, so legs, neck, head, helmets, tassets and leg armour's tops don't move (shaping the hips too sank tassets and chausses' seats into the belly). `Layout::biped` uses it; `rings::from_stations` remains for bodies with no measurements (birds, arthropods).
- What it gives a person: the chest about 160 mm in front of the spine and 60 behind (shoulder blades 80), the small of the back about 50 behind; the frame's chest was 110 each way.
- `item_art/armour/worn/tests.rs`: every sample item on every people stays outside the middle of the trunk's front and back (from a little above the hips to a little below the shoulders). Run against `master` too: there, as here, hidden parts (sleeve roots in the shoulders, tasset hinges under the fauld, leg pieces' seats) go inside the body by design; the strapped and tied breast bands' straps and knot and the barbe's pleats sank into the chest even before, and were fixed.
- Client: an upright ring is drawn as half an oval in front of the spine and half behind (a level one, a beast's, as one oval round its middle as before; between for a leaning spine), and `trunk_half` gives a `Girth { across, front, back }` in that shape; `trunk_depth` takes which side. Torso shells, waist `Hips` (and so underclothes), neck `Around`, shoulders and capes follow it; beards and hanging hair keep off the rings' front and back.
- Limbs were left as they were: the measured bones match the frames within a few millimetres (thigh 82 to 54 mm, shin 56 to 30, upper arm 45 to 38, forearm 38 to 23 against our 42 to 28).

### PR 5: beast peoples' heads
- Minotaur from the cow head, kobold and lizardfolk from the reptile mouth, others where a target fits.
- Helmets already allow for snouts (`crates/client/src/item_art/armour/head/face.rs`): check them against the new shapes.

### PR 6: markings
- New `Pattern` options (`crates/shared/src/sheet.rs`: pale belly, face mask, patches), chosen when a creature is made, as other looks are.
- Touches saved looks and what the server sends of a creature.

## Checking each PR
- Before and after screenshots of every affected kind at token size and large (Item Pictures, and the 3D models on a map).
- `cargo xtask test`, then `cargo xtask latest`.
- The owner signs off each animal's look; with away mode on, that is after the merge, and anything they dislike becomes a follow-up.
