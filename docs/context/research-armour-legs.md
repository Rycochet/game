# Research: armour for the legs

Done 2026-11-03 by the Legs chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/FxiC2zmWQofYnzsofXZdAj) and answered yes to all six questions; the answers are below and in `decisions.md`. Code: `crates/shared/src/armour/legs/`, `crates/client/src/item_art/armour/legs/`.

Evidence: **S** sourced (Wikipedia quote or museum record), **W** worked out, **E** estimate or general knowledge.

## Sources

The parent chat's 199 armour articles, with every sentence mentioning legs pulled out by script and read directly, plus the short leg articles (Cuisses, Chausses, Greave, Poleyn, Schynbalds, Splint armour, Sangu) and a few more (Byzantine army, Kyahan, Murmillo). No local model was needed. Ngram 2000-2019, per billion: greaves 84 (+ greave 13, jambes 20), chausses 16, cuisses 11 (+ cuisse 7), ocrea 4 (+ ocreae 2), leg armour 6, poleyns 1.6, suneate 0.5, haidate 0.5, knemides 0.6, schynbalds 0.1. Weight: Fitzwilliam Museum records and reproduction listings. The dizçek: museum and auction records (Higgins Armory collection at Worcester). Reference pictures from Wikimedia Commons are on the review page only.

- Cuisses (S): padded (gamboised) over mail chausses in the 12th–13th c., poleyns attached; brigandine or splinted leather; steel plate from about 1340, one plate from 1370. Continental ones open at the back, English closed. Greek perimeridia: thigh plates, mostly cavalry.
- Chausses (S): mail leggings above the knee, with feet until later; demi-chausses cover the front only; padded ones under mail. Poleyns from about 1230 and schynbalds from the mid-13th c. were strapped over them.
- Poleyn (S): 1230–1650; later articulated to cuisses and greaves; Gothic ones with a side wing.
- Schynbalds (S): one plate over the front and outside of the shin, not enclosing. Closed greaves (S): two plates hinged outside, buckled inside, felt padding, from about 1290. Maximilian fluting never on the greaves (S).
- Greek and Roman (S): bronze knemides, often only one; Roman greaves pressed from sheet and lined; episphyria (ankle guards) worn with greaves. Gladiators' fasciae padded under ocreae. Laminated leg manica (Gamla, Dura Europos); scale demi-chausses at Dura Europos; a Sarmatian splinted cuisse and greave with plate poleyns.
- Byzantine (S): the Strategikon's "iron or wooden greaves"; Avar splinted greaves.
- Japanese (S): suneate of iron or leather plates joined by hinges or mail on cloth, tied round the calf, with a stirrup guard; tsutsu-suneate three plates with large knee guards; shino-suneate splints with mail; kikko and karuta versions. Haidate: a cloth apron from the waist with armour over the thighs. Kyahan: cloth gaiters under suneate.
- Ottoman and Persian (S): the dizçek, columns of small plates over the thigh, a large knee plate, mail round it. A labelled Turkish diagram gives the name to the greave instead; the museum records' use is kept.
- Vietnamese (S): bronze greaves cast as coils of rings, with small bells.

## What was made

Three item kinds, named, sized and weighed by their form: Leg Armour (Outer on Legs), Chausses (footed mail, Outer on Legs and Feet), Padded Chausses (Middle on Legs, in place of trousers). Rolled in order: style, a construction from the style's own list, cover (knee, shin, thigh, thigh and knee, knee and shin, whole leg, ankle), material, closed or front only, knee (cop, winged, Japanese tall plate), reinforcements on European and fantasy mail (poleyns; poleyns and schynbalds; quilted thighs and poleyns), one leg (Classical greaves, 1 in 20), fantasy ornament (knee spikes, shin fins), studs, finish, cloth dye, brass trim. Specific names when they fit: Chausses, Footless Chausses, Demi-chausses, Reinforced Chausses, Padded Chausses, Gamboised Cuisses, Schynbalds, Suneate, Tsutsu-suneate, Haidate, Knemides (Knemis), Ocreae (Ocrea), Ring Greaves, Perimeridia, Episphyria, Leg Manicae, Dizçek; otherwise by cover and construction (Poleyns, Greaves, Demi-greaves, Cuisses, Leg Harness, Knee Guards, Thigh Guards, Leggings, ...), with Spiked, Bladed, Studded and an unusual material first. Ring greaves are Classical bronze, as the arms' coiled guards are, there being no Southeast Asian style.

Cut: trews, hose, braies, leggings, gaiters, puttees and kyahan (clothes); cuirassier boots (footwear); coat skirts and tassets of every length (Waist); horse leg bards (barding, later). No named Indian thigh or knee guard could be sourced.

## Edges with other locations

Agreed with Waist: cuisses start at the crotch (`hip - thigh`), rising to the belt outside; rigid leg pieces stand 15 mm off the skin, under Waist's tassets at 35 mm; Waist's lobster tassets may use the leg's poleyn. Agreed with Feet: the line is the ankle; greaves come down to it, flared over a sabaton's top; footed chausses draw their own foot for now (Feet offered a mail-foot builder for later); greave and sabaton are joined by sets.

## Weights (W)

Area of one leg all round: thigh 900 cm², knee 380, shin 1100, ankle 250, foot 350; front-only pieces 60%, whole legs a tenth more for overlaps, winged knees 15% more. Per area: plate 750 mg/cm², hardened leather 400, mail 750, mail and plate 800, splints 600 plus leather 300, scale, lamellar, laminar and plates in cloth 950, padding 220; scaled by the material's density. A steel leg harness comes to about 4 kg a pair, greaves 1.7, poleyns 0.65, footed mail chausses about 4.5, padded chausses about 1.2; checked against a surviving cuisse with poleyn of about 1.2 kg a leg (S) and reproduction plate legs of 2.8 to 3.9 kg (S).

## Owner's answers (2026-11-03)

1. Plates strapped over mail are reinforcements chausses roll, one item: yes.
2. Padded chausses are Middle, in place of trousers: yes.
3. A single greave as a rare variant: yes.
4. Japanese, Roman, Greek and Ottoman are styles too: yes (as Japanese, Classical and Indo-Persian in `armour::Style`).
5. Keep the dizçek and the ring greaves: yes.
6. Knee spikes and shin fins as rare fantasy ornaments: yes.

Later: the owner gave the Greek bronze ankle guards (episphyria) to Legs rather than Feet. Every piece will take up to 3 gem sockets, shown on the item's frame, not the model.
