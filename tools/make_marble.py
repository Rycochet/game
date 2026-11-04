"""Makes crates/client/assets/hall/marble.png, the Hall of Worlds' floor: a
slab of pale marble with soft veins, 32 pixels like the other tiles, joining
its neighbours with a faint line at its edge. The game tints it by the Hall's
hue. Run from the repository's root: python tools/make_marble.py (needs
Pillow)."""

import math
import random

from PIL import Image

OUT = "crates/client/assets/hall/marble.png"
SIZE = 32
BASE = (236, 234, 230)
VEIN = (176, 172, 168)
JOINT = (198, 195, 190)


def wave_noise(rng, x, y, waves):
    """Smooth noise that repeats every SIZE pixels both ways: a sum of waves
    with whole numbers of periods across the tile."""
    total = 0.0
    for fx, fy, phase, weight in waves:
        total += weight * math.sin(2 * math.pi * (fx * x + fy * y) / SIZE + phase)
    return total


def main():
    rng = random.Random(7)
    waves = [
        (rng.randint(-3, 3), rng.randint(-3, 3), rng.uniform(0, 6.3), 1 / (i + 1))
        for i in range(8)
    ]
    out = Image.new("RGB", (SIZE, SIZE))
    for y in range(SIZE):
        for x in range(SIZE):
            # Veins run along where a slanted line, bent by the noise, comes
            # near a whole number.
            bent = (x + 2 * y) / SIZE * 2 + 0.35 * wave_noise(rng, x, y, waves)
            vein = max(0.0, 1.0 - abs(bent - round(bent)) * 9.0) ** 2
            cloud = 0.5 + 0.08 * wave_noise(rng, x, y, waves[2:])
            colour = [
                b + (v - b) * vein * 0.75 - 10 * (cloud - 0.5)
                for b, v in zip(BASE, VEIN)
            ]
            if x == 0 or y == 0:
                colour = [(c + j) / 2 for c, j in zip(colour, JOINT)]
            out.putpixel((x, y), tuple(max(0, min(255, int(c))) for c in colour))
    out.save(OUT)


if __name__ == "__main__":
    main()
