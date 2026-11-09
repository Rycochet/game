# Research: underclothing

What underclothes a generator can make for the Skin layer, medieval and earlier first, in the terms the 3D models are built from. Done in November 2026 by the method in `research-footwear.md`, with Claude through the API in place of local models. Wikipedia's undergarment, hosiery, nightwear, medieval and Byzantine clothing, corsetry, caps and shirt categories gave 581 usable titles. 52 garment articles were read with one extraction each, and every quote was checked against its article by script: 546 checked, 2 failed, 39 unquoted "none"s. 22 overviews of period dress (Anglo-Saxon, Byzantine, Roman, Greek, Egyptian, Japanese, Chinese, Korean, European fashion by century to 1600) were searched for underclothes without articles of their own: 275 named, all quotes checked. The API drafted 28 fantasy pieces on the historical forms. Pictures are from Wikimedia Commons, chosen by hand from contact sheets (links only, below). The code is `shared::underclothes` (see `shared/underclothes.md`); models come in the next change, in `item_art`.

Articles almost never give sizes or weights (3 of 52 gave a length), so weights are worked out from cloth areas: linen about 150 to 200 g/m², wool cloth 250 to 350, silk 50 to 80, cotton 120 to 150, felt 400 to 600.

## Owner's decisions (2026-11-09)

Before the research:
- Stays and corsets can be Skin; hose cover legs and feet, so they block socks; underclothes keep close to historical shapes, medieval and earlier first, and are "tucked in" as they are mostly hidden; night caps and sleepwear are Skin; what doesn't fit goes to the mid-layer research.

Answers to the review:
1. **Layering:** one Skin layer. Braies, loincloths and petticoats list only the Waist (they hang from it, as the rule for half-covered locations already allows), so braies with hose, a shirt with braies and a petticoat over stockings all fit. A biped's Waist takes Skin. Loincloth, braies and petticoat are one of three; a breast band can't go under a shirt (rare in the sources).
2. **Stays:** an option on the shirt itself (none, pair of bodies, stays: long shirts only, 16th century on), drawn over it, so they always sit over the chemise. No Stays kind. For later: a Middle piece could let the layer below change how it is drawn (a gown that supports stays).
3. **Names:** "Linen Coif" for the cap (the mail Coif is armour); "Hose", never "Chausses", for cloth legs.
4. **Footless and stirrup hose** are Legs only, so socks go with them.
5. **Tails, horns, wings and hooves:** no rule; the model cuts the opening for whoever wears it ("fantasy is better than reality here").
6. **Nightwear** can be worn at any time; it only adds choice. Wardrobe sets come later.
7. **Later pieces** (stays, smocks with standing collars, knitted stockings, the brimmed nightcap) kept, rarer.
8. **Regional names** (fundoshi, juban, dudou, sokgot, kaupinam, tabi) kept as specific names.
9. **Hair shirt** kept, with Haircloth; it wants an intrinsic effect once item effects exist.
10. **Materials:** Hemp, Ramie and Haircloth under Textile; lawn (fine linen) and net as kinds of cloth.
11. **Hard mode:** underclothes stay on at death (Skin, unpadded): wearing better ones is the player's choice.

## What the sources agree on

- **Men:** a linen shirt (hip to thigh, tucked into the braies), linen braies on a belt through the waistband (ankle-length early, knee by the 13th century, short by the 15th), wool hose cut on the bias and tied to the braies belt (separate to the thigh; joined to the waist with a codpiece from the 15th century), a coif tied under the chin.
- **Women:** a linen chemise or smock (calf to ankle) under the kirtle, hose or stockings to the knee gartered below it, a coif, hairnet or undercap with a wimple and veil. A breast band ("breast girdle", 14th century; linen bras found at Lengberg, 15th century) is attested but rare. No drawers until after 1500, Venice first.
- **Earlier:** Roman subucula, subligaculum (tied loincloth or shorts; a leather pair from London) and strophium or fascia; Greek perizoma and apodesmos; Egyptian linen cache-sexe and slit-leather loincloths; Anglo-Saxon linen shirts and drawers, linen or leather stockings, leg bindings; the Norse særk.
- **Elsewhere:** Japanese fundoshi, hadajuban, nagajuban and tabi (15th century); Chinese zhongyi and dudou; Korean sokgot layers (sokchima petticoats, pants under skirts); Indian kaupinam and langota; Mamluk undershirts and drawers.

## Kinds and properties

Nine kinds, all Skin. Every choice is rolled from what the earlier ones allow; the grids are in `shared::underclothes` (one file per garment). Periods: ancient, early, high and late medieval, 16th century on (later ones rarer).

