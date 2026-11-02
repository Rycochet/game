# Research: bladed weapons

What the tables in `crates/shared/src/blade/` came from, reviewed by the owner on 2026-11-02. Not kept up to date with the code: the code wins. Tweaks to sizes and weights need no new research.

## Method

Wikipedia's Edged and bladed weapons and Polearms categories gave 887 articles (no deaths, attacks, people, fiction, bayonets, kitchen knives, scythes or sickles). Titles were sorted by hand to 176 weapon types, ranked with the Google Books Ngram Viewer (`en-2019`, mean of 2000 to 2019) and cut to about 5 to 10 a family. `qwen3:8b` and `llama3.1:8b` then read 98 articles, quoting what they found, and a script dropped quotes not in the article. Their measurements were read and corrected by hand. Their judgement was poor, so edges, curve and hands come from clear statements in the articles and general knowledge.

## Reading the table

Sizes in cm. A plain number comes from a checked quote, `~` is an estimate, `=` is worked out from other sourced numbers. Popularity is uses per million words. `*` is inflated by other meanings of the word (stiletto shoes, lance corporal). Jian and dao couldn't be measured, as they swamp results as ordinary words. `†` is below the cut, kept because a cell or the owner's example needs it.

The articles rarely gave handle length (18 of 98), blade width (8), weight (22) or spear head size (0 of 11). The owner accepted estimates for these.

## Owner's decisions

- The assegai stays, for its flavour, so a short blade may go on a haft-length handle.
- The athame is in, as it will have uses with magic. The kirpan, an article of faith, is out.
- Viking sword is out. Scythes and sickles are out, as tools for later; all axes are in.
- Every extra property found in the articles is in: guard and pommel styles, cross-section, fuller, ricasso and hooks, langets, back of the head, barbs and tines, more kinds of point.

## Names

### Knives

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knife |  | ~8–30 | ~8–15 |  | Single | Straight | One | Generic |
| Machete | 0.53 | 30–66 | ~10–15 |  | Single | Straight | One | Wide, heavy chopping blade |
| Hunting knife | 0.16 | ~10–20 | ~10–13 |  | Single | Slight | One |  |
| Bowie knife | 0.09 | 24–30 | ~10–15 | 30–46 | Single | Slight | One | Clip point, large guard |
| Parang | 0.05 | 25–91 | ~10–15 |  | Single | Slight | One | Broad at the tip |
| Throwing knife | 0.03 | ~15–30 | ~5–12 |  | Double | Straight | One | Often no real handle |
| Kukri | 0.03 | 26–38 | =10–15 | 40–45 | Single | Inward | One | Strongly bent, widens at the tip |
| Combat knife | 0.02 | ~15–20 | ~10–13 |  | Single | Straight | One |  |
| Seax | 0.01 | ~8–75 | ~10–20 |  | Single | Straight | One | Long forms from 50 cm |
| Fighting knife | 0.01 | ~15–25 | ~10–13 |  | Double | Straight | One |  |

### Daggers

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Dagger | 3.17 | 18–30 | ~10–13 |  | Double | Straight | One | Generic |
| Tanto* | 0.48 | 15–30 | ~10–15 |  | Single | Slight | One | Blade length only in source |
| Stiletto* | 0.32 | ~20–30 | ~10 |  | Point only | Straight | One | Thin thrusting blade |
| Dirk | 0.15 | ~20–35 | ~10–13 |  | Single | Straight | One |  |
| Kris | 0.08 | 38–127 | ~8–12 |  | Double | Wavy | One | Asymmetric blade, angled handle. Very wide range: I'd cap it |
| Poignard | 0.08 | ~15–30 | ~10–13 |  | Double | Straight | One | Another word for dagger: slender |
| Athame | 0.06 | ~10–20 | ~10–13 |  | Double | Straight | One | Ritual knife with a dark handle; for magic later |
| Kunai | 0.01 | 20–30 | ~10–15 |  | Double | Straight | One | Leaf blade, ring pommel |
| Rondel dagger | 0.01 | 30+ | =10–21 | to 51 | Double | Straight | One | Disc guard and pommel |
| Pugio | 0.01 | 18–28 | 10–13 |  | Double | Straight | One | Leaf blade, 5 cm wide |
| Parrying dagger | 0.01 | =35–40 | ~10–13 | 48 | Double | Straight | One | Large guard, for the off hand |
| Misericorde | 0.01 | ~20–30 | ~10 |  | Point only | Straight | One | Thin blade |

