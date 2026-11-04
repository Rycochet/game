# Research: armour for the torso

Done 2026-11-03 by the Torso chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/XZ45Ef31Rd6wEiTgFzXwgs, with 29 reference pictures from Wikimedia Commons, credited there); their answers are recorded below and in `decisions.md`. Code: `crates/shared/src/armour/torso/`, `crates/client/src/item_art/armour/torso/`.

Evidence: **S** sourced (Wikipedia quote), **W** worked out, **E** estimate or general knowledge.

## Sources

About 200 armour articles fetched by the parent chat, plus 22 more for the torso (most redirected to ones already held: Dingjia and Kuyak to Brigandine, Yushman and Bakhterets to Mail and plate armour, Aketon and Arming doublet to Gambeson). Measurement sentences were pulled by script and read directly; no local model was needed. Ngram 2000-2019 ranked the names.

- Breastplate (S): 1 to 2.5 mm thick. Plate armour (S): a full harness 15 to 25 kg. Munition armour (S): a cuirassier's 32 to 45 kg.
- Lorica hamata (S): rings about 5 mm inside, 7 mm outside; about 11 kg. Chain mail (S): rings 1.0 to 1.6 mm wire.
- Jack of plate (S): about 7.7 kg. Buff coat (S): leather 1.5 to 5.6 mm, whole coats 1.9 to 3.4 kg. Yanghai leather scale (S): 4 to 5 kg whole.
- Chinese armour (S): Tang lamellae 9 cm long, 1.3 to 2.6 cm wide, 2.2 to 2.8 mm thick; a Ming full set about 20 kg.
- Muscle cuirass (S): replaced the bell cuirass around 480 BC; "about 25 pounds", which looks to include more than the shell (to check). Peascod belly (S): the 16th-century padded belly that breastplates copied.
- Pictures (Commons): Kastenbrust (Konrad Witz), brigandines (Jakob von Ems drawings, Italian brigandine), Greek bell and muscle cuirasses (Met), tankō (Met), okegawa-dō and hara-ate, char-aina plate shapes, yushman, Tibetan lamellar (Met), linothorax on a vase and the Mars of Todi, Tlingit slat, hide and coin armour.
- Ngram (per million): breastplate 0.57, cuirass 0.18, hauberk 0.09, corslet 0.075, leather armour 0.056, mail shirt 0.046, backplate 0.035, studded leather 0.023, scale shirt 0.019, brigandine 0.015, gambeson 0.014, byrnie 0.013, cuir bouilli 0.012, haubergeon 0.011, mithril 0.010, buff coat 0.009, ring mail 0.007, peascod 0.006, lorica segmentata 0.004, linothorax 0.002, coat of plates 0.001. Museum terms scoring near zero (Kastenbrust, klivanion, yushman, tankō, the dō names) are kept.

## Names

Every piece is Outer on Torso (item kind `TorsoArmour`), stopping at the waist; longer coats are sets with a Waist piece. The Arming Vest is Middle (`ArmingVest`).

| Construction | Generic | Specific | Cells |
| --- | --- | --- | --- |
| Plate | Cuirass; Breastplate (no back) | Milanese, Gothic, Maximilian, Peascod, Munition, Muscle, Bell, Triple-disc, Elven, Dwarven Cuirass; Kastenbrust; Tankō; Kardiophylax (disc, no back) | 3 × 3, breastplate 2 × 2 |
| Leather | Leather Cuirass | Studded Leather, Buff Jerkin, Hide Armour (rawhide), Leather Muscle Cuirass | 3 × 3, soft 2 × 3 |
| Mail | Mail Shirt | Byrnie, Lorica Hamata, Kusari Katabira, Mithril Shirt | 2 × 2 |
| Ring mail | Ring Mail | | 2 × 3 |
| Padded | Padded Jack; Arming Vest | Linothorax, Myeonje Baegap, Ichcahuipilli, Coir Armour | 2 × 3, vest 2 × 2 |
| Scale | Scale Shirt | Lorica Squamata, Lorica Plumata (on mail), Horn Scale Shirt, Dragonscale Shirt, Coin Armour (brass) | 2 × 3 |
| Lamellar | Lamellar Cuirass | Dō-maru, Haramaki, Ō-yoroi Dō, Mountain Pattern Cuirass, Klivanion, Bone Lamellar | 2 × 3, boxed 3 × 3 |
| Laminar | Laminar Cuirass | Lorica Segmentata, Okegawa-dō, Banded Mail (over mail) | 3 × 3 |
| Plates in cloth | Brigandine | Coat of Plates, Jack of Plate, Leather Brigandine, Dingjia, Tatami Dō | 2 × 3 |
| Mail and plate | Plated Mail | Yushman, Bakhterets, Zirah Bagtar | 2 × 3 |
| Mirror | Mirror Armour | Char-aina | 2 × 3 |
| Splint | Splint Cuirass | Slat Armour (wood or bone) | 2 × 3 |
| Woven | Rattan Cuirass | | 3 × 3 |

Dropped while coding: Kuyak (the Steppe style makes no plates in cloth) and Baju Rantai (no Malay style).

## What is rolled

Style first (or a set's), then a construction it allows, material, the construction's own parts, then back, over mail, lining, finish, cover dye and brass trim. Plate's breast follows the style: White round or munition, Milanese globose with a half plackart, Gothic waisted with a pointed plackart and fanning ridges, Kastenbrust boxed, Maximilian fluted, Greenwich peascod, Classical muscle, bell or discs (bronze), Japanese tankō (iron), Elven leaf, Dwarven panels. Lance rests only on European plate. Mail rolls links (butted, riveted, riveted and solid, welded), weave (4-in-1, 6-in-1, Japanese in Japanese style) and rings (6, 9, 12 mm); padding rolls quilting (channels, diamonds, squares, laminated for Greek linen and Chinese-style cotton).

## Weights (W)

A human torso shell from waist to shoulders is about 0.31 m² (breast 0.17, back 0.14). Weight = area × thickness × density, by the material's density against the reference, plus fittings; by size squared for other sizes. Steel cuirass about 5.3 kg (breast 2.5 mm, back 1.5 mm), breastplate 3.6, munition 7, bronze muscle 4.4, hardened leather 1.9, buff jerkin 1.3, mail shirt with short sleeves 3.1 to 3.4 (about 8.5 kg/m², from the lorica hamata), padded jack 2.5 to 3, arming vest 1.2. A mail shirt under a rigid piece adds 3.1, a lining 0.9. Small plates overlap more: iron scale 3.5 to 4.2 (about 1.4 times mail per m²), lamellar 3.9 to 4.6 (13 kg/m², from a Ming set of about 20 kg), laminar 0.6 a lame, brigandine plates 3.5 to 3.8 plus 0.5 cover, plated mail 5.1 to 5.9, mirror armour 4 to 4.5 with its mail, wooden slats 1.1, rattan 1.5.

## Owner's answers

Mail under plate: yes, as part of the piece. An Arming Vest: yes. Dyes: yes, now, with finishes handled alongside. Breastplates alone: yes. Keep slat armour, ichcahuipilli, myeonje baegap and Kiribati coir armour. Add adamantine. Studded leather: both, a leather finish with less protection and a brigandine with a leather cover. From the parent: up to 3 gem sockets on every piece, not modelled.