| Kind | Locations | Cells | Grams (all materials) | Properties |
| ---- | --------- | ----- | ----- | ---------- |
| Shirt | Torso, Arms (sleeveless: Torso) | 2×2, knee and longer 2×3 | 100 to 1000 (stays add 350 to 450) | Length (hip, thigh, knee, calf, ankle), sleeves (none, short, elbow, wrist, full), neck (slit, round, square, gathered, collar, crossover), cut (gussets, gored, full), for the night, stays |
| Braies | Waist | 2×2, thigh 2×1 | 80 to 500 | Length (thigh, knee, calf, ankle), fit, waist (belt, drawstring), leg ends (open, tied at the calf, tucked into the hose) |
| Breast Band | Torso | 2×1 | 25 to 140 | Wound, twisted, wide wrap, tied, panel, strapped |
| Loincloth | Waist | 2×1 | 25 to 155 | Strip (twisted or not), apron, narrow (sewn or not), triangle, tied, briefs |
| Hose | Legs, Feet (footless and stirrup: Legs) | 2×2, joined 2×3 | 55 to 570 a pair | Feet, height (knee, thigh, joined at the waist), make (bias-cut, leather, knitted), leather sole, held by (ties, points, garters), colours (plain, striped, parti), codpiece |
| Socks | Feet | 1×1, calf and knee 1×2 | 25 to 260 a pair | Make (sewn, nålbound, felt, wrapped, knitted), toe (whole, split), height (foot, ankle, calf, knee), top |
| Petticoat | Waist | 2×2, very full to the ankle 2×3 | 170 to 2250 (quilted wool) | Length (knee, calf, ankle), fullness, make (plain, quilted, horsehair), waist (drawstring, ties, wrap) |
| Linen Coif | Head | 1×1 | 10 to 80 | Coif, undercap, net, brimmed, pointed or tied nightcap |
| Wimple | Neck | 1×1 | 35 to 70 | Wimple, barbe |

Names come from signatures (`specific_name`): Shirt, Long Shirt, Chemise, Smock, Undertunic, Subucula, Særk, Hadajuban, Nagajuban, Zhongyi, Nightshirt, Night Chemise, Hair Shirt; Braies, Short Braies, Drawers, Sokgot; Fascia, Strophium, Sarashi, Breast Wrap, Breast Girdle, Dudou, Lengberg Band; Loincloth, Fundoshi, Etchu, Kaupinam, Langota, Cache-sexe, Perizoma, Subligaculum, Tubban; Hose, Joined Hose, Stockings, Soled Hose, Stirrup Hose, Leggings; Socks, Udones, Foot Wraps, Tabi, Coptic Socks, Nålbound Socks, Piloi; Petticoat, Underskirt, Quilted Petticoat, Koshimaki, Sokchima, Mamigun; Coif (always with its material: Linen Coif), Undercap, Hairnet, Caul, Nightcap; Wimple, Barbe. An unusual material or lawn goes first ("Silk Chemise", "Lawn Shirt"), stays after ("Chemise and Stays").

## Building the models

- **Body points:** every edge is a `body::Joint` level or a share between two (thigh halfway from hip to knee, calf halfway from knee to ankle), so models scale with each people's frame as armour does.
- **Tucked in:** close, 1 to 2 mm off the skin, wherever a higher layer covers that location: socks 1 mm under footwear's 4 mm upper, everything well inside armour's 15 mm `STANDOFF_MM`. A piece's own ease only where it is uncovered: fitted 3 to 5 mm, loose 8 to 15, full 20 to 40. `Inventory::shown` already leaves out a piece covered at every place.
- **Hems and ends:** shirt hems tuck into the braies or hang free; braies legs end inside the hose tops; hose tops tie to the braies belt with points or are gartered below the knee; sleeves gather at the wrist under a Middle sleeve.
- **Shapes:** mostly rectangles and gussets: a few lofted tubes and a panel or two. Wraps (breast bands, loincloths, foot wraps) are bands following the body with a tucked end.
- **Bodies (from the fantasy pieces):** tails through a slit closed with toggles, a keyhole or split ties round the tail root; horns through ringed holes in coifs, wimples pinned behind swept-back horns; wing-arms take sleeveless pieces open down the sides and tied at the shoulders, breast bands below the wing roots; toe-walking legs take hose with a high heel and a short open-toed foot; hooves take felt cups drawn up to the fetlock; talons take socks with open toe tubes.

## Fantasy pieces

Worth keeping as named specials once uniques exist: Mithril-Thread Undertunic (dwarf), Moonthread Strophium (elf), Penitent's Hair Shirt, Horn-Cut Coif (minotaur), Wingroot Bandeau and Skyborne Wingslit Shift (birdfolk), Ember-Tail Breechclout (kobold), Riverbank Tail Cloth and Marshwife's Tail Skirt (lizardfolk), Felted Hoof Wraps (minotaur), Talon-Sheath Socks (birdfolk), Clawtoe Hose (kobold). Each keeps a historical shape and differs in material, decoration and fit.

## Left out

- Modern and 19th-century pieces: briefs, bras, union suits, pantalettes, slips, bloomers, girdles, A-shirts, bed jackets.
- Religious garments: kachera, tallit katan, sedreh, temple garments, priestly undergarments. The hair shirt stays (decision 9).
- Codpieces and garters as items (they are hose properties); chastity belts (largely a later myth).
- For the mid-layer research: tunics worn alone, kirtle and cotte, doublet, pourpoint, gipon, bodice, partlet, smock-frock, poet shirt, chiton, peplos, stola, tunica, jeogori, baji, kosode and hitoe, sirwal and shintiyan, the shendyt, leg bindings and puttees (worn over hose), veils, fillets and snoods, farthingale and bum roll.

