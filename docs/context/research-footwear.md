# Research: footwear

What shoes, boots, sandals, clogs and slippers a generator can make, with the pictures each model is based on. Done in November 2026 by the method in the bladed weapons research: Wikipedia's footwear categories (339 articles), 140 names ranked with the Google Books Ngram Viewer, 97 articles read by two local models (`qwen3:8b`, `llama3.1:8b`) whose quotes were checked by script and read by hand. About 700 pictures from Wikimedia Commons were looked at, and the chosen ones are linked below (links only: none are in the repo). The code is `shared::footwear` (see `shared/footwear.md`) and `item_art/footwear/`.

Owner's rules: nothing mainly rubber or modern (rubber standing in for leather is fine), nothing with sexual overtones, knee-high boots in. Boots and shoes are the Middle layer; foot armour, worn over them, belongs to the armour research (`research-armour-feet.md`).

## Owner's decisions

- **Thigh-high boots.** Keep the cavalier bucket-top boot, which folds down to a knee boot with a wide cuff, and drop the plain thigh boot? Commons pictures for thigh boots are almost all fetish photos, so references would come from portraits only. Decided: Yes: the cavalier boot only. It is the one thigh-high boot; the plain thigh boot is dropped.
- **Chopine and okobo.** Both are platform clogs: chopines up to 50 cm (Venetian noblewomen and courtesans), okobo worn by maiko, who are teenagers. Keep, drop, or keep only as tall clogs without the specific names? Decided: Yes: chopine and okobo dropped, and platform height stays out of the generator.
- **Paduka.** The toe-knob sandal is everyday footwear but also a votive object in Hinduism and Jainism. Keep it? Decided: Yes: paduka kept as everyday footwear, plain wood only.
- **Wellington.** The name now means rubber boots, but the original was leather. Use the name for the leather boot, or leave it out so nobody expects rubber? Decided: Yes: the name Wellington is left out; the leather boot is a plain riding boot.
- **Outer layer.** Spurs, sabatons, pattens, spats and crampons are worn over shoes or boots. Keep all five as Outer layer items, or drop the layer for now? Greaves are worn on the legs, so they belong with leg armour. Decided: None of them for now: spurs, pattens, spats and crampons are all left out, to revisit later. Foot armour (sabatons) belongs to the armour research (`research-armour-feet.md`). The Plate build goes, as footwear has no Outer items.
- **Item shapes.** Shoes take 2×1 and boots 2×2 today. Knee boots could take 2×3. Item pictures are square so far, so a 2×3 boot needs pictures that aren't square. Decided: Yes: shoes, sandals and slippers 2×1, ankle and calf boots 2×2, knee boots (and the cavalier boot) 2×3. Pictures can already be tall, as weapons' are.

## Properties

Lengths are in foot lengths (L), so they scale with the creature. What each build allows is the grid in `shared::footwear` (`tables.rs`, `holding.rs`, `finish.rs`, `materials.rs`).

