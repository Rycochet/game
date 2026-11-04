# Research: armour for the neck

Done 2026-11-03 by the Neck chat of the armour job, following `item-research-method`. The owner reviewed it on a private artifact (https://claude.ai/artifact/FVKs5art2jjtJod9Aextxr) and said yes to all seven questions; the answers are below and in `decisions.md`. Code: `crates/shared/src/armour/neck/`, `crates/client/src/item_art/armour/neck/`.

Evidence: **S** sourced (Wikipedia quote), **W** worked out, **E** estimate or general knowledge.

## Sources

The neck articles (Gorget, Bevor, Aventail, Standard (mail collar), Kulah khud, Japanese armour, Sallet, Bascinet, Armet, Close helmet, Dendra panoply, Chinese and Korean armour) are short and give no measurements or weights at all, so they were read by hand, with no local model. Ngram 2000-2019, per billion words: gorget 89, bevor 21, aventail 4.3, camail 3.2, colletin 2.0, padded collar 1.5, standard (as "mail standard") 1.4, nodowa 0.3; pisan is swamped by Pisa. Pictures came from Wikimedia Commons (the Met, V&A, Skokloster, Cleveland, DHM) and are credited on the review page.

- Gorget (S): a padded textile neck guard in the 12th-13th centuries, often over mail; by about 1400 a standing plate collar over the aventail, with room to turn the head; in the early 16th century a high collar of articulated lames, front and back halves hinged at the side, covering neck, nape, shoulders and upper chest, worn under the cuirass and holding the pauldrons on pins or straps; in the 17th century a low one-piece collar with larger plates, worn under the breastplate or over a buff coat or clothes, parade pieces etched, gilded or enamelled; in the 18th century only a small crescent on a chain, an officer's badge.
- Bevor (S): usually one plate over chin and throat, filling the gap between helmet and breastplate, worn with the sallet over the cuirass with a mail standard under it; "scoop-shaped", from the upper chest to just below the nose (Sallet). An armet's wrapper had gorget lames over the throat (Gorget).
- Aventail or camail (S): a mail curtain from a helmet's lower edge, covering at least the back and sides of the neck, often the face, throat and shoulders; early ones closed over the lower face; a ventail flap laced over the mouth; from the 1320s laced to the bascinet over pierced staples (vervelles); brass or bronze edging rings, or zig-zag (vandyked) edges.
- Standard or pizaine (S): a mail collar, 14th-16th centuries, standing upright by thicker rings, usually over the shoulders; worn under an aventail, gorget or bevor, or alone, especially by archers; brass edging or vandyked edges.
- Nodowa (S): the Japanese gorget, tied on alone or part of the face mask; lames of lacquered leather or iron, solid or laced scales, joined by silk laces. Tate-eri: a small padded pillow with a standing armoured collar lined with kikko; guruwa: a throat and neck guard (Japanese armour).
- Kulah khud (S): an aventail of iron and brass, or brass and copper rings, over the neck, shoulders and temples, sometimes over the eyes and nose; often vandyked.
- Dendra panoply (S): a deep bronze neck guard; high bronze collars were typical of Near Eastern armour. Chinese helmets had leather neck and throat guards; Korean ones mail or lamellar (S).
- Bishop's mantle (E): a short mail cape over the neck and shoulders, early 16th-century German.

## What was planned and made

- Five kinds, each its own item kind: Collar (Middle: padded, mail standard, the Japanese kikko guruwa), Gorget (Outer: plate, leather, scale, and laminar or lamellar nodowa), Bevor (Outer: plate or leather), Aventail (Outer: mail, ring mail, mail and plate, lamellar, scale, quilted, leather), Mantle (Outer on Neck and Shoulders: mail or scale).
- Gorgets cover the collar alone, a bib (nodowa), plates over chest and back, or plates and shoulder lames (colletin); a tall collar of 2-4 lames (16th century, under the breastplate) or a low rim (17th century, over it).
- Bevors reach the mouth or the chin, with 0-3 lames below; pointed with a ridge in Gothic and sometimes elven.
- Aventails are open at the front, round the chin, closed to the nose (Indo-Persian especially), or with a ventail; aventails and mail collars may reach over the shoulders. Mail hems straight or vandyked, scales and lamellae scalloped; brass edging, or Indo-Persian patterned bands.
- Names: Standard, Guruwa, Colletin, Nodowa, Bishop's Mantle; otherwise Padded/Mail/Ring Mail/Plated Collar, Plate/Leather/Scale Collar, Gorget, Leather/Scale/Lamellar Gorget, Bevor, Leather Bevor, Aventail (Ring Mail, Lamellar, Scale, Quilted, Leather, Plated), Scale Mantle. "Spiked", "Studded" and unusual materials go first. Aventail rather than camail and Standard rather than pisan, by Ngram and the article titles.
- Weights (W, area × per-area weight × density, checked against the usual museum figures, E): a steel gorget with plates about 1.2 kg, a colletin about 1.7, a bevor about 1.1, a mail aventail round the chin about 1.2 (as a cape about 1.7), a standard about 0.7, a bishop's mantle about 2, a padded collar about 0.2.

## Gaps and decisions

- No measurements in the sources: worked out from the model's areas, as the other locations.
- Style details on neck pieces: Maximilian flutes and Greenwich etched bands are drawn on plate gorgets; the other styles change constructions, materials and the bevor's point. More style detail can follow with sets.
- Left out: the officer's crescent gorget, European brigandine collars, the wrapper and mentonnière as names (a bevor with lames), torcs and neck rings (jewellery), wolf collars (dogs, later). The mengu's throat flap (yodarekake) and the kabuto's shikoro stay with the helmet.
