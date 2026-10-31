"""Makes crates/client/assets/town_atlas.png from the AngbandTk tiles, the one
picture a town's roads, gardens, walls, roofs and doors are drawn from. The
order must match `dungeon_map::town` in the client. Run from the repository's
root: python tools/make_town_atlas.py (needs Pillow)."""

from PIL import Image, ImageDraw

TILES = "crates/client/assets/angbandtk/"
OUT = "crates/client/assets/town_atlas.png"
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


def window():
    """A black pane in a dark wooden frame."""
    out = Image.new("RGBA", (SIZE, SIZE), (70, 45, 25, 255))
    ImageDraw.Draw(out).rectangle((3, 3, SIZE - 4, SIZE - 4), fill=(0, 0, 0, 255))
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
    # A blind window, and one spare.
    window(),
    window(),
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
]

sheet = Image.new("RGBA", (COLUMNS * SIZE, ROWS * SIZE))
for index, picture in enumerate(atlas):
    sheet.paste(picture, (index % COLUMNS * SIZE, index // COLUMNS * SIZE))
sheet.save(OUT)
print(f"{len(atlas)} tiles")
