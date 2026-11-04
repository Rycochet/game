# Research: armour for the feet

Done 2026-11-03 by the Feet chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/39ryiZaL5JFzyF1qCuBvhS) and accepted all six suggestions; the answers are below and in `decisions.md`. Code: `crates/shared/src/armour/feet/`, `crates/client/src/item_art/armour/feet/`.

Evidence: **S** sourced (Wikipedia quote or a museum label), **W** worked out, **E** estimate or general knowledge.

## Sources

Wikipedia covers foot armour thinly: one Sabaton article, and single lines in Japanese armour, Poulaine, Duckbill shoe, Scale armour, Chain mail and Kusari. There are no articles for kogake, Ottoman, Mughal or Byzantine armour, and foot armour outside Europe and Japan seems rare. The paragraphs were few enough to read by hand, so no local model was used. Ngram 2000-2019, per million: sabaton(s) 0.0043, solleret(s) 0.0022, armoured/armored boots 0.0023, mail shoes 0.0001, kogake 0. Pictures came from Wikimedia Commons (Bata Shoe Museum, Higgins Armory, Walters, the Met, British Museum, Samuraiantiqueworld) and are credited on the review page.

- Sabaton (S): a sabaton or solleret covers the foot. 14th-15th century ones end in a tapered point well past the toes, following the shoes; late 15th-early 16th century ones follow the duckbill shoe, ending at the toes but much wider. Riveted iron lames, generally over the top of the foot only. Mail of the chausses once covered the foot, later ending at the ankle over or under the sabaton. The Beauchamp effigy: a toe cap, four lames, a foot plate and ankle plate, a hinged heel cap, buckled straps; spurs probably fixed to the heel cap. Solleret is a modern word (1826).
- Poulaine (S): sabatons were often in the poulaine style and could hinder walking; at Sempach (1386) dismounted knights cut off the tips. Maximilian I's surviving sabatons have extremely long ends for riding, detachable for fighting on foot, the catches over the big toe.
- Duckbill shoe (S): also bear-paw or cow's mouth; duckbills rounded, cowsmouth widening abruptly at the toes, bearsclaw with slashes parallel to the toes; up to a foot wide.
- Maximilian armour (S): squared sabatons, fluted. Greenwich armour (S): very wide sabatons; mail shoes in a garniture's drawing.
- Scale armour (S): evidence for scale sabatons. Splint armour (S): effigies show discs or splints on sabatons.
- Japanese armour, Kusari (S): kogake, armoured tabi covering the top of the foot; kusari tabi socks of mail.
- Higgins Armory label, sabaton for the left foot, Germany about 1490 (S): weighs 7 oz (about 200 g); laced to the shoe through paired holes, a strap under the foot, a knob and stud at the toe for a toe extension plate.
- British Museum (S, picture and label): bronze foot guards with modelled toes, Greek or Italian, about 520-480 BC.

## What was planned and made

- Pieces: sabatons (toe cap, lames, ankle plate, maybe a heel cap or a cuff over the ankle), mail shoes, foot guards. One item covers both feet.
- Toes: pointed, round, duck-bill, bear-paw, cow-mouth (S) and a fantasy claw; the style chooses them (Gothic pointed, Milanese round, Maximilian bear-paw or cow-mouth, Greenwich duck-bill or cow-mouth, Kastenbrust pointed or round, Elven short points curling up, Dwarven cow-mouth or claws).
- Points 20-130 mm past the toes; detachable ones 80-250 mm (owner).
- Lames: four on the effigy (S); 6-10 Gothic, 5-10 other pointed, 4-7 others, 3-5 leather and Dwarven (E, from pictures).
- Constructions: plate, leather, scale and splint sabatons; mail and ring mail shoes; plate, leather, plates in cloth and mail-and-plate foot guards. Chinese, steppe, Byzantine and Indo-Persian styles, which have no foot armour of their own, make mail shoes and leather foot guards (E).
- Weights (W, checked against the 200 g sabaton): plain plate pair about 430 g, with heel caps about 730 g; leather about 300; mail shoes about 700; kogake 200-400.

## Gaps and decisions

- Measurements beyond the one label: worked out from area × per-area weight × density, as the other locations.
- Non-European foot armour other than Japanese: none found; regional styles get mail shoes and leather guards.
- Spurs: left out for now (owner, via the Footwear research), so foot armour has the Feet Outer layer to itself.
- Episphyria (bronze ankle guards): given to the Legs work (owner).
