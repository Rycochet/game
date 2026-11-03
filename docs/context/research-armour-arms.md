# Research: armour for the arms

Done 2026-11-03 by the Arms chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/Sy79oKap5F6v5YdHU3rY6i) and accepted all six recommendations; the answers are recorded below and in `decisions.md`. Code: `crates/shared/src/armour/arms/`, `crates/client/src/item_art/armour/arms/`.

Evidence: **S** sourced (Wikipedia quote), **W** worked out, **E** estimate or general knowledge.

## Sources

The parent chat's 199 armour articles, with every paragraph mentioning arms pulled out by script and read directly (the arm articles are short, so no local model was needed). Ngram 2000-2019, per million: bracer 0.049, vambrace 0.018, brassard 0.014 (mostly armbands), arm guard 0.013, manica 0.006, couter 0.0034, splinted vambrace 0.0028, rerebrace 0.0014, mail sleeve 0.0014, bazuband 0.0007, arm harness 0.0004; kote 0.025 is inflated by kendo; dastana, kolçak, naruchi and the kinds of gote too rare to measure.

- Vambrace (S): forearm guard, tube or gutter, steel or boiled leather, sometimes splinted; from the early 14th century. Rerebrace (S): upper arm, between the pauldron and the couter; splinted ones Byzantine. Couter (S): the elbow, a curved plate becoming articulated, common by the 1320s; "elbow cop". Rondel (S): the 14th-century globular couter gave way to winged couters in the 15th.
- Bracer (S): an archer's guard on the inside of the bow arm, usually part of the forearm; in role-playing games a general piece of armour. Ketoh (S): Navajo and Hopi bracers with silver and turquoise.
- Brassard (S): also the whole arm, vambrace, couter and rerebrace together.
- Manica (S): laminated iron or copper-alloy hoops on leather straps, Roman, Parthian and Persian, for cavalry, infantry and gladiators; Xenophon: shoulder to hand on the rein arm.
- Kote (S): padded cloth sleeves, often silk brocade, laced inside, covered in mail and small plates; the tekkō on the hand is tied on; han kote cover the forearm only; kusari kote are mail.
- Splint (S): strips on cloth or leather, mostly vambraces and greaves; also wood, leather and bone. Transitional armour (S): steel elbow cops over mail sleeves, leather rerebraces and vambraces.
- Mail sleeves (S): hauberk sleeves to the elbow or the wrist; oriental mail and plate had sleeves with vambraces. Scale sleeves (S): Persian, gladiators. Vietnamese and Dendra bronze forearm guards (S); Vietnamese ones cast as coils.
- Bazuband, dastana, kolçak, naruchi (E): the articles under those names are places; one long outer plate rising to a point over the elbow, a narrow inner plate, a mail hand-cover.

## What was made

One item kind, Arm Armour (Outer on Arms), covering both arms or, for an archer's bracer or a Classical manica, one. Rolled in order: style, a construction from the style's own list, reach, material, which arms, couter (cup or winged), closed or open tubes, spikes, studs, finish, cloth dye, brass trim. Names: specific when they fit (Kote, Han Kote, Kusari Gote, Manica/Manicae, Bazubands, Archer's Bracer), otherwise by reach and construction (Bracers, Vambraces, Couters, Rerebraces, Arm Harness, Splinted Arms, and Mail, Ring Mail, Scale, Lamellar, Padded, Brigandine or Armoured Sleeves), with Spiked, Studded and an unusual material first.

## Weights (W)

Area of one arm at the piece's distance from the skin: wrist 260 cm², forearm 720, elbow 400, upper arm 610; whole pieces a tenth more for overlaps, winged couters 15% more on the elbow. Per area: plate 870 mg/cm² (1 mm steel and straps), hardened leather 400, mail 750, mail and plate 800, splints 600 plus leather 300, scale, lamellar, laminar and plates in cloth 1000, padding 220; scaled by the material's density. A steel arm harness comes to about 3.5 kg, mail sleeves 2.9 kg, leather bracers 0.3 kg, one manica about 2 kg; checked against a full harness at 15 to 25 kg (S) and a mail shirt at 9 to 11 kg (S).

## Owner's answers (2026-11-03)

1. A manica or an archer's bracer may be on one arm, taking the whole Arms place: yes.
2. A full plate arm is "Arm Harness": yes.
3. Bracers are light forearm armour now, with an archery bonus once bows exist: yes.
4. Spikes yes; forearm blades later as weapons; gems only in sockets on the item's frame.
5. Couters on their own: yes.
6. Kote also take the Middle layer: yes.

From the parent's review, applied here: a piece that only half covers a location doesn't count, so sleeves always reach the wrist; style is shared across a set; mithril is Metal, dragon scale and chitin Bone; ring mail and studded leather stay.

## Gaps

- Arm piece weights and thicknesses: none in the articles; worked out above.
- Inner elbow lames (Greenwich) and mail goussets: not modelled; goussets belong to arming doublets.
- Brigandine sleeves with a leather cover, and mirror arm pieces: not made (mirror armour wore bazubands).
