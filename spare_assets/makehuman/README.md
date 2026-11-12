# MakeHuman meshes

Shapes that creatures' bodies are measured from by `cargo xtask shapes` (`xtask/src/shapes.rs`). Never shipped, and never read by the game: only figures measured from them go into the code (`docs/context/creature-shapes.md`).

The files are unchanged, as downloaded (line endings too: `.gitattributes` stops git changing them). They are plain text:

- `base.obj`: a Wavefront mesh. `v x y z` is a point, `g name` starts a group, `f a/b ...` is a face. The `body` group is the skin (the first 13380 of the 19158 points); `joint-...` groups are small cubes marking the joints; the rest are helpers (hair, skirt, teeth). Units are decimetres; y is up, z forwards, x across.
- `*.target`: how far each point moves to make another shape, a line each: `<point, counting from 0> <dx> <dy> <dz>`.

MakeHuman's programs are AGPL (and its Blender plugin GPL): none of their code is used or copied here. These files are assets, under the licences below.

## Sources

Asset packs are listed at <https://static.makehumancommunity.org/assets/index.html>. Each target's own page (`www.makehumancommunity.org/node/<number>`, from its pack's `packs/*.json`) was unreachable when these were downloaded, as the site's old asset repository was down.

| File | What | Author | Licence | Source | Downloaded |
| --- | --- | --- | --- | --- | --- |
| `base.obj` | The base mesh of a person (hm08) | The MakeHuman team (Data Collection AB, Joel Palmius, Jonas Hauquier) | CC0 | <https://raw.githubusercontent.com/makehumancommunity/makehuman/master/makehuman/data/3dobjs/base.obj> | 2026-11-11 |
| `animal04/jaldmic_vaca_cow.target` | A cow's whole body | JALdMIC | CC-BY | Pack `animal04` (below); page node/3543 | 2026-11-11 |
| `animal04/jaldmic_perro_dog.target` | A dog's whole body | JALdMIC | CC-BY | Pack `animal04`; page node/3539 | 2026-11-11 |
| `animal04/jaldmic_horse.target` | A horse's whole body | JALdMIC | CC-BY | Pack `animal04`; page node/3533 | 2026-11-11 |
| `animal03/jaldmic_wolf_headv3.target` | A wolf's head on a person | JALdMIC | CC-BY | Pack `animal03`; page node/3009 | 2026-11-11 |
| `animal03/jaldmic_vaca_cow_head.target` | A cow's head on a person | JALdMIC | CC-BY | Pack `animal03`; page node/3544 | 2026-11-11 |
| `animal03/jaldmic_perro_dog_head.target` | A dog's head on a person | JALdMIC | CC-BY | Pack `animal03`; page node/3540 | 2026-11-11 |
| `animal02/jaldmic_canine_ears_v2.target` | Pointed ears | JALdMIC | CC-BY | Pack `animal02`; page node/2926 | 2026-11-12 |
| `animal02/jaldmic_dropped_ears.target` | Dropped ears | JALdMIC | CC-BY | Pack `animal02`; page node/3127 | 2026-11-12 |
| `animal02/jaldmic_furrymouth_canine_v4.target` | A dog's muzzle | JALdMIC | CC-BY | Pack `animal02`; page node/2927 | 2026-11-12 |
| `animal02/jaldmic_furry_mouth_reptile.target` | A reptile's muzzle | JALdMIC | CC-BY | Pack `animal02`; page node/2931 | 2026-11-12 |

The packs, each a zip with its targets in `targets/animal/` (a mirror: `files.` in place of `files2.`):

- `animal02`: <https://files2.makehumancommunity.org/asset_packs/animal02/animal02_ccby.zip>
- `animal03`: <https://files2.makehumancommunity.org/asset_packs/animal03/animal03_ccby.zip>
- `animal04`: <https://files2.makehumancommunity.org/asset_packs/animal04/animal04_ccby.zip>

Licences are as each pack's `packs/*.json` and the base mesh's own header give them: "CC-BY" (Creative Commons Attribution; no version is named) and CC0. JALdMIC is credited in `docs/credits.md`, as CC-BY asks; MakeHuman as a courtesy.

## Adding one

Copy the file in unchanged, add its row here, and name it in `xtask/src/shapes.rs`. Only keep targets something is measured from.
