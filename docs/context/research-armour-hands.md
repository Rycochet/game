# Research: armour for the hands

Done 2026-11-03 by the Hands chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/7aCLC3kHuSe9RbUuFe7vbf); their answers are recorded below and in `decisions.md`. Code: `crates/shared/src/armour/hands.rs`, `crates/client/src/item_art/armour/hands.rs`.

Evidence: **S** sourced (Wikipedia quote), **W** worked out, **E** estimate or general knowledge.

## Sources

Wikipedia has one short article on gauntlets, so shapes and weights rest mostly on museum pictures (Wikimedia Commons) and general knowledge.

- Gauntlet (glove) (S): mail mittens ("muffs") with a thumb pocket on 12th-century sleeves, some fully fingered; plate gauntlets from the early 14th century in mitten and fingered styles; the demi-gauntlet "only protects the back of the hand and the wrist", worn with mail or padded leather gloves.
- Chain mail, Hauberk (S): sleeves covering the hands "with a supple glove leather face on the palm of the hand, or even full mail gloves".
- Chausses (S): 3rd-century arena mosaics with mail or scale sleeves and armoured mittens.
- Rondel (S): small rondels on the back of gauntlets; the manifer (left-hand jousting gauntlet) used one. Kastenbrust armour (S): "gauntlets with a long cuff and a rectangular cutout".
- Kote (S): tekkō, "covers of metal plates for the back of the hand", tied to the kote.
- Tekkō (Okinawan weapon, S): the tekkō-kagi's "four iron nails like bear claws", the model for fantasy claws.
- Ngram 2000-2019: "gauntlet" 0.82; every specific plate name 0.0000-0.0003 (descriptions rather than words of their own, kept anyway); "mail mitten" inflated by "mufflers".

## Coverage

| Fingers | Covers | Finger count | Rings |
| --- | --- | --- | --- |
| Fingered | Hands, Fingers | must match | locked under it |
| Mitten | Hands, Fingers | any (owner) | locked under it |
| Open (demi-gauntlets, tekkō) | Hands | any | free |

## Names

Generic: Gauntlets, Mitten Gauntlets, Demi-gauntlets, Padded Gloves. Specific, when a roll fits, in this order: Clawed Gauntlets; Studded Leather Gauntlets; Spiked Gauntlets (metal); Locking Gauntlets; Kastenbrust Gauntlets; Gothic Gauntlets; Hourglass Gauntlets; Brigandine Gauntlets (plates in cloth); Mail Mittens / Mail Gloves; Ring Mail Mittens / Ring Mail Gloves; Scale Mittens / Scale Gloves; Tekkō (mail and plate, open); Mail-and-plate Gauntlets; Leather Gauntlets.

Left out: cestus, knuckledusters, bagh nakh, the tekkō as a weapon and the pata (weapons); the yugake (an archer's glove); single gauntlets such as the manifer and the bridle-hand gauntlet, as items are pairs (owner). Ring mail gloves came once `Construction::RingMail` existed (2026-11-04): fingered or mitten, a wrist cuff, flat rings sewn over the back and fingers of a leather glove; iron, steel or brass; (hand + cuff) × 3/4 at steel plus a 120 g glove, about 450 g a pair.

## Rolls and grid

Order: style, construction, fingers, cuff, knuckles, claws (fingered metal, 1 in 12), locking (plate mittens, 1 in 8), material, cloth, trim (brass on plate, 1 in 6). The grid (`HandsForm::allowed`):

| Construction | Kind | Fingers | Cuffs | Knuckles | Claws |
| --- | --- | --- | --- | --- | --- |
| Padded | Padded Gloves (Middle) | fingered | wrist | plain | no |
| Leather | Gauntlets | fingered, mitten | wrist, bell, long | plain, studs | no |
| Mail | Gauntlets | fingered, mitten | wrist | plain | no |
| Ring mail | Gauntlets | fingered, mitten | wrist | plain | no |
| Scale | Gauntlets | fingered, mitten | wrist, bell | plain, spikes | fingered |
| Plates in cloth | Gauntlets | fingered, mitten | wrist, hourglass | plain, spikes | no |
| Mail and plate | Gauntlets, Demi-gauntlets | fingered, open | wrist, bell (open: wrist) | plain, ridged | fingered |
| Plate | Gauntlets, Demi-gauntlets | fingered, mitten, open | hourglass, bell, pointed, long (open: bell) | plain, ridged, spikes, rondel | fingered |

Not on hands: lamellar, laminar, splint, mirror. Styles narrow plate (owner): Kastenbrust mitten and long; Gothic pointed; Maximilian fingered and bell; Milanese mostly mitten; Greenwich bell; Dwarven hourglass or bell.

Cuff reach on a human frame: wrist 50, hourglass 100, bell 110, pointed 140, long 200 mm, scaled by the frame's `wrist_length`; 28 mm out from the forearm, over a vambrace (agreed with the Arms chat).

## Weights (W)

A hand is about 400 cm² of skin, half on the back. Plate is 1 mm steel: about 160 g over the back and fingers, 180 g for an hourglass cuff, 60 g of leather glove and 30 g of straps: about 430 g a hand. Mail at about 7 kg/m², from a 10 kg shirt over 1.4 m². Checked against the parent's figures: a mail shirt at 9-11 kg and a full harness at 15-25 kg. Per pair, steel, ordinary size: plate hourglass 860 g, long cuff 1150 g, mitten about 820 g, demi 450 g, brigandine about 870 g, mail mittens about 660 g, scale about 600 g, mail-and-plate about 780 g, tekkō about 190 g, leather 350-500 g, padded 150 g; spikes +40, rondel +60, claws +80, locking +100. The main material scales by its density against steel (leather, linen for those).

## Materials (owner's decisions)

Metal: Steel, Bronze, Brass, Mithril, Adamantine. Plant: Coir (for other locations). Bone: Horn, Dragon Scale, Chitin. New Leather category: Leather, Hardened Leather, Buff Leather. New Textile category: Linen, Wool, Cotton, Silk. Lacquer, gilding, bluing and etching are finishes, not materials.

## Owner's answers (2026-11-03)

1. Mitten gauntlets fit any number of fingers.
2. Demi-gauntlets (Outer, hands only) and Padded Gloves (Middle) are their own kinds.
3. Single gauntlets are dropped; the rondel stays as a knuckle option.
4. Rings stay under gloves and gauntlets: in this setting rings are always against the skin.
5. A style narrows what plate may roll.
6. Claws and knuckle spikes will give damage bonuses in unarmed combat.