### Short swords

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Short sword | 0.15 | ~35–60 | ~10–15 |  | Any | Any | One | Generic |
| Cutlass* | 0.23 | 61–76 | =10–15 | 71–81 | Single | Slight | One | Cup or shell guard |
| Gladius | 0.07 | 60–68 | ~15 | 60–85 | Double | Straight | One | Largest form sourced; others shorter. 5 cm wide |
| Falcata | 0.03 | ~35–55 | ~10–13 |  | Single | Inward | One | Hooked handle |
| Wakizashi | 0.01 | 30–60 | ~15–25 |  | Single | Slight | One |  |
| Yataghan | 0.01 | 60 | =15 | 75 | Single | Inward | One | No guard, flared pommel |
| Kopis | 0.01 | 48–65 | ~10–13 |  | Single | Inward | One |  |
| Xiphos | 0.004 | 45–60 | ~10–13 |  | Double | Straight | One | Leaf blade |
| Khopesh | 0.004 | =35–45 | ~12–15 | 50–60 | Single | Strong | One | Sickle-shaped blade, but used as a sword |

### Swords

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Sword | 20.29 | ~60–100 | ~10–20 |  | Any | Any | One | Generic |
| Sabre* | 1.03 | ~70–90 | ~12–15 |  | Single | Slight to strong | One | Hand guard |
| Rapier* | 0.31 | 100–130 | ~12–15 |  | Double | Straight | One | 2.5 cm wide, complex guard |
| Scimitar | 0.21 | 75–90 | ~12–15 |  | Single | Strong | One |  |
| Broadsword | 0.16 | =75–90 | ~12–15 | 90–105 | Double | Straight | One | Basket hilt, 3.4 cm wide |
| Katana | 0.11 | 60–80 | ~25–30 |  | Single | Slight | Two | Round or square guard |
| Falchion | 0.04 | =70–80 | ~12–15 | 81–97 | Single | Slight | One | Heavy tip |
| Small sword | 0.03 | 60–85 | ~10–13 |  | Point only | Straight | One | Light, shell guard |
| Tulwar | 0.02 | ~70–85 | ~10–13 |  | Single | Strong | One | Disc pommel |
| Spatha | 0.01 | 60–75 | 18–20 |  | Double | Straight | One |  |
| Backsword | 0.005 | ~75–90 | ~12–15 |  | Single | Straight | One | Hand guard |
| Arming sword† | 0.003 | 69–81 | =10–20 | 90–105 | Double | Straight | One | Cross guard |
| Shamshir† | 0.003 | ~75–90 | ~10–13 |  | Single | Strong | One | Slim, strongly curved |
| Jian | n/a | 45–80 | ~15–25 |  | Double | Straight | One or two | Tassel on the pommel |
| Dao | n/a | =65–90 | ~15–20 | 85–114 | Single | Slight | One |  |

### Long swords

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Longsword | 0.17 | 80–110 | 20–30 | 100–140 | Double | Straight | One or two | Generic |
| Claymore | 0.08 | 107 | 33 | 120–140 | Double | Straight | Two | Guard angled to the blade |
| Greatsword | 0.08 | =100–150 | ~30–50 | to 213 | Double | Straight | Two | Generic |
| Bastard sword† | 0.006 | ~75–95 | ~20–28 |  | Double | Straight | One or two | Shares an article with longsword; shorter here |
| Odachi† | 0.001 | ~90–120 | ~30–40 |  | Single | Slight | Two | Sourced record of 227 is a shrine piece |
| Zweihänder† | 0.0002 | =100–150 | ~30–50 | to 213 | Double | Straight or wavy | Two | Greatsword with ricasso and parrying hooks |