- **Build**: How it is made. Decides almost everything else. Sewn (Upper sewn or welted to a separate leather sole; heels possible); Soft (Sole and sides from one piece wrapped up the foot (moccasin, turnshoe, mukluk)); Rawhide (One piece of hide gathered by thongs (pampootie, opanak, carbatina)); Woven (Bast, straw, rope or grass plaited or coiled (bast shoe, waraji, espadrille sole)); Felted (Felted wool in one piece (valenki)); Strapped (A sole held on by straps, loops or a thong (sandals)); Wooden (Carved wood, whole or as the sole (clogs, geta)).
- **Height**: Where the shaft stops, in foot lengths (L) above the ground. A medium foot is about 26 cm. Low (Below the ankle bone: 0.2 to 0.3 L); Ankle (Over the ankle bone: 0.45 to 0.6 L); Calf (Mid-calf: 1.0 to 1.2 L); Knee (Below or at the knee: 1.7 to 1.9 L); Thigh (Above the knee: 2.6 to 2.9 L (the cavalier boot only)).
- **Toe**: Seen from above. Width is a share of foot length; a normal foot is about 0.38 L wide. Round (The usual); Almond (Narrowing to a soft point); Pointed (A clear point at the toe); Long point (Point extending 0.1 to 0.5 L beyond the toes (poulaine)); Square (Flat across (17th c. shoes)); Broad (Wider than the foot, up to 1.2 L (duckbill)); Upturned (Tip curls up (jutti, gutal, tsarouchi, opanak)); Open (Toes show (sandals)).
- **Heel**: Height under the heel. No stilettos or modern high heels. Flat (None); Low (Up to 2.5 cm, stacked leather); Riding (2.5 to 4.5 cm, stacked and angled (cowboy, cavalier, engineer)); Louis (Curved, 4 to 6.5 cm (17th to 19th c. mules and button boots)); Teeth (Wooden blocks or teeth under the sole (geta, nalin)).
- **Fastening** Pull-on (Nothing: slip-on shoes, pull-on boots); Closed laces (Facings under the vamp); Open laces (Facings on top of the vamp); Loops (Laces through loops, no tongue, wound round the ankle); Buckle strap (One or two straps with buckles); Buttons (A row up the outer side); Toggles (Leather toggles through loops (turnshoes)); Elastic sides (Chelsea boots only); Thongs (Thongs or cords wound round the foot and shin); Toe thong (Between the first two toes, or a knob held by them); Drawstring (Round the top of a soft boot).
- **Top edge**: Boots only, except Plain. Plain (Straight round); Cuff (Turned down, often another colour (top boot)); Bucket (Wide and flared (jackboot, cavalier)); Notch (V at the front with a tassel (Hessian)); Scalloped (Wavy edge (cowboy, button boot)); Fur (Fur trim or a fur cuff); Uneven (Higher at the front than the back (gutal)).
- **Decoration** None; Broguing (Punched holes and serrated edges); Embroidery (Thread, gold or silver); Beading (Beads or quillwork); Slashing (Cuts showing the lining); Stitching (Decorative rows of stitching or tooling); Pompom (Wool pompom on the toe); Tassels (On the vamp or the top); Fringe (Leather fringe); Carved (Carved or painted wood); Jewels (Pearls or stones (rank only)).
- **Under the sole** Nothing; Hobnails (Iron studs in rows); Irons (Heel and toe plates; clog irons on clogs).
- **Material**: Upper first; the sole follows the build. Leather (Cow, calf, goat); Suede; Rawhide; Fur (Sealskin, caribou, sheepskin); Cloth (Canvas, wool, silk, velvet, brocade); Felt; Plant (Bast, straw, rope, reed); Wood.

## Styles

Each takes its name when a roll matches its signature (`names.rs`); anything else has the plain name for its kind, build and height.

