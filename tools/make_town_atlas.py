"""Makes crates/client/assets/town_atlas.png from the AngbandTk tiles, the one
picture a town's roads, gardens, walls, roofs and doors are drawn from. The
order must match `dungeon_map::town` in the client. Run from the repository's
root: python tools/make_town_atlas.py (needs Pillow)."""

import random

from PIL import Image, ImageDraw

TILES = "crates/client/assets/angbandtk/"
OUT = "crates/client/assets/town_atlas.png"
CROPS = "crates/client/assets/field_crops.png"
PLANTS = "crates/client/assets/herb_plants.png"
COLUMNS, ROWS, SIZE = 8, 6, 32


def tile(sheet, row, column):
    name = {"g": "dg_grounds32", "f": "dg_features32", "e": "dg_extra132"}[sheet]
    return Image.open(f"{TILES}{name}_{row:02d}_{column:02d}.png").convert("RGBA")


def tinted(picture, colour):
    """The picture's detail in one colour."""
    grey = picture.convert("L")
    average = sum(grey.tobytes()) / (SIZE * SIZE)
    out = Image.new("RGBA", picture.size)
    for y in range(SIZE):
        for x in range(SIZE):
            scale = grey.getpixel((x, y)) / average
            out.putpixel((x, y), tuple(min(255, int(c * scale)) for c in colour) + (255,))
    return out


def window(pane=(0, 0, 0, 255)):
    """A pane, black unless told otherwise, in a dark wooden frame."""
    out = Image.new("RGBA", (SIZE, SIZE), (70, 45, 25, 255))
    ImageDraw.Draw(out).rectangle((3, 3, SIZE - 4, SIZE - 4), fill=pane)
    return out


def signed(draw_icon):
    """A door with a blank sign, and an icon drawn on it."""
    out = tile("f", 2, 6).copy()
    draw = ImageDraw.Draw(out)
    draw.rectangle((10, 9, 22, 22), fill=(72, 72, 72, 255))
    draw_icon(draw)
    return out


