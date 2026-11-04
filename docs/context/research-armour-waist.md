# Research: armour for the waist

Done 2026-11-03 by the Waist chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/LKs2gxs4ApgK8s7qgML3UD) and answered yes to all six questions; the answers are below and in `decisions.md`. Code: `crates/shared/src/armour/waist/`, `crates/client/src/item_art/armour/waist/`.

Evidence: **S** sourced (Wikipedia quote), **W** worked out, **E** estimate or general knowledge.

## Sources

84 Wikipedia articles (the parent chat's plus the short waist ones: Fauld, Tasset, Culet, Codpiece, Pteruges, Cingulum militare, Bases, Plackart, Kastenbrust, Ō-yoroi, Japanese armour, Linothorax, Dendra panoply, Mail and plate armour), every sentence mentioning a waist piece pulled out by script and read directly; no local model was needed, as the articles give names and forms, not measurements. Tonlet, Culet (armour), Mitra and Kremasmata have no English article. Ngram 2000-2019, per million: tasset 0.058, fauld 0.009, culet 0.006 (mostly gems), pteruges 0.004, kusazuri 0.001, mail skirt 0.0009, tonlet 0.0004 (codpiece and cingulum swamped by other meanings). Wikimedia Commons refused picture searches (rate limits), so the review page drew silhouettes on the body frame instead.

- Tassets (S): separate plates hanging from the breastplate or fauld over the upper thighs; 16th-century lames on sliding rivets (almain rivets); from the 16th century sometimes joined to the cuisses down to the poleyn.
- Fauld (S): covers the lower stomach, hips and groin, below a cuirass that ends at the waist; a full suit had "a fauld, tassets and a culet, a mail skirt".
- Tonlet (S): the Kastenbrust harness's plate skirt. Steel bases copied pleated cloth skirts, to the knee, for foot combat, with a detachable rear for riding.
- Pteruges (S): leather or layered linen strips hanging from the waist over hips and thighs, in one row of long strips or two or more shorter overlapping rows. Fringed leather at a muscle cuirass's lower edge; under the lorica hamata.
- Cingulum militare (S): a belt with metal fittings and an apron of strips with discs at their ends, a badge of status.
- Kusazuri (S): iron or leather panels hanging from the dō; the ō-yoroi had four, the dō-maru and haramaki usually seven.
- Brigandine skirts were split to the waist for riding (S). Persian mail and plate had a skirt of long horizontal plates (S). Laminar or lamellar loin-guards on the steppe (S).
- Dendra panoply (S): three pairs of curved plates hang from the waist over the groin and thighs.
- Greenwich (S): knee-length cuirassier tassets; very wide barrier tassets; blued or gilt bands down tassets. Maximilian (S): knee-long fluted tassets.
- Codpieces were part of some 16th-century full suits (S); Korean armour had groin protection (S).

## What was made

Two item kinds, named, sized and weighed by their form: Waist Armour (Outer on Waist) and Cuirassier Tassets (Outer on Waist and Legs). Rolled in order: style (cuirassier tassets: European or fantasy), a construction from the style's own list (never mirrors), the shape of piece (war girdle, fauld, tassets, hip panels, pteruges, skirt, mail breeches), length (band, hip, mid-thigh, knee), hoops, lames or rows, culet hoops, how many tassets or panels hang (2 or 4 tassets, the Dendra's three pairs, 4 to 8 kusazuri, 2 or 4 flaps), front only (the Roman apron; the mitra's plate on a girdle), where a skirt is split, mail beneath plate, a plain groin plate, studs, material, finish, cloth or lacing dye, brass trim. Specific names when they fit: Tonlet, Steel Bases, Almain Tassets, Cuirassier Tassets, Dendra Hip Plates, Kusazuri, Pteruges, Cingulum Apron, War Kilt (fantasy), Kremasmata, Mitra, Italic Belt; otherwise by shape and construction (Fauld, Tassets, Mail Skirt, Lamellar Hip Panels, Brigandine Skirt, War Girdle, Mail Breeches, ...), with Studded and an unusual material first.

Lengths on the frame: top `waist + 30` under the torso's hem; band `waist - 90`, hip the crotch, thigh `hip - (hip - knee) × 11/20`, knee `knee + 20`.

Weights (W): area × weight per area × density, calibrated to a mail skirt to mid-thigh of about 3 kg and a steel tonlet of about 5.5 kg; a four-hoop fauld about 1 kg, tassets to mid-thigh with their fauld about 1.5 kg, cuirassier tassets about 3 kg, leather pteruges about 1 to 1.5 kg.

## Owner's answers

1. Codpieces: only an optional plain, low groin plate on plate faulds, tassets and skirts; never a codpiece's shape.
2. The Roman belt apron is kept, as front-only pteruges.
3. Mail under plate: both. Plate and laminar faulds and tassets may have a mail skirt beneath (one item), and mail skirts are items of their own.
4. Cuirassier tassets are a Waist item that also takes Legs, the only one.
5. Regional looks are styles in the one set-wide list, each construction allowing some.
6. Weight grows with the square of size.

Also: every piece may take up to three gem sockets later; nothing here models them.

## Edges with other locations

Agreed with Torso: the torso's hem is at the waist with a flange 20 mm below; waist pieces start 30 mm above the waist, under it, at the shared 15 mm off the skin. Agreed with Legs: anything hung from the waist is Waist however far down it reaches, anything strapped to the thigh (cuisses, haidate) is Legs; hanging tassets stand 35 mm off the thigh over the cuisses (within 14 mm); cuirassier tassets end on the legs' own knee cop (`add_knee`). A coat's skirt (hauberk, lamellar coat, brigandine) is drawn in a set as the torso piece and a matching waist skirt.