### Shoes

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Oxford | Closed lacing: the eyelet facings sit under the vamp. | Britain, 19th c. on | Low heel, almond or round toe, often a toe cap. | [1](https://commons.wikimedia.org/wiki/File:Red_leather_oxford_shoes_by_Eugene_Pik_(2021)_05.jpg) |
| Derby | Open lacing: the facings are sewn on top of the vamp. Blucher is folded in here. | Europe, 19th c. on | As the Oxford, with the two facings lifting away from the vamp. | [1](https://commons.wikimedia.org/wiki/File:A_Derby_of_suede_leather_with_brogues.jpg) |
| Brogue | Broguing: punched holes and serrated edges along the seams. Any laced shoe or boot can take it. | Scotland and Ireland, then everywhere | Wingtip toe cap is the common pattern. | [1](https://commons.wikimedia.org/wiki/File:Mens-Shoes-Oxfords-Brogue-thomas-george-collection.jpg) [2](https://commons.wikimedia.org/wiki/File:Leather_soled_shoes_-_Made-to-order_shoes_(patina).png) |
| Monk shoe | No laces: one or two buckled straps across the instep. | Europe | Strap from the outer side over the instep. | [1](https://commons.wikimedia.org/wiki/File:Double_Monk_Felsted_(Grenson).jpg) |
| Loafer | Slip-on, flat or low heel, sewn moccasin-style vamp; tassels or a saddle strap. | 20th c. | Low cut, the raised seam round the vamp. | [1](https://commons.wikimedia.org/wiki/File:A_Penny_Loafer.jpg) [2](https://commons.wikimedia.org/wiki/File:A_brown_Loafer_Shoes_from_TODS.jpg) |
| Court shoe | Slip-on with a low-cut vamp, whole-cut top, bow or buckle as ornament. | 18th c. on | Very low vamp, thin sole; heel low to medium only (no stilettos). | [1](https://commons.wikimedia.org/wiki/File:Shoes_MET_DT2630.jpg) |
| Mary Jane | One buckled strap across the instep, round toe, low heel. | 20th c. | Rounded toe box, thin sole. | [1](https://commons.wikimedia.org/wiki/File:Start-rite_Louisa_traditional_bar_shoes_or_%22Mary-Jane%22_shoes.jpg) |
| Saddle shoe | Laced, with a contrasting panel (saddle) over the instep. | 20th c. | Two-colour upper. | none |
| Brogan | Heavy, ankle-high, laced or buckled, untanned or rough leather, often hobnailed. | Scotland and Ireland, US Army 19th c. | Sits on the ankle bone; thick sole. | [1](https://commons.wikimedia.org/wiki/File:Brogans_MET_55.16.19_CP4.jpg) [2](https://commons.wikimedia.org/wiki/File:Brogans_MET_55.16.20_CP2.jpg) |
| Latchet shoe | Square toe, ties over the tongue through two latchets, stacked red heel. | 17th c. Europe | Square toe, tongue high on the instep. Found in the pictures, no article: name to confirm. | [1](https://commons.wikimedia.org/wiki/File:Shoes_MET_1973.114.3ab_F.jpg) |
| Buckled shoe | A large decorative buckle over the tongue, often jewelled. | Late 17th to 18th c. | Tongue rises above the buckle; heel low to medium. | [1](https://commons.wikimedia.org/wiki/File:French_baroque_shoe,_early_18th_century_-_Bata_Shoe_Museum_-_DSC00112.JPG) |
| Turnshoe | Soft, sewn inside out and turned, so the seams hide inside; 3 to 4 mm sole; toggles, a side lace or a strap. | Middle Ages | No heel, thin sole, ankle-high or below; topband round the opening. | [1](https://commons.wikimedia.org/wiki/File:Turnshoes.jpg) [2](https://commons.wikimedia.org/wiki/File:Object_from_the_exhibition_We_call_them_Vikings_produced_by_The_Swedish_History_Museum_(13619023323).jpg) |
| Poulaine | A very long pointed toe, stuffed with moss or horsehair to keep its shape. Crakow is another name. | 14th to 15th c. Europe | Point well beyond the toes (estimated 0.1 to 0.5 foot lengths). | [1](https://commons.wikimedia.org/wiki/File:Dutch_poulaines,_circa_1450_(5774095801).jpg) [2](https://commons.wikimedia.org/wiki/File:Medieval_shoe.jpg) |
| Duckbill shoe | Very broad, square or rounded toe, slashed to show the lining; straps over the instep. | Early 16th c. Europe | Toe up to a foot wide at its most extreme; flat sole of two thicknesses. | [1](https://commons.wikimedia.org/wiki/File:German_Tudor_sabatons_and_greaves,_possibly_worn_by_a_Lanzichenecchi_official,_16th_century_-_Bata_Shoe_Museum_-_DSC00108.JPG) |
| Moccasin | Soft, one piece wraps up from below, an added vamp on top, puckered seam round it; beads or embroidery. | North America | No heel, sole and sides one piece; some have a cuff. | [1](https://commons.wikimedia.org/wiki/File:Moccasins,_Montagnais,_collected_in_1892_-_Native_American_collection_-_Peabody_Museum,_Harvard_University_-_DSC05835.jpg) [2](https://commons.wikimedia.org/wiki/File:Penobscot_moccasins_ANMH.jpg) |
| Pampootie | One piece of untanned hide, hair left on, folded round the foot and tied with a thong. | Aran Islands | Shapeless and low; hide texture. | none |
| Opanak | Rawhide gathered round the foot, woven straps over the top, a horn-like turned-up tip. | Balkans | Low sides, woven top, curled point. | [1](https://commons.wikimedia.org/wiki/File:Top_view_of_Serbian_Opanak_-_Flickr_-_Ivan_Radic.jpg) [2](https://commons.wikimedia.org/wiki/File:Opanci_XIX_XX_vek_detalj.jpg) |
| Carbatina | One piece of rawhide with the edge cut into loops, laced over the foot. No separate sole. | Greece and Rome | Openwork sides, no heel. | [1](https://commons.wikimedia.org/wiki/File:Carbatina_da_bambino,_in_cuoio,_200-213_dc_ca..,_da_vindolandia_(bardon_mill,_roman_vindolandia_fort_and_museum).jpg) |
| Jutti | Closed, flat, toe tip curled up, heavy gold or silver embroidery; leaves the top of the foot nearly bare. Mojari and khussa are folded in here. | North India, Pakistan | M-shaped or round vamp, no heel, tip curls up. | [1](https://commons.wikimedia.org/wiki/File:Indian_juttis,_late_20th_century,_Punjab_-_Bata_Shoe_Museum_-_DSC00432.JPG) [2](https://commons.wikimedia.org/wiki/File:Punjabi_jutti_at_Dilli_Haat.jpg) |
| Tsarouchi | Stiff leather, sewn moccasin fashion, upturned toe with a large wool pompom. | Greece | Upturned toe, pompom on the point; studded sole. | [1](https://commons.wikimedia.org/wiki/File:Tsarouchia.jpg) [2](https://commons.wikimedia.org/wiki/File:Greek_Evzones_wearing_tsarouchi_shoes.jpg) |
| Espadrille | Rope sole of esparto grass, canvas or cotton upper. | Spain, southern France | Flat, thick rope sole edge visible all round. | [1](https://commons.wikimedia.org/wiki/File:Espadrilles_(drawing).jpg) [2](https://commons.wikimedia.org/wiki/File:Alpargatas_oliva.jpg) |
| Bast shoe | Woven from bast (linden bark), basket-like, tied on with cords. | Russia, Eastern Europe | Plaited lattice, low sides, cords round the shin. | [1](https://commons.wikimedia.org/wiki/File:Lapti_-_19c_-_Technical_museum,_Nizhny_Novgorod.jpg) [2](https://commons.wikimedia.org/wiki/File:Lapti_-_Smolensk_Historical_Museum.jpg) |
| Cloth shoe | Cloth upper, slip-on or a strap, flat layered sole. | China | Low-sided, rounded toe, no heel. | [1](https://commons.wikimedia.org/wiki/File:Cloth_shoes_by_Pixian_Shoe_Factory_intended_for_Deng_Xiaoping%27s_Hong_Kong_tour_(20250115103135).jpg) |
| Ghillie | No tongue; laces run through loops and wrap round the ankle. | Scotland | Open top, laces criss-crossed up the ankle. | [1](https://commons.wikimedia.org/wiki/File:Ghillie_Brogues.jpg) |

### Ankle boots

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Chelsea boot | Pull-on, elastic panels on both sides, a tab or loop at the back. | Victorian Britain | Low heel; elastic gore is the only rubber part. | [1](https://commons.wikimedia.org/wiki/File:Chelsea_boot,_black.jpg) [2](https://commons.wikimedia.org/wiki/File:Chelsea_boot,_brown_suede.jpg) |
| Chukka boot | Open lacing with two or three pairs of eyelets, suede or calf, unlined. Desert boot is folded in here. | 20th c. | Two-piece upper, thin sole, round toe. | [1](https://commons.wikimedia.org/wiki/File:Desert_boots_(drawing).jpg) |
| Jodhpur boot | A strap wraps round the ankle and buckles at the side, through a loop at the back. | India, then riding | Low heel, round toe. | [1](https://commons.wikimedia.org/wiki/File:Jodhpur_Grenson_Hereford.jpg) |
| Hobnail boot | Laced, iron hobnails under the sole, heel iron and toe plate. Ammunition and trench boots are folded in here. | 18th to 20th c. soldiers and workers | Studs set in rows under the sole. | [1](https://commons.wikimedia.org/wiki/File:Haferlschuhe_mit_Krampen_(fcm).jpg) [2](https://commons.wikimedia.org/wiki/File:British_Army_GS_boot.jpeg) |
| Button boot | Buttoned up the outer side; curved Louis heel. | 19th c. | Scalloped top edge, row of small buttons. Bottine is the French name: to confirm. | [1](https://commons.wikimedia.org/wiki/File:Boots_MET_43.129.8_S.jpg) [2](https://commons.wikimedia.org/wiki/File:Boots_MET_CI42.67.23ab_F.jpg) |
| Brogan | See Shoes: sits at the ankle. |  |  | none |

### Calf boots

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Cowboy boot | Pull-on, shaft to at least mid-calf, angled stacked heel over 2.5 cm, decorative stitching, pointed or round toe. | American West, 19th c. on | Scalloped top with pull holes, slim angled heel. | [1](https://commons.wikimedia.org/wiki/File:Cowboy_boots_IMGP9014.jpg) [2](https://commons.wikimedia.org/wiki/File:Cowboy_boots_cropped.png) |
| Engineer boot | Pull-on, thick leather, buckled strap at the top of the shaft and over the instep. | US, 1930s | Shaft 18 to 43 cm, heel about 4.4 cm. | [1](https://commons.wikimedia.org/wiki/File:2970_engineer.jpg) |
| Mukluk | Soft boot of sealskin or caribou, fur outside, wrapped sole, drawstring top. Kamik is the Inuit name. | Arctic | No heel, wrapped sole with the seam on the sides, fur cuff or drawstring. | [1](https://commons.wikimedia.org/wiki/File:Inuit_boot,_Iglulingmiut,_1987_-_Bata_Shoe_Museum_-_DSC00413.JPG) [2](https://commons.wikimedia.org/wiki/File:Inuit-Stiefel.jpg) |
| Valenki | Felted wool in one piece, no seams, no outer sole. | Russia | Straight round shaft, thick and soft. | [1](https://commons.wikimedia.org/wiki/File:Valenki.jpg) [2](https://commons.wikimedia.org/wiki/File:Handmade-feltboots.jpg) |
| Gutal | Upturned toe, thick flat sole, tall shaft with an uneven top, leather appliqué. | Mongolia | Toe curls up, thick sole, tall shaft cut lower at the back. | [1](https://commons.wikimedia.org/wiki/File:Mongolian_boot,_2006_-_Bata_Shoe_Museum_-_DSC00360.JPG) [2](https://commons.wikimedia.org/wiki/File:Mongolian_boots_belonging_to_Dondoghulam,_first_consort_of_Bogd_Javzandamba_Agvaanluvsan_the_8th,_ruler_of_Mongolia,_early_20th_century_-_Bata_Shoe_Museum_-_DSC00350.JPG) |
| Calceus | Closed leather shoe-boot to the lower shin, tied with crossing thongs, often hobnailed. | Rome | Thongs wound round the shin. | [1](https://commons.wikimedia.org/wiki/File:P%C3%A9rigueux_Vesunna_Museum_-_Bronze_1_Calceus.jpg) [2](https://commons.wikimedia.org/wiki/File:J.S._Hall,_Greek_and_Roman_shoes,_in_The_book_of_the_feet_Wellcome_L0015084.jpg) |
| Buskin | Laced from the toes to the top of the shaft, sometimes open across the toes. | Greece and Rome, then Renaissance | Calf or knee shaft, lacing up the front. | none |
| Hwa | Soft black boots, felt or leather, thick white-edged sole. | Korea | Straight shaft, upturned edge of the sole at the toe. | [1](https://commons.wikimedia.org/wiki/File:Korean_traditional_men%27s_winter_shoes.jpg) |

### Knee boots

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Riding boot | Pull-on, plain, low heel under 2.5 cm, smooth sole. | Europe | Tall straight shaft; the top cut higher at the front. | [1](https://commons.wikimedia.org/wiki/File:Riding_boots_MET_CI42.10.1ab.jpg) [2](https://commons.wikimedia.org/wiki/File:Boots_MET_CI45.30.1ab.jpg) |
| Top boot | A riding boot with a turned-down cuff of another colour. | 18th to 19th c. | Contrasting cuff of about 0.5 foot lengths. | [1](https://commons.wikimedia.org/wiki/File:Riding_boots_006.jpg) |
| Hessian boot | V-notch at the front of the top with a tassel; low heel, polished. | 18th to 19th c. | Heart or V-shaped top edge, tassel at the notch. | [1](https://commons.wikimedia.org/wiki/File:Hessian_boots_(PSF).png) [2](https://commons.wikimedia.org/wiki/File:Pair_of_Man%27s_Hessian_Boots_LACMA_M.67.8.140a-b.jpg) |
| Jackboot | Very stiff, the top widened into a wing or bucket over the knee; sometimes mail in the lining. | 17th to 18th c. cavalry | Rigid tube, flared top, spur rest at the heel. | [1](https://commons.wikimedia.org/wiki/File:JackbootsHouseholdCavalry.jpg) [2](https://commons.wikimedia.org/wiki/File:Jackboot(PSF).png) |
| Russian boot | Soft leather, wide top, wrinkles round the ankle; Cuban or Louis heel. | Russia, Cossacks | Concertina folds at the ankle. | [1](https://commons.wikimedia.org/wiki/File:Dierussischearmee1901wien190110_0458_DIE_RUSSISCHE_ARMEE_(_Hof-_u_Staatsdruckerei_Wien_1901)_Uniformen_Imperial_Russian_Army_uniforms_insignia_Tafel_05_Kasak_20._Don-Kasaken-Regt._(cossack)_Husar_Leibgarde-Husaren-regt_des_kaisers_(h.jpg) |

### Thigh boots

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Cavalier boot | Soft calfskin, wide flared bucket top worn over the knee or folded down; high wooden heel; spur leathers. | 1620s to 1660s | Bucket top; folded down it shows as a knee boot with a wide cuff. | [1](https://commons.wikimedia.org/wiki/File:Prince_Rupert_(1619-1682),_1st_Duke_of_Cumberland_and_Count_Palatine_of_the_Rhine_RMG_BHC2990.jpg) |

### Sandals

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Caliga | Openwork boot-sandal cut from one hide, laced up the middle of the foot and the ankle, thick hobnailed sole. | Roman soldiers | Lattice of straps, studs under the sole. | [1](https://commons.wikimedia.org/wiki/File:MAN_-_Caliga_2257.jpg) [2](https://commons.wikimedia.org/wiki/File:Caligae_from_side.jpg) |
| Crepida | Thick sole, a low upper with loops that straps thread through, net-like over the foot. | Greece and Rome | Loop net over the top of the foot. | [1](https://commons.wikimedia.org/wiki/File:Belvedere_Apollo_Pio-Clementino_Inv1015_n7.jpg) |
| Huarache | Upper of woven leather straps, leather or woven sole. | Mexico | Basket-weave front. | [1](https://commons.wikimedia.org/wiki/File:Huarache_Izq.PNG) |
| Kolhapuri | Braided leather T-strap with a loop for the big toe; no nails; about 100 g. | India | Flat, toe loop, broad strap. | [1](https://commons.wikimedia.org/wiki/File:Kolhapuri_Chappals_in_roadside_shop_in_Kolhapur2.jpg) |
| Waraji | Woven rice straw, toes over the front edge, tied on with long cords round the ankle. | Japan | Thick plaited sole, cord loops. | [1](https://commons.wikimedia.org/wiki/File:Waraji_on_tatami.jpg) [2](https://commons.wikimedia.org/wiki/File:Waraji.jpg) |
| Zori | Flat thong sandal of straw, cloth, lacquered wood or leather. | Japan | Flat oval sole, V-shaped thong. | [1](https://commons.wikimedia.org/wiki/File:Zoori_straw.png) |
| Reed sandal | Woven or coiled papyrus, palm or grass, one toe post and a strap. | Ancient Egypt | Flat oval sole, thin strap. | [1](https://commons.wikimedia.org/wiki/File:Ancient_Egyptian_sandals.jpg) [2](https://commons.wikimedia.org/wiki/File:Palm_leaf_sandals_New_Kingdom_(1539-1075_BCE_or_Later)_Egypt_Penn_Museum.jpg) |

### Clogs

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Sabot | Carved from one block of wood, sole and upper together. Klompen are the Dutch kind. | France, Low Countries | Thick, round, upturned tip; painted or carved. | [1](https://commons.wikimedia.org/wiki/File:Sabot1.jpg) [2](https://commons.wikimedia.org/wiki/File:Klompen_(Dutch_Clogs),_Wooden_Shoes_Museum_in_Drenthe.jpg) |
| Clog | Leather upper on a wooden sole, laced or clasped, clog irons nailed under. | Britain | Thick wooden sole with a raised toe, row of tacks round the welt. | [1](https://commons.wikimedia.org/wiki/File:Red_Lancashire_clogs_(side_view).jpg) |
| Geta | Flat wooden board on two wooden teeth, cloth thong between the toes. | Japan | Board about 0.15 foot lengths above the ground (estimate). | [1](https://commons.wikimedia.org/wiki/File:Geta.png) [2](https://commons.wikimedia.org/wiki/File:Geta_footwear.jpg) |
| Paduka | Wooden sole in the shape of a footprint with a knob held between the toes; no straps. | India | Knob post, two narrow stilts. | [1](https://commons.wikimedia.org/wiki/File:Paduka.jpg) |
| Nalin | Wooden bath clog on blocks under heel and ball, 5 to 30 cm high, inlaid; one strap. | Ottoman Empire | Two blocks; height marked rank. | none |
| Bakya | Light wooden sole with carved sides and one cloth or rattan strap. | Philippines | Carved side panels. | none |

### Slippers

| Style | Signature | When and where | Look | Pictures |
| ----- | --------- | -------------- | ---- | -------- |
| Mule | No back; flat, or a heel of 4 to 6.5 cm in the 17th and 18th centuries; embroidered. | Europe | Open heel, pointed or round toe. | [1](https://commons.wikimedia.org/wiki/File:Mules_MET_29.23.11dig.jpg) [2](https://commons.wikimedia.org/wiki/File:Mules_MET_54.61.101a-b_CP4.jpg) |
| Babouche | Leather slipper, pointed toe, the back folded flat under the heel. Balgha is folded in here. | Morocco, Middle East | No heel, long pointed vamp. | [1](https://commons.wikimedia.org/wiki/File:Babuchas_01_--_2014_--_Marrakech,_Marruecos.jpg) [2](https://commons.wikimedia.org/wiki/File:Traditional_Moroccan_shoes_called_%22babouches%22.JPG) |
| Soccus | Low soft leather slipper, the comic actor's shoe. | Rome | Very low sides. | [1](https://commons.wikimedia.org/wiki/File:Soccus_1.1_-_Anthony_Rich,_Illustrated_Companion_to_the_Latin_Dictionary,_and_Greek_Lexicon,_p._607.jpg) |
| Slipper | Generic soft indoor shoe of cloth, felt or leather. |  | Thin sole, no fastening. | [1](https://commons.wikimedia.org/wiki/File:The_Masonic_Blue_Slipper.jpg) |

## Weights

Per pair, in grams; all estimates but the kolhapuri's (about 100 g). The code adds heels, nails, irons and wide tops, and scales by the upper's material.

- Sandal, slipper, soft shoe: ~250 to 350
- Shoe: ~500 to 700 (game: 500)
- Wooden clog: ~800 to 1,200
- Ankle boot: ~900 to 1,100 (game boots: 1,100)
- Calf boot: ~1,200 to 1,500
- Felt boot: ~1,000 to 1,300
- Knee boot: ~1,600 to 2,000

## What the sources didn't say

- **Sizes and weights.** Almost no article gives shaft height, heel height or weight. The sourced ones: engineer shafts 18 to 43 cm, heels 4.4 cm; cowboy heel over 2.5 cm; riding heel under 2.5 cm; okobo 10 to 15 cm; nalin 5 to 30 cm; chopine over 50 cm; turnshoe sole 3 to 4 mm; patten raised 10 cm; kolhapuri about 100 g. Decision: Work out the rest from foot length (L) and accept estimates, as for blades.
- **Long points and broad toes.** The poulaine's length and the duckbill's width are stated only loosely (a foot wide at most). Decision: Estimate: long point up to 0.5 L beyond the toes, broad toe up to 1.2 L wide. Tweak by eye on the model.
- **Model judgement.** Both models guessed at the choice fields (rubber soles on waraji, rubber essential to moccasins, geta above the knee). Decision: Ignored. Choices come from the quotes, the pictures and general knowledge.
- **Pictures.** The Met's open access API now answers 410 Gone, so all pictures come from Commons (many are Met photos posted there). Some searches went wrong (a crampon search found a priest named Crampon). Decision: Good references for most styles; buskin, saddle shoe, pampootie, nalin, bakya and crampon have none yet.
- **Popularity.** Ngram is case sensitive and lower case was queried, so names of places and people (Oxford, Chelsea, Hessian, Mary Jane, Russian) score near zero. Pumps and mules are swamped by other meanings. Decision: Judged by hand, marked ‡.
- **Colours and dyes.** Articles mention colours only in passing (red heels, purple straps, black, brown). Decision: Use the game's colour rules for materials; no research needed.

## Left out

- **Mainly rubber or modern:** Wellington (rubber), galoshes (rubber), trainers, hiking and combat boots with moulded soles, flip-flops, Crocs, jelly shoes, moon boots, caulk boots, sea boots.
- **Sexual overtones:** Stilettos, high heels as a style, platform shoes, go-go boots, ballet boots, thigh boots as fashion (Commons category), fetish boots.
- **Thigh-high, not historical:** Waders, hip boots, over-the-knee fashion boots.
- **Sensitive, out unless the owner says otherwise:** Lotus shoes (foot binding), papal and episcopal slippers.
- **Tools for later:** Snowshoes, ice cleats, skates, stilts, horseshoes.
- **Not footwear:** Socks and foot wraps (another layer), greaves (legs), shoe polish, shoe trees.
- **Owner's decisions:** chopines and okobo (platforms), the plain thigh boot, the name Wellington, and spurs, pattens, spats and crampons (for now).
- **Not a style:** talaria, Hermes' winged sandals, are a magic item rather than a kind of footwear.