def saw(d):
    d.polygon([(11, 13), (21, 13), (21, 18), (11, 15)], fill=(200, 205, 215, 255))
    d.rectangle((20, 12, 22, 19), fill=(140, 90, 45, 255))
    for x in range(11, 20, 2):
        d.point((x, 17 - (21 - x) // 4), fill=(90, 90, 100, 255))


def scissors(d):
    steel = (205, 210, 220, 255)
    d.line((12, 10, 19, 19), fill=steel)
    d.line((20, 10, 13, 19), fill=steel)
    d.ellipse((10, 18, 14, 22), outline=(200, 60, 60, 255))
    d.ellipse((18, 18, 22, 22), outline=(200, 60, 60, 255))


def hide(d):
    d.polygon(
        [(12, 10), (15, 12), (18, 12), (21, 10), (20, 15), (22, 20), (18, 19), (16, 22),
         (14, 19), (10, 20), (12, 15)],
        fill=(150, 95, 50, 255),
    )
    d.point((16, 15), fill=(90, 55, 30, 255))


def diamond(d):
    d.polygon([(12, 12), (20, 12), (22, 15), (16, 22), (10, 15)], fill=(120, 215, 240, 255))
    d.line((10, 15, 22, 15), fill=(215, 245, 255, 255))
    d.line((16, 22, 14, 15), fill=(70, 160, 200, 255))
    d.line((16, 22, 18, 15), fill=(70, 160, 200, 255))


def furrows():
    """Bare earth in ridges, one every quarter of a cell, so they meet the next
    cell's."""
    shades = [(84, 60, 38), (110, 80, 50), (132, 98, 62), (110, 80, 50)]
    rng = random.Random(1)
    out = Image.new("RGBA", (SIZE, SIZE))
    for y in range(SIZE):
        for x in range(SIZE):
            r, g, b = shades[x // 2 % len(shades)]
            d = rng.randint(-5, 5)
            out.putpixel((x, y), (r + d, g + d, b + d, 255))
    return out


def crop(height, colour):
    """Blades this high from the bottom of a tile, on nothing."""
    rng = random.Random(height)
    out = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    for x in range(SIZE):
        if x % 2 == 1 and rng.random() < 0.5:
            continue
        tall = height - rng.randint(0, height // 3)
        shade = rng.uniform(0.75, 1.1)
        blade = tuple(min(255, int(c * shade)) for c in colour) + (255,)
        for y in range(SIZE - tall, SIZE):
            out.putpixel((x, y), blade)
    return out


# Crops as they grow, standing in a field: `dungeon_map::field` must match.
crops = Image.new("RGBA", (3 * SIZE, SIZE))
for index, (height, colour) in enumerate(
    [(6, (88, 158, 62)), (12, (150, 178, 62)), (18, (222, 196, 72))]
):
    crops.paste(crop(height, colour), (index * SIZE, 0))
crops.save(CROPS)

STEM_GREEN = (70, 130, 60, 255)
STEM_GREY = (172, 170, 160, 255)
GROUND = SIZE - 1


def leafy(d, x, leaf, flower):
    """Big leaves up a stem."""
    d.line((x, GROUND, x, 15), fill=STEM_GREEN)
    for y, side in [(26, -1), (23, 1), (19, -1), (16, 1)]:
        d.ellipse((x + (1 if side > 0 else -6), y - 2, x + (6 if side > 0 else -1), y + 2), fill=leaf)
    d.point((x, 14), fill=flower)


def fern(d, x, leaf, flower):
    """Fronds getting shorter towards the top."""
    d.line((x, GROUND, x, 12), fill=leaf)
    for y in range(28, 12, -3):
        reach = max(2, (y - 10) // 3)
        d.line((x, y, x - reach, y - 2), fill=leaf)
        d.line((x, y, x + reach, y - 2), fill=leaf)
    d.point((x, 11), fill=flower)


def daisy(d, x, leaf, flower):
    """A big flower on a stem."""
    d.line((x, GROUND, x, 15), fill=STEM_GREEN)
    d.line((x, 27, x - 4, 24), fill=leaf, width=2)
    d.line((x, 25, x + 4, 22), fill=leaf, width=2)
    d.ellipse((x - 5, 9, x + 5, 19), fill=flower)
    d.ellipse((x - 1, 13, x + 1, 15), fill=(240, 205, 70, 255))


def spike(d, x, leaf, flower):
    """A spike of little flowers over thin leaves."""
    d.line((x, GROUND, x, 10), fill=STEM_GREEN)
    d.line((x, GROUND, x - 4, 22), fill=leaf)
    d.line((x, GROUND, x + 4, 22), fill=leaf)
    for y in range(10, 21, 2):
        d.line((x - 2, y, x + 2, y), fill=flower)


def bells(d, x, leaf, flower):
    """Hanging flowers from a bent stem, over round leaves."""
    d.line((x, GROUND, x, 13), fill=STEM_GREEN)
    d.line((x, 13, x + 6, 15), fill=STEM_GREEN)
    d.ellipse((x - 6, 23, x, 28), fill=leaf)
    d.ellipse((x, 24, x + 6, 29), fill=leaf)
    for bx in (x + 2, x + 6):
        d.polygon([(bx - 2, 19), (bx + 2, 19), (bx, 15)], fill=flower)


def dome(d, x, cap, spot):
    """A round cap with spots."""
    d.rectangle((x - 1, 20, x + 1, GROUND), fill=STEM_GREY)
    d.pieslice((x - 7, 11, x + 7, 27), 180, 360, fill=cap)
    d.point([(x - 3, 15), (x + 3, 16), (x, 13)], fill=spot)


def flat(d, x, cap, spot):
    """A wide flat cap."""
    d.rectangle((x - 1, 19, x, GROUND), fill=STEM_GREY)
    d.ellipse((x - 8, 15, x + 7, 20), fill=cap)
    d.line((x - 5, 16, x + 4, 16), fill=spot)


def cone(d, x, cap, spot):
    """A pointed hat."""
    d.line((x, GROUND, x, 18), fill=STEM_GREY, width=2)
    d.polygon([(x - 5, 19), (x + 6, 19), (x, 7)], fill=cap)
    d.point((x, 13), fill=spot)


def puffs(d, x, cap, spot):
    """Puffballs with hardly any stem, in a huddle."""
    for px, py, r in [(x - 4, 25, 4), (x + 4, 26, 3), (x, 21, 3)]:
        d.ellipse((px - r, py - r, px + r, py + r), fill=cap)
        d.point((px - 1, py - 1), fill=spot)
    d.rectangle((x - 5, 29, x + 5, GROUND), fill=STEM_GREY)


def inkcap(d, x, cap, spot):
    """A tall thin stem and a narrow bell."""
    d.line((x, GROUND, x, 10), fill=STEM_GREY)
    d.ellipse((x - 3, 5, x + 3, 15), fill=cap)
    d.line((x - 2, 15, x + 2, 15), fill=spot)


def plants():
    """A column per shape, herbs then mushrooms; rows of dots, of stalks, then
    of full grown plants in three colours each: `dungeon_map::herbs` must match."""
    colours = [
        [((62, 140, 70), (225, 225, 240)), ((120, 170, 60), (240, 200, 60)), ((46, 120, 112), (215, 90, 120))],
        [((86, 150, 56), (190, 220, 110)), ((40, 112, 72), (120, 180, 90)), ((132, 158, 72), (90, 150, 110))],
        [((70, 140, 62), (245, 245, 245)), ((96, 150, 80), (235, 150, 60)), ((60, 128, 92), (210, 90, 200))],
        [((110, 150, 110), (130, 90, 200)), ((84, 134, 64), (225, 80, 80)), ((130, 170, 90), (90, 130, 230))],
        [((54, 124, 60), (120, 150, 240)), ((104, 160, 70), (245, 150, 190)), ((70, 136, 100), (250, 230, 120))],
    ]
    caps = [
        [((200, 45, 40), (250, 245, 235)), ((60, 90, 200), (210, 230, 255)), ((236, 150, 40), (255, 235, 180))],
        [((150, 100, 60), (215, 180, 130)), ((120, 60, 160), (215, 180, 235)), ((90, 170, 130), (205, 240, 215))],
        [((230, 200, 70), (150, 110, 40)), ((215, 90, 130), (250, 200, 215)), ((70, 60, 55), (170, 160, 150))],
        [((225, 220, 200), (160, 150, 120)), ((240, 170, 120), (170, 90, 60)), ((170, 210, 90), (100, 140, 50))],
        [((55, 50, 60), (200, 200, 210)), ((90, 200, 215), (30, 110, 130)), ((180, 40, 90), (240, 150, 190))],
    ]
    herb_shapes = [leafy, fern, daisy, spike, bells]
    mushroom_shapes = [dome, flat, cone, puffs, inkcap]
    sheet = Image.new("RGBA", (10 * SIZE, 5 * SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(sheet)

    def at(column, row):
        """Draws on one tile, in the tile's own coordinates."""
        out = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
        return out, ImageDraw.Draw(out), (column * SIZE, row * SIZE)

    for column in range(10):
        mushroom = column >= 5
        rng = random.Random(column + 1)
        # Dots on the ground.
        tile_image, draw, place = at(column, 0)
        greens = [(86, 150, 60), (112, 170, 70), (60, 125, 60)]
        greys = [(170, 170, 165), (140, 140, 138), (195, 195, 188)]
        for _ in range(14):
            x, y = rng.randint(2, 28), rng.randint(2, 28)
            colour = rng.choice(greys if mushroom else greens) + (255,)
            draw.rectangle((x, y, x + 1, y + 1), fill=colour)
        sheet.paste(tile_image, place)
        # Stalks.
        tile_image, draw, place = at(column, 1)
        for x in range(4, 30, 5):
            tall = rng.randint(6, 11)
            stalk = STEM_GREY if mushroom else STEM_GREEN
            draw.line((x, GROUND, x, GROUND - tall), fill=stalk, width=2 if mushroom else 1)
        sheet.paste(tile_image, place)
        # Full grown, in each colour: two to a tile.
        for way in range(3):
            tile_image, draw, place = at(column, 2 + way)
            if mushroom:
                cap, spot = caps[column - 5][way]
                shape = mushroom_shapes[column - 5]
            else:
                cap, spot = colours[column][way]
                shape = herb_shapes[column]
            # Where the planes of a `#` cross: a quarter and three quarters across.
            for x in (8, 24):
                shape(draw, x, cap + (255,), spot + (255,))
            sheet.paste(tile_image, place)
    sheet.save(PLANTS)


plants()

atlas = [
    # Roads, poorest first (the order of the cells).
    tile("g", 1, 5),
    tile("g", 11, 3),
    tile("e", 12, 3),
    tile("g", 3, 3),
    tile("e", 11, 6),
    tile("e", 11, 0),
    # Garden, hedge.
    tile("g", 10, 3),
    tinted(tile("g", 10, 3), (45, 105, 40)),
    # Walls: wattle, timber, plaster, brick, stone.
    tile("g", 3, 1),
    tile("e", 6, 6),
    tinted(tile("g", 1, 3), (215, 200, 170)),
    tile("e", 13, 3),
    tile("e", 11, 3),
    # Roofs: mud and sticks, thatch, wooden shingles, clay tiles, slate tiles.
    tinted(tile("g", 3, 2), (95, 70, 45)),
    tile("g", 18, 0),
    tinted(tile("e", 12, 3), (120, 85, 50)),
    tinted(tile("e", 13, 3), (175, 75, 45)),
    tinted(tile("e", 11, 3), (70, 75, 90)),
    # A blind window, dark and with a light on behind it.
    window(),
    window((255, 240, 190, 255)),
    # Doors, in the order of `BuildingKind`.
    tile("f", 3, 1),  # House: a plain door
    tile("f", 9, 0),  # Shrine
    tile("f", 8, 7),  # Inn
    tile("f", 7, 5),  # Tavern
    tile("f", 7, 8),  # Blacksmith
    tile("f", 8, 6),  # Herbalist
    signed(saw),  # Carpenter
    signed(scissors),  # Tailor
    signed(hide),  # Leatherworker
    tile("f", 7, 6),  # Fletcher
    tile("f", 8, 2),  # Church
    tile("f", 2, 5),  # Weaponsmith
    tile("f", 2, 4),  # Armoursmith
    tile("f", 2, 7),  # Potion shop
    tile("f", 7, 3),  # Training hall
    tile("f", 8, 8),  # Scroll shop
    tile("f", 8, 4),  # Adventurers' guild
    tile("f", 2, 8),  # Jeweller
    tile("f", 3, 2),  # Library
    tile("f", 8, 5),  # Bank
    signed(diamond),  # Gem cutter
    tile("f", 8, 0),  # Black market
    tile("f", 2, 6),  # Cathedral
    # A farm's fields: ploughed with furrows running north to south, grass, and
    # the wood of fences.
    furrows(),
    tinted(tile("g", 10, 3), (70, 130, 50)),
    tile("e", 12, 3),
]

sheet = Image.new("RGBA", (COLUMNS * SIZE, ROWS * SIZE))
for index, picture in enumerate(atlas):
    sheet.paste(picture, (index % COLUMNS * SIZE, index // COLUMNS * SIZE))
sheet.save(OUT)
print(f"{len(atlas)} tiles")