## Pictures

| Kind | References |
| ---- | ---------- |
| Shirt | [1](https://commons.wikimedia.org/wiki/File:Tr%C3%A9sors_de_Saint_Louis_dans_le_tr%C3%A9sor_de_Notre-Dame_de_Paris_2018-05-12.jpg) [2](https://commons.wikimedia.org/wiki/File:Chemise_(drawing).jpg) [3](https://commons.wikimedia.org/wiki/File:Chemise_MET_65.273.1_front_CP4.jpg) [4](https://commons.wikimedia.org/wiki/File:Ambrosius_Holbein_002detail.jpg) [5](https://commons.wikimedia.org/wiki/File:Juban.png) [6](https://commons.wikimedia.org/wiki/File:Nightshirt_MET_1984.84_F.jpg) [7](https://commons.wikimedia.org/wiki/File:A_pair_of_stays.JPG) [8](https://commons.wikimedia.org/wiki/File:Bodice_MET_DP240324.jpg) |
| Braies | [1](https://commons.wikimedia.org/wiki/File:Servant_in_braies_-_details.gif) [2](https://commons.wikimedia.org/wiki/File:Braies.jpg) [3](https://commons.wikimedia.org/wiki/File:Braies_(drawing).jpg) |
| Breast Band | [1](https://commons.wikimedia.org/wiki/File:The_so-called_Bikini_Girls_Mosaic_depicting_ten_female_athletes_dressed_in_a_subligaculum_(loincloth_short)_and_a_strophium_(breast-cloth)_doing_various_sports_exercises_and_receiving_awards,_dated_to_AD_320,_Villa_del_Casale_3.jpg) [2](https://commons.wikimedia.org/wiki/File:The_Childrens_Museum_of_Indianapolis_-_Embroidered_infant_undergarment.jpg) |
| Loincloth | [1](https://commons.wikimedia.org/wiki/File:Box_decorated_with_offering_scenes_for_Kha_and_Merit,_containing_loincloths_and_other_linen,_wood_-_Museo_Egizio_Turin_S_8613_p09.jpg) [2](https://commons.wikimedia.org/wiki/File:Romanbikini.gif) [3](https://commons.wikimedia.org/wiki/File:Subligaculum_1.1_-_Anthony_Rich,_Illustrated_Companion_to_the_Latin_Dictionary,_and_Greek_Lexicon,_p._627.jpg) [4](https://commons.wikimedia.org/wiki/File:Hokkaidoinstructionswiki.JPG) [5](https://commons.wikimedia.org/wiki/File:PSM_V65_D205_Fishermen_of_misaki.png) |
| Hose | [1](https://commons.wikimedia.org/wiki/File:Chausses_1440_Cleeves.jpg) [2](https://commons.wikimedia.org/wiki/File:Calzebrache.jpg) [3](https://commons.wikimedia.org/wiki/File:Braies.jpg) [4](https://commons.wikimedia.org/wiki/File:Don_Juan_d%27Austria_1.JPG) [5](https://commons.wikimedia.org/wiki/File:Codpiece.jpg) |
| Socks | [1](https://commons.wikimedia.org/wiki/File:Wool_socks_made_by_nalbinding_-_53642875380.jpg) [2](https://commons.wikimedia.org/wiki/File:Wool_socks_made_by_nalbinding_-_53642753494.jpg) [3](https://commons.wikimedia.org/wiki/File:Tabi_(hosiery).jpg) [4](https://commons.wikimedia.org/wiki/File:Marathon_Tabi_from_Idaten.jpg) [5](https://commons.wikimedia.org/wiki/File:FootClothFromFinnishDefenceForces.JPG) |
| Petticoat | [1](https://commons.wikimedia.org/wiki/File:Horsehair_petticoat_MET_CI43.126.26_F.jpg) [2](https://commons.wikimedia.org/wiki/File:Underskirt.jpg) |
| Linen Coif | [1](https://commons.wikimedia.org/wiki/File:Coif_(England),_late_16th%E2%80%93early_17th_century_(CH_18445167).jpg) [2](https://commons.wikimedia.org/wiki/File:Coif_MET_182923.jpg) [3](https://commons.wikimedia.org/wiki/File:1580-1620_man%27s_nightcap,_linen_embroidered_with_silk_and_spangles.jpg) [4](https://commons.wikimedia.org/wiki/File:Night_cap_MET_DP159780.jpg) [5](https://commons.wikimedia.org/wiki/File:Charles_V_and_Joanna_of_Bourbon,_8_October_2012_(detail_of_Joanna_de_Bourbon).jpg) |
| Wimple | [1](https://commons.wikimedia.org/wiki/File:Aleanor_of_Aqutaine.jpg) [2](https://commons.wikimedia.org/wiki/File:Catherine_d%E2%80%99Alen%C3%A7on_mini.jpg) [3](https://commons.wikimedia.org/wiki/File:Gebende.jpg) |