### Axes

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Axe | 3.40 | ~8–20 | ~30–90 |  | Single |  | One or two | Generic. One axe blade |
| Hatchet | 0.70 | ~6–12 | ~25–40 |  | Single |  | One | Small |
| Tomahawk | 0.25 | to 10 | to 61 |  | Single |  | One | Spike or pipe at the back |
| Battle axe | 0.07 | ~10–25 | ~40–120 | 30–150 | Single |  | One or two | Generic |
| Broadaxe | 0.01 | ~15–30 | ~50–80 |  | Single |  | One or two | Very wide blade |
| Labrys | 0.007 | ~10–25 | ~60–100 |  | Single |  | Two | Two matching blades, one each side |
| Throwing axe | 0.003 | ~8–12 | ~30–45 |  | Single |  | One |  |
| Francisca | 0.003 | 10 | 40–45 |  | Single |  | One | Arched head at an angle |
| Tabar | 0.002 | ~15–25 | ~60–100 |  | Single |  | One or two | Crescent blade |
| Dane axe† | 0.001 | 20–30 | 90–170 |  | Single |  | Two | Thin crescent blade |

### Spears

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Spear | 4.30 | ~10–30 | =150–220 | 180–240 | Double | Straight | One or two | Generic |
| Lance* | 1.59 | ~8–15 | =250–390 | 250+ | Double | Straight | One | Hand guard. For riding |
| Javelin | 0.40 | ~5–15 | =170–220 | 183–225 | Double | Straight | One | For throwing |
| Harpoon | 0.35 | ~10–20 | ~120–200 |  | Double | Straight | One or two | Barbed head |
| Trident | 0.28 | ~15–25 | ~120–200 |  | Point only | Straight | Two | Three tines |
| Pike* | 0.10 | ~15–25 | =280–680 | 300–700 | Double | Straight | Two |  |
| Assegai | 0.04 | ~20–30 | 61 |  | Double | Straight | One | Stabbing form |
| Pilum | 0.02 | ~60–100 | =100–140 | 200 | Point only | Straight | One | Long iron shank. For throwing |
| Sarissa | 0.02 | ~20–50 | =450–680 | 500–700 | Double | Straight | Two |  |
| Yari | 0.01 | ~15–60 | 100–600 |  | Double | Straight | Two | Some forms have side blades |

### Polearms

| Name | Popularity | Blade or edge | Handle | Overall | Edges | Curve | Hands | What sets it apart |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Polearm | 0.01 | ~30–60 | ~120–250 |  | Any |  | Two | Generic |
| Halberd | 0.11 | ~20–30 | =170–220 | 200–250 | Single |  | Two | Axe blade, opposite spike or hook, spike on top |
| Glaive | 0.03 | 45 | ~170–200 |  | Single | Slight | Two | Blade continues the pole's line |
| Naginata | 0.01 | 30–60 | 120–240 | 205–260 | Single | Slight | Two | Infobox says 85–100, text 30–60: used text |
| Poleaxe | 0.01 | ~15–30 | 150–200 |  | Single |  | Two | Axe or hammer with spike on top |
| Spontoon | 0.007 | ~15–30 | =160–210 | 180–240 | Double | Straight | Two | Leaf blade with side lugs |
| Bill* | 0.006 | ~20–40 | 120–270 |  | Single | Inward | Two | Hook, with spikes |
| Guandao | 0.001 | 30–46 | 91–180 |  | Single | Strong | Two | Heavy blade, 2–10 kg |
| Lochaber axe | 0.001 | 46 | 150–180 |  | Single |  | Two | Long blade with hook on top |
| Voulge | 0.001 | ~30–60 | ~170–250 |  | Single |  | Two | Cleaver-like blade |
| Guisarme | 0.001 | ~20–40 | ~180–250 |  | Single | Inward | Two | Hook with spike |
| Bardiche | 0.0007 | 60+ | to 150 |  | Single | Slight | Two | Very long blade, fixed at its base |
| Nagamaki† | 0.0007 | 60+ | =60+ |  | Single | Slight | Two | Handle about as long as the blade |
