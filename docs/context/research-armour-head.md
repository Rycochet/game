# Research: armour for the head

Done 2026-11-03 by the Head chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/1pDxkxyRcMwvCQ14xzDDQY); their answers are in section 13 and in `decisions.md`. Code: `crates/shared/src/armour/head/`, `crates/client/src/item_art/armour/head/`. Where the code differs from the plan below, the code and `armour.md` win: the helmet rows are in `head/table.rs`, and a few moved apart to keep their names (barbutes have no flare, Boeotian helmets have one).

Evidence: **S** sourced (Wikipedia quote), **W** worked out, **E** estimate or general knowledge.

Sources: 77 Wikipedia articles (42 read by qwen3:8b and llama3.1:8b, quotes checked), Google Books Ngram 2000-2019, and Wikimedia Commons pictures for 42 styles (kept outside the repo).

Evidence: **S** sourced (Wikipedia article quote), **W** worked out, **E** estimate or Claude's knowledge, to check.

## 1. What a head piece is

Kinds, decided first (as built, after the owner's answers):

| Kind | Layer | Locations | What |
| --- | --- | --- | --- |
| Arming Cap | Skin | Head | padded cap: worn under a coif or helmet, or alone |
| Coif | Middle | Head, Neck | hood of mail, ring mail or quilting to the shoulder line, face open |
| Helmet | Outer | Head | everything else, rolled from the properties below |
| Bevored Helm | Outer | Head, Neck | the frog-mouth helm and great bascinet, whose plates guard the throat |

Boundary agreed with the Neck chat: aventails, camails, gorgets and separate bevors are Neck items. Chin and throat plates that are part of a helmet (armet, close helmet, falling buffe) stay Head. A frog-mouth helm and a great bascinet, whose plates replace a gorget, are Head + Neck.

## 2. Properties of a helmet (rolled in this order)

1. **Construction** (`shared::armour::Construction`): Plate, Lamellar, Leather. (Padded and Mail are the cap and coif; Scale, Laminar, Splint, Mirror, Plates in cloth, Mail and plate don't make helmets: E.)
2. **Bowl**, how the skull is built (Plate only): Raised (one piece), Ridged (two halves on a central ridge: late Roman, Vendel), Segmented (spangen: plates in a frame of bands), Plates (many narrow plates riveted round: kabuto, Mongol, Chinese). Lamellar: laced. Leather: moulded.
3. **Skull** shape: Round, Low (shallow), Conical, Ogival (pointed, the point towards the back: bascinet, sugarloaf), Flat (cylinder, flat top: early great helm), Onion (bulbous, fluted: turban helmet), Forward (top curling forward: Phrygian).
4. **Face**: Open, Nasal (a bar down the nose), Spectacles (guards round the eyes), Bars (one or three bars from a peak: lobster pot), Opening (T or Y shaped, cheeks part of the bowl: barbute, Corinthian), Faceplate (fixed, sight slits and breaths: great helm), Slit (one sight slit between the skull and a raised front: frog-mouth, some sallets), Visor (moving), Mask (a face, fixed or tied on: Roman cavalry, mempō).
5. **Visor** style (Visor only): Rounded (armet, great bascinet), Pointed (hounskull, "pig-faced"), Klapp (small, hinged at the brow), Bellows (close helmet: ridged, slotted), Grotesque (a face: Savoyard, fantasy skull or beast).
6. **Cheeks**: hinged or fixed cheek pieces, yes/no (with Open, Nasal, Bars only).
7. **Back**: None, Flare (a small lip: Corinthian, Montefortino), Guard (a broad plate: Imperial galea), Tail (one piece sweeping back: sallet), Lames (overlapping strips: lobster tail, Japanese shikoro), Flaps (hanging lamellar or quilted flaps: Mongol, Chinese, Korean).
8. **Brim**: None, Peak (front only: burgonet, lobster pot, kabuto's mabizashi), Narrow (all round: cabasset), Wide (all round, sloping: kettle hat, Boeotian, jingasa), Boat (curving up fore and aft: morion).
9. **Top**: None, Knob (Montefortino), Spike (Mongol, Chinese, kulah khud), Stalk (cabasset's little point), Comb (a low ridge: close helmet, armet), High comb (morion, burgonet), Crest (horsehair on a holder: Greek, Roman), Plume (feather holder: kulah khud, later helmets).
10. **Ornament**: None, Horns, Wings, Front crest (Japanese maedate / kuwagata), Crown (a circlet), plus fantasy Antlers, Dragon. Rare, except as named rows.
11. **Material** (by construction, section 4) and **finish** (section 5).

Things not rolled, drawn from the form: rivets, sight slit sizes, breaths, the brow line. A lining is assumed in every helmet (no property).

## 3. Allowed combinations

Rules (so nothing is rolled and thrown away; each row of the table rolls only inside them):

- Brim Narrow, Wide or Boat: face Open only (cheeks allowed with Boat), back None.
- Brim Peak: face Open, Nasal, Bars or Mask; back Flare, Guard or Lames.
- Visor, Faceplate, Slit, Opening: brim None; Faceplate and Slit with back None; Opening with Flare or None; Visor with Flare, Tail or None.
- Back Tail: skull Round only. Back Flaps: construction Lamellar, or Plate with bowl Plates.
- Cheeks: only with face Open, Nasal or Bars.
- Skull Flat or Ogival: face Faceplate, Visor or Open only. Onion: Open or Nasal.
- Lamellar: skull Round or Conical, face Open or Nasal, back Flaps or None, brim None.
- Leather: skull Round, Low or Conical; face Open (cheeks allowed); back None or Flare; brim None, Peak or Wide.
- Top Crest: skull Round or Forward. High comb: Round. Spike: Round, Conical or Onion. Stalk: Ogival.

| Face \ Back | None | Flare | Guard | Tail | Lames | Flaps |
| --- | --- | --- | --- | --- | --- | --- |
| Open | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Nasal | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| Spectacles | ✓ | – | – | – | – | – |
| Bars | – | ✓ | – | – | ✓ | – |
| Opening | ✓ | ✓ | – | – | – | – |
| Faceplate | ✓ | – | – | – | – | – |
| Slit | ✓ | – | – | ✓ | – | – |
| Visor | ✓ | ✓ | – | ✓ | – | – |
| Mask | ✓ | ✓ | ✓ | – | ✓ | – |

## 4. Names

### Generic names (always kept), by face and brim

| Name | When | Example |
| --- | --- | --- |
| Arming Cap | Cap kind | |
| Coif / Mail Coif | Coif kind (Padded → Coif, Mail → Mail Coif) | |
| Skull Cap | helmet, face Open, no cheeks, back None, brim None, skull Round or Low | cervelliere, secrete |
| Helmet | any other open-faced helmet (Open, Nasal, Spectacles, Bars, Opening) | |
| Helm | closed face: Faceplate, Slit, Visor, Mask | |
| War Hat | brim Narrow, Wide or Boat | |

Material comes first when it isn't iron or steel: "Bronze Helmet", "Leather Cap", "Lamellar Helmet". Fantasy ornaments come first on any name: "Horned Helm", "Winged Helmet", "Crowned Great Helm".

### Specific names (rows of the table)

Ngram is per million words, 2000–2019, summed spellings; ancient names in capitals. ✂ marks below the cut, kept for a reason given.

| Name | Construction / bowl | Skull | Face | Cheeks | Back | Brim | Top | Weight kg | Ngram | Ev. |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Arming Cap | padded cap | Round | – | – | – | – | – | 0.2–0.3 | 0.0022 | S/E |
| Coif | padded coif | – | – | – | – | – | – | 0.4–0.6 | | E |
| Mail Coif | mail coif | – | – | – | – | – | – | 2–3 | 0.0042 | S/E |
| Skull Cap (cervelliere) | Plate raised | Round, Low | Open | no | None | None | None | 0.8–1.2 | 0.0002 | S/E |
| Nasal Helm | Plate raised or segmented | Conical | Nasal | no | None | None | None | 1.4–2.0 | 0.0004 | S/W |
| Spangenhelm | Plate segmented | Conical, Round | Open, Nasal | yes/no | None | None | None, Knob | 1.5–2.0 | 0.0010 | S/W |
| Ridge Helmet | Plate ridged | Round | Open, Nasal | yes | Guard, None | None | None, Crest | 1.5–2.0 | 0.0003 | S/W |
| Spectacle Helm | Plate ridged or segmented | Round | Spectacles | – | None | None | None, Comb | 1.5–2.0 | (viking helmet 0.0058) | S/W |
| Great Helm | Plate raised | Flat, Ogival | Faceplate | – | None | None | None, Crest | 2.5–3.5 | 0.0072 | S |
| Sugarloaf Helm | Plate raised | Ogival | Faceplate | – | None | None | None | 2.5–3.5 | 0.0004 | E (no article) |
| Bascinet | Plate raised | Ogival | Open | no | None | None | None | 1.8–2.5 | 0.0154 | S/W |
| Hounskull | Plate raised | Ogival | Visor Pointed | – | None | None | None | 2.8–3.6 | 0.0001 ✂ well known | S/W |
| Klappvisor Bascinet | Plate raised | Ogival | Visor Klapp | – | None | None | None | 2.4–3.0 | ✂ visor style | S/W |
| Great Bascinet (Head + Neck) | Plate raised | Round | Visor Rounded | – | None | None | None | 4–5 | ✂ | E |
| Kettle Hat | Plate raised or segmented | Round, Conical | Open | no | None | Wide | None, Comb | 1.5–2.5 | 0.0007 | S/W |
| Sallet | Plate raised | Round | Open, Slit, Visor Klapp | no | Tail | None | None, Comb | 2.0–3.5 | 0.0153 | S/W |
| Barbute | Plate raised | Round | Opening | – | None, Flare | None | None, Comb | 2.0–2.5 | 0.0015 | S/W |
| Armet | Plate raised | Round | Visor Rounded | – | Flare | None | Comb | 3.0–3.6 | 0.0026 | S/W |
| Frog-mouth Helm (Head + Neck) | Plate raised | Flat | Slit | – | None | None | None | 7–10 | 0.0000 ✂ iconic jousting | S/E |
| Close Helmet | Plate raised | Round | Visor Bellows, Rounded | – | Flare | None | Comb | 3.0–4.5 | 0.0014 | S/W |
| Burgonet | Plate raised | Round | Open (+ Visor Bellows: closed burgonet with falling buffe) | yes | Flare | Peak | High comb, Comb | 1.8–2.5 | 0.0040 | S/W |
| Morion | Plate raised | Round | Open | yes/no | None | Boat | High comb | 1.5–2.3 | 0.0074 | S/W |
| Cabasset | Plate raised | Ogival | Open | no | None | Narrow | Stalk | 1.3–2.0 | 0.0005 | S/W |
| Lobster-tailed Pot | Plate raised or segmented | Round | Bars, Nasal | yes | Lames | Peak | None, Knob | 2.0–2.7 | 0.0001 ✂ well known | S/W |
| Chichak | Plate raised | Conical | Nasal | yes | Lames | Peak | Spike | 1.8–2.5 | 0.0002 (+ shishak 0.0005) | S/E |
| Turban Helmet | Plate raised | Onion | Open, Nasal | no | None | None | Spike | 2.0–3.0 | 0.0002 | S/W |
| Kulah Khud | Plate raised | Round | Nasal | no | None | None | Spike, Plume | 1.3–2.0 | 0.0000 ✂ Indo-Persian staple | E |
| Lamellar Helmet | Lamellar | Round, Conical | Open | no | Flaps | None | Spike, Plume | 1.5–2.5 | 0.0001 | S |
| Steppe Helmet (Mongol, Chinese) | Plate plates | Conical, Round | Open, Nasal | no | Flaps | None, Peak | Spike, Plume | 1.5–2.5 | 0.0002 | S/E |
| Kabuto | Plate plates | Low, Round | Open, Mask | no | Lames | Peak | None (Ornament Front crest) | 2.0–3.5 | 0.0042 | S/E |
| Jingasa | Plate raised, Leather | Low | Open | no | None | Wide | None | 0.5–1.2 | 0.0007 | E |
| Corinthian Helm | Plate raised (bronze) | Round | Opening | – | Flare | None | None, Crest | 1.5–2.2 | 0.0059 | S/E |
| Attic Helmet | Plate raised (bronze) | Round | Open | yes | Flare | Peak | None, Crest | 1.2–1.8 | 0.0020 | S/W |
| Chalcidian Helmet | Plate raised (bronze) | Round | Nasal | yes | Flare | None | None, Crest | 1.2–1.8 | 0.0003 | S/W |
| Boeotian Helmet | Plate raised (bronze) | Round | Open | no | None | Wide | None | 1.0–1.5 | 0.0004 | S/W |
| Illyrian Helmet | Plate raised (bronze) | Round | Opening | – | Flare | None | Crest | 1.2–1.8 | 0.0002 | S/W |
| Phrygian Helmet | Plate raised (bronze) | Forward | Open | yes | Flare | Peak | None, Crest | 1.3–2.0 | 0.0011 | S/W |
| Pilos Helmet | Plate raised (bronze) | Conical | Open | no | None | None | None | 0.8–1.2 | 0.0003 | S/W |
| Montefortino Helmet | Plate raised (bronze, iron) | Round | Open | yes | Flare | None | Knob | 1.0–2.0 | 0.0004 | S/W |
| Galea | Plate raised (bronze, iron) | Round | Open | yes | Guard | Peak | None, Crest | 1.5–2.5 | 0.0537 (inflated: biology) | S/W |
| Mask Helmet (Roman cavalry) | Plate raised | Round | Mask | – | Guard, None | None | None, Crest | 1.5–2.5 | 0.0001 | S/E |
| Boar's Tusk Helmet | Leather (rows of tusk plates) | Conical | Open | yes | None | None | Knob, Crest | 1.0–1.5 | 0.0001 ✂ Homeric | S/E |
| Leather Cap / Leather Helmet | Leather | Round, Conical | Open | yes/no | None, Flare | None, Peak | None | 0.4–0.8 | | E |

Cut (no model of their own; the generic rows make them): Kegelhelm, Konos, Negau (rounded bowls with a small brim: generic Bronze War Hat), Enclosed helmet (an early great helm), Brocas helm (a jousting helm like the frog-mouth), Venetian helmet (the barbute's Italian name), Pikeman's pot (kettle hat with a comb), Kokyet, Mahiole (wicker and feathers: no materials for it), Pickelhaube and anything later (modern).

### Fantasy

| Name | How | Note |
| --- | --- | --- |
| Horned Helm / Helmet | Ornament Horns on any skull except Onion | 0.0132; Bronze Age Veksø helmets and Henry VIII's horned mask are the only real ones (S) |
| Winged Helm / Helmet | Ornament Wings | 0.0081; 19th century invention (S, Winged helmet) |
| Full Helm | another generic word for Helm | 0.0038; suggest as the closed-face generic instead of "Helm"? |
| Crowned Helm | Ornament Crown | 0.0014 |
| Skull Helm | Visor Grotesque, as a skull | 0.0013 |
| Dragon Helm | Ornament Dragon (a crest of a dragon's head and wings) | 0.0006 |
| Tarnhelm | Wagner's helm of invisibility: a magic item, not a shape | 0.0013; later, with magic |

## 5. Materials and finishes

| Construction | Materials (new in bold) |
| --- | --- |
| Plate | Iron, **Steel**, **Bronze** (ancient rows only, or any at low weight), Copper (rare, early), fantasy **Mithril** (question) |
| Lamellar | Iron, **Steel**, **Hardened Leather**, **Horn** |
| Leather | **Leather**, **Hardened Leather** |
| Mail coif | Iron, **Steel**, **Bronze** (rare), fantasy **Mithril** |
| Padded cap, coif | **Linen**, **Wool**, **Cotton** (silk as a rare cover) |

Decisions to record (new materials): category **Leather** (Leather, Hardened Leather); category **Textile** (Linen, Wool, Cotton); Metal gains **Steel** and **Bronze**; Bone gains **Horn** and **Tusk** (boar's tusk plates; or tusk as Bone). Same list as the parent's, nothing extra except Tusk.

Not materials, only colours on the model (fixed by the form, not rolled separately): horsehair crests, feather plumes, cords and lacing (Japanese odoshi), linings.

**Finish** (proposal for all armour, so a set shares one; in `shared::armour`): Bright (polished), Blackened, Russet (brown), Blued, Painted, Lacquered (Japanese, leather), Gilt, Etched (bands), Tinned. Bronze shows its own colour. Weighted by quality: gilt and etched at Rare and above.

## 6. Weights (worked out)

Weight = shell area × thickness × density + fittings. Shell area comes from the same millimetre figures the model is built from (bowl, faceplate, brim, back), so shared code and the model agree. Calibrated to: great helm 1.5–3 kg (S, reenactment copies), and E figures from museum pieces: bascinet 2–2.5 kg, armet about 3.5 kg, close helmet 3–4.5 kg, frog-mouth helm 8–10 kg, kettle hat about 2 kg, Corinthian about 1.5–2 kg.

| Thing | Thickness | Note |
| --- | --- | --- |
| Iron or steel plate | 1.5 mm (2.5 mm faceplate of a great helm; 4 mm front of a frog-mouth) | W: a 0.12 m² bowl at 1.5 mm is 1.4 kg |
| Bronze plate | 1.0 mm (thicker at the brow and rim) | W |
| Leather | 4 mm (density about 1.0) | E |
| Lamellar | plate × 1.6 for overlap, plus lacing | W |
| Mail | about 6 kg per m² | W from a mail shirt at 9–11 kg (S, parent) over about 1.6 m²; a coif of 0.35 m² is 2.1 kg |
| Padded | 0.25 kg | E |
| Lining and straps | +0.2 kg on every helmet | E |

## 7. Size and fit

- Inventory shapes: Arming cap 2×1, Coif 2×2, Skull cap 2×1, helmets 2×2, wide brims (kettle hat, jingasa, Boeotian) 3×2, Frog-mouth and Great Bascinet 2×3 (E: owner may prefer all 2×2).
- Worn sizes: helmets `(0, 0)` and not adjustable (a helmet must fit), arming cap and coif `(1, 0)`, adjustable for a padded cap.
- Built on the frame's head (`head` half sizes, `chin` to `height`), with 12 mm of lining between the head and the shell, so every kind with a frame gets a helmet that fits it (a kobold's helmet is wider for its big head). An item's size scales it on top, as for blades.

## 8. Visual sheet (for the models)

Frame for a human: head half sizes 78 across, 100 deep; chin 1530, crown 1750, so the skull centre (`Joint::Skull`) is at 1640. Brow line at about 1665, eyes 1640, nose tip 1600, mouth 1570, ear centre 1635 at the sides, nape 1560.

Every helmet shell is the head's ellipsoid grown by lining (12) and thickness, cut by an edge line that goes round the head (its height at each angle round): that one edge decides the face opening, the cheeks, the nape. Then parts are added in place:

| Part | Built as | Placed |
| --- | --- | --- |
| Bowl | ellipsoid stretched upwards by skull shape: Round 1.0, Low 0.8, Conical/Ogival to a point 60–90 mm above the crown (ogival's point 30 mm behind the middle), Flat a cylinder 180 mm high with a flat or slightly domed top, Onion bulging 15% at mid height then drawn to a spike, Forward a cone leaning 60 mm forward | `Joint::Skull` |
| Face edge | Open: cut at brow across the front, down past the ears; Opening: T or Y slot 30 mm wide, eye bar at 1645; Faceplate: closed to 1530 with two slits 25 mm at 1645 and a cross of breaths | front of the bowl |
| Nasal | strip 25 mm wide, 3 mm proud, from the brow to 1595 | front centre |
| Spectacles | two rings round the eyes from the brow to 1620, with a nasal | front |
| Cheeks | plates from the ears to 1540, 60 mm wide, curving round the jaw | each side |
| Visor | a second shell 25–40 mm proud of the face, pivoting at the temples (1660); Pointed pulls forward 90 mm to a snout; Bellows has three horizontal ridges and a slit; Klapp covers face only below the brow from a brow hinge | front |
| Back | Flare: lip 15 mm turned out; Guard: plate 50 mm out at 1570; Tail: bowl continuing back and down 120 mm past the nape; Lames: 3–5 strips 35 mm high, each wider; Flaps: three soft panels to the shoulders (1440) at back and sides | back |
| Brim | Peak: 50 mm out at the front; Narrow: 30 mm round; Wide: 90–130 mm sloping down 20°; Boat: 70 mm at the sides, rising to points 120 mm above the brim fore and aft | at the brow line |
| Top | Knob 25 mm; Spike 80–150 mm; Stalk 20 mm curled back; Comb 15 mm high front to back; High comb 70 mm; Crest a 30 mm holder and a 60 × 250 mm fan of horsehair, front to back; Plume a tube with a 200 mm feather | on the crown |
| Ornament | Horns two curved tapering tubes 250 mm from the temples; Wings two flat fans 180 mm; Front crest a flat badge 150 mm wide at the brow (kuwagata: two upright prongs); Crown a toothed band at the brow | |
| Mail coif | hood: ellipsoid grown 20 mm, round face opening from brow to chin, skirt flaring to the shoulder line (1420) | |
| Arming cap | close ellipsoid grown 8 mm, quilting lines, chin strap | |

Japanese additions with Kabuto: fukigaeshi (the top lames turned back at the face edge), the tehen (small hole at the crown), lacing colours across the shikoro lames.

## 9. Open questions (for the review artifact)

1. Kinds: is a separate Cap / Coif / Helmet split right, with arming cap and coif on Middle so either goes under a helmet?
2. Mail coif covers Head and Neck: fine as one item across both (then no gorget with it)?
3. Generic for closed helmets: "Helm" or "Full Helm"? ("Helmet" stays for open ones.)
4. Fantasy: horns, wings, crowns and dragons as rare ornaments on any helmet; skull helm as a grotesque visor. Also mithril for mail and plate?
5. Finishes shared by all armour (bright, blackened, russet, blued, painted, lacquered, gilt, etched, tinned): agree, and is gilt Rare+?
6. Inventory sizes: wide-brimmed hats 3×2 and the two Head + Neck helms 2×3, or everything 2×2?
7. Cut list: Kegelhelm, Konos, Negau, Enclosed, Brocas, Venetian, Pikeman's pot, Kokyet, Mahiole (feathers and wicker), and everything modern (Pickelhaube on). Any to keep?

## 10. Owner's answers to the parent (2026-11-03) and what they change here

- **Style** is a set property: plan below. A helmet bought alone has no style (Plain).
- **Mithril** is a Metal (no Fantasy category). Dragon scale and chitin are Bone (not used on the head).
- **Ring mail and studded leather kept:** ring mail coif (rings on leather), studded as a finish of Leather helmets and caps (needs the parent's construction or finish decision).
- **Rattan and cork** under Plant: rattan helmets were real (Chinese rattan-shield troops, Taiwanese and Philippine peoples) but need a woven construction: question on the review.
- **Over layer:** asked whether Head takes Over (crests with mantling, helmet covers, turbans wound round helmets). My recommendation: crests stay on the helmet; the rest later with clothes.
- **Partial cover:** a helmet's fixed neck guard or lames don't make it Neck; only the mail coif, padded coif, frog-mouth helm and great bascinet are Head + Neck.
- **Neck chat agreed** the boundary (2026-11-03). I promised them `pub fn rim(form, frame) -> Vec<[i32; 3]>` in `shared::armour::head`: a helmet's lower edge in mm, round from the front, for aventails.

### Styles on helmets

A style picks which named rows a set rolls, and adds detail to the same shell in the model (so every helmet draws in every style):

| Style | Rows it rolls | Model detail |
| --- | --- | --- |
| Plain (white harness) | any | smooth, rolled edges |
| Gothic | sallet, armet, hounskull, barbute | keel, flutes fanning from the crown, cusped edges, longer sallet tail |
| Milanese | armet, barbute, close helmet | fuller bowl, low comb, brow reinforce, turned edges |
| Maximilian | close helmet, burgonet | close parallel flutes all over, bellows visor, roped edges |
| Greenwich | close helmet, burgonet, armet | tall comb, etched gilt bands, roped edges |
| Kastenbrust | hounskull, great bascinet | tall point, long snout |
| Elven | sallet, burgonet, Attic, Corinthian, kabuto-like | tall, swept point, leaf cheeks, swept crest or wings, thin curled edges, filigree bands |
| Dwarven | great helm, spectacle helm, nasal helm, close helmet | low and broad, thick brow, broad nasal or bearded mask, rivet rows, square edges |

Model hooks this needs: flutes (ridges along meridians of the shell), edge profiles (rolled, roped, cusped, square), proportion factors (height, width), etched bands (darker strips), rivet rows.

## 11. What the model extraction added (2026-11-03)


## 12. Late fetch (93 titles, 16 missing): sourced figures

- Sutton Hoo helmet (Spectacle/Mask helm, iron with tinned bronze sheets): about 2.5 kg (S); crest 25.5 cm front to back (S); "little such cushioning" between head and helmet (S).
- Coppergate helmet: brow band 572 mm round, nose-to-nape band 492.8 mm (S). The human frame's head is about 562 mm round (W, ellipse 78 × 100 half sizes), so the gap is small. **Change:** lining gap 8 mm, not 12 (the Neck chat was told 12 + 1.5: tell them 8 + 1.5 when coding).
- Korean eight-plate helmet (Met): 13 cm high, 24.3 × 21 cm, 1066 g (S); Tibetan eight-plate helmets 21–22 cm high (S). Calibrates the Steppe Helmet and Lamellar Helmet: about 1.1 kg for a low bowl of plates, so plate helmets of 1.5 mm at 1.4 kg for a full bowl agree.
- Late Roman ridge helmet (Deurne): 368 g of silver in its covering (S): a gilt or silver-sheathed finish, not a material.

## 13. Owner's answers to the head review (2026-11-03)

1. Three kinds: yes, more accurately layered: the arming cap is on **Skin** (Head's Skin also holds glasses, eye patches and face masks later), coifs on Middle, helmets on Outer, so cap, coif and helmet stack as they did. When glasses and the like come, Head's Skin will need places of its own; the cap takes the scalp.
2. Mail coif on Head and Neck: yes.
3. Closed helmets are "Helm": yes.
4. Fantasy ornaments and skull helm as planned: yes.
5. No Over layer on Head: Over stays body only.
6. Rattan: yes, add a woven construction for rattan helmets (`Construction::Woven`, Plant: Rattan); studded leather caps and ring-mail coifs in.
7. Finishes shared by all armour, gilt and etched Rare+: yes.
8. Inventory sizes: try the various sizes (wide brims 3×2, Head + Neck helms 2×3) for now.
9. Cut list: agreed.

## 14. Before coding (state of master, 2026-11-03)

- Hands (#109) added `armour::Style` (8 styles), all armour materials and categories, `armour::hands`, and model helpers in `item_art/armour/hands.rs` and `item_art/armour/mod.rs`: reuse them, don't add my own. `armour` is already a folder.
- Torso adds `Construction::RingMail` (ring-mail coif waits for it). I add `Construction::Woven` unless it's on master by then.
- Finish and Dye (FINAL with Torso): `Dye { Natural, Red, Blue, Green, Black, Brown, White, Yellow, Purple }`, `Finish { Bright, Blackened, Russet, Blued, Tinned, Painted(Dye), Lacquered(Dye), Gilt, Etched }`, `rare_only()` Gilt and Etched. Pieces have `finish`, plus `cover: Dye` (cloth) and `lacing: Dye` where they have them. Locked spec: shared notes `finish.md`.
- Owner: every armour item may take up to 3 sockets and gems later; not modelled, nothing may rule it out.
- New materials from Torso's answers: Coir (Plant), Adamantine (Metal): adamantine for dwarven plate helmets too.

## 15. Reference pictures


## 16. More owner rules (Waist review, 2026-11-03), from the parent

- Armour weight grows with size squared: `grams * q * q / 16` (not cubed like blades).
- `armour::Style` also gets regional looks: Japanese, Chinese, Steppe, Classical, Byzantine, IndoPersian; each construction allows only some. Shoulders adds both in its PR: build on them. Head mapping: Japanese → kabuto, jingasa; Chinese and Steppe → steppe helmet, lamellar helmet; Classical → Greek and Roman rows; Byzantine → spangenhelm, ridge helmet, lamellar helmet; IndoPersian → kulah khud, turban helmet, chichak.
