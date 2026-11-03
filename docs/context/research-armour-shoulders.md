# Research: armour for the shoulders

Done 2026-11-03 by the Shoulders chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/4dBVz9nrXiFAhJvGJZW88E); their answers are recorded below and in `decisions.md`. Code: `crates/shared/src/armour/shoulders.rs`, `crates/client/src/item_art/armour/shoulders.rs`.

Evidence: **S** sourced (Wikipedia quote), **W** worked out, **E** estimate or general knowledge.

## Sources

The shoulder articles are short (Pauldron, Spaulder, Ailette, Besagew), so they and the shoulder sentences of about 200 longer armour articles were read directly; no local model was needed. Pictures: Met open-access photographs on Wikimedia Commons (pauldrons, sode, Dendra).

- Spaulder (S): one plate over the shoulder with lames below, 14th century; leaves the armpit open, covered by besagews or left to the mail.
- Pauldron (S): grew from the spaulder in the 15th century; larger, a dome (the cop) with lames to the arm and neck, covering the armpit and parts of breast and back. Italian pairs often uneven, the right cut away for the lance rest. Fantasy art gives them spikes.
- Besagew, Rondel (S): plain, slightly domed discs at the armpit, hung from the pauldron or gorget; mostly Gothic. Gorget (S): pauldrons were pinned or strapped to it.
- Plate armour, White armour (S): gardbraces with Italian and French pauldrons. Greenwich armour (S): rounded pauldrons with blued or gilt bands along each lame.
- Ailette (S): late 13th to mid 14th century, thick, flat, nearly rectangular boards of hardened leather or wood (sometimes plate), tied on, usually heraldic.
- Lorica segmentata, Manica (S): curved shoulder lames; a manica's strips 25 to 30 mm wide, 0.35 to 0.5 mm thick. Lorica hamata (S): a mail doubling over the shoulders, hooked on, reaching the upper arm under Augustus. Spaulder (S): Roman officers with pteruges below spaulders.
- Linothorax (S): a top piece over the shoulders tied down on the chest. Dendra panoply (S): large bronze shoulder guards over the cuirass.
- Japanese armour, Karuta, Kusari (S): sode of iron or leather plates, of karuta and of mail. Korean armour (S): shoulder guards over the upper arm. Mirror armour (S): naplechniki. Laminar armour (S): Chukchi and Koryak wing pauldrons of leather and wood, used as a shield.
- Ngram 2000-2019 (per million): pauldron 0.013, spaulder 0.0015, ailette 0.0018, besagew 0.0005, gardbrace 0.0002, haute-piece and naplechniki 0. Kept anyway where a grid or the owner needs them.

## Kinds and names

Each item is a pair, Outer on Shoulders (item kind `ShoulderArmour`); its form says its kind, name, cells and weight. Wings and hanging guards reach over chest, back and upper arm but count only as Shoulders, so Arms stays free.

| Kind | Constructions | Specific names | Cells |
| --- | --- | --- | --- |
| Pauldrons | plate, leather, scale, mail and plate | Great, Spiked, Skull, Studded Leather, Leather, Scale, Plated Mail, Gothic Pauldrons | 3 × 2 |
| Spaulders | plate, leather, laminar | Dendra Shoulder Guards (bronze plate), Lorica Shoulder Guards (laminar), Leather Spaulders | 2 × 2 |
| Shoulder guards | lamellar, scale, laminar, plates in cloth, leather, mail | Ko-, Chū-, Ō-sode (Japanese, by size); Lamellar, Scale, Brigandine, Ring Mail, Leather Shoulder Guards | 2 × 1 small, else 2 × 2 |
| Shoulder cape | mail, scale (round); padded, leather (yoke); leather with ring mail (round) | Mail Doubling, Scale Doubling, Ring Mail Cape, Linothorax Yoke | 2 × 2, yoke 2 × 1 |
| Shoulder discs | mail and plate, plate, mirror | Naplechniki, Mirror Shoulder Plates | 2 × 1 |
| Ailettes | leather, plate (steel, iron or wood) | | 2 × 1 |
| Wing | laminar (wood or hardened leather) | Chukchi Wing | 2 × 3 |

## What is rolled

- Pauldrons: cut (matched, great left, lance cut: uneven only in plate), wings (small, large: large only rigid), 3 to 6 lames (rigid) or none (hanging), 0 to 2 upper lames, haute-piece (none, 25 mm, 50 mm), gardbrace, besagews (small wings only); all plate-only parts as marked.
- Spaulders: 2 to 5 lames (none for Dendra), besagews (plate, leather), 0 to 2 rows of pteruges (laminar, leather).
- Guards: size (small 200 × 200 mm, 3 to 5 rows; medium 260 × 280, 5 to 6; large 330 × 360, 6 to 7), shape (straight, tapered, flared), lacing (dense, spaced: lamellar and laminar), Japanese or eastern.
- Cape: cut (round, yoke) by construction, length (short, long). Discs: 110 to 160 mm in 10 mm steps, boss. Ailettes: shape (rectangle, rounded, shield, lozenge), size, heraldry: two tinctures, metal on colour, and a charge (plain, cross, saltire, pale, fess, bend, chevron).
- Adornment, on rigid tops whatever the style: spikes (1, 3, 5), horns, beast head (guards and plate pauldrons), skull. Leather finish: plain, tooled, studded, ring mail (guards and round capes).
- Style comes from a set, or is rolled, and says which constructions are allowed; Gothic and Kastenbrust lean to spaulders with besagews, Milanese to large and uneven pauldrons. Regional styles: Japanese guards are sode; Classical gives Dendra and Roman spaulders, doublings and the linothorax yoke; ailettes are European or fantasy; the wing Steppe.
- Materials per construction as for the hands, with Bronze for Dendra spaulders and Wood for ailettes and the wing; silk or leather lacing; linen, wool or silk over plates in cloth; brass trim on plate and mail.

## Weights (pair, ordinary size)

Worked out (W) from area × thickness × density: plate 1.2 to 1.6 mm steel, hardened leather 4 mm, mail about 7 kg/m² from a 9 to 11 kg shirt (S). Steel pauldrons 2.5 to 3.4 kg (more with extras), spaulders 1.5 to 1.8 kg, Roman laminar 0.6 kg, leather pauldrons about 1.1 kg, large lamellar guards 2.8 kg, a long mail doubling 2.1 kg, ailettes 0.3 kg in leather, discs with mail about 1 kg, the wing about 3.3 kg. Other materials scale by density, and a size by its square (owner's decision).

## Dropped or left for later

- Separate padded shoulder pads: the Gambeson covers Shoulders instead (owner).
- Parchment ailettes; velvet and rawhide (not among the approved materials yet).
- Gem sockets: up to 3 on any armour (owner), shown on the item's frame later, not modelled.

## Owner's answers (2026-11-03)

One item per pair, uneven pairs allowed. Adornments stay, with skulls added; more can come later. Ailettes stay with plain heraldry. The wing stays. The Gambeson covers Shoulders. Japanese, Roman, Russian and Bronze Age names, generic names otherwise.
