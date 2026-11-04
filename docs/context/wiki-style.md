# Wiki style: how the players' pages are written

The rules every page of the players' wiki (`docs/`, outside `docs/context/`) keeps to, first set down for the armour pages. Other tasks use them and add to them. **Each rule is checked off and signed off by the owner**; a task that relies on them doesn't start until they are. A new rule goes in unticked until the owner ticks it.

Signed off by the owner: _not yet_.

## Where pages go

- [ ] A subject with several pages gets its own folder (`docs/armour/`) with an `index.md` overview, which has `has_children: true`, and one page per part, each with `parent:` set to the overview's title.
- [ ] Front matter: `title`, `nav_order`, and `parent` / `has_children` where they apply. A page's place in the menu follows how a player would come to it, not when it was written.
- [ ] File names are lower case words joined by hyphens (`full-sets.md`); titles are plain words ("Torso armour").

## What goes in

- [ ] What the game makes now: the kinds of thing, what they are called, what they look like in each style, what they can be made of, and roughly what they weigh.
- [ ] Rules a player feels: which body place and layer a thing takes, what clashes with what, what goes over or under what.
- [ ] Names as the game shows them, in the same spelling and capitals, with a plain description beside any unfamiliar one ("Kusazuri: hanging panels round the hips").
- [ ] Weights as typical figures for an ordinary-sized piece in its usual material, rounded (about 1.2 kg), with a line saying that size and material change them.

## What stays out

- [ ] Questions, decisions, who decided what, and research notes: those live in `docs/context/`.
- [ ] Evidence marks, sources and article names, Ngram figures.
- [ ] Pictures and screenshots, while the game changes quickly.
- [ ] Code names (`TorsoForm`, `Construction::PlatesInCloth`), file paths and numbers only code needs (roll weights, millimetres on the body frame).
- [ ] Plans and things not made yet, except one line where a player would otherwise wonder ("Gem sockets are planned").

## How each page is laid out

- [ ] One template for every page of a subject. For armour: a short introduction (what the location covers, its layer, what goes under and over it), then **Kinds** (a table), **Styles** (a table), **Materials**, **Weights**. A heading with nothing to say is left out rather than left empty.
- [ ] Tables have a first column naming the thing and one row per thing; the same columns in the same order on every page of a subject.
- [ ] The overview page explains what the subject's pages share once (layers, styles, constructions, materials), and the other pages link to it rather than repeat it.

## How it reads

- [ ] Plain British English, as the rest of the site: short sentences, everyday words, no jargon a player wouldn't know without explaining it.
- [ ] Addressed to the player ("you can wear a surcoat over it"), in the present tense, about what is true now.
- [ ] Links between pages are relative (`[Sets](sets)`), and every page links back to its overview.
