/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { Alea } from "../../../../common";
import { getWorld } from "../../../World/World";
import { DigRooms } from "./DigRooms";

export const useDungeon = (x: number, y: number, z: number) => {
    const { seed } = getWorld();
    const rng = Alea(seed, x, y, z);

    const width = rng.uint(80, 30);
    const height = rng.uint(80, 30);
    const startX = rng.uint(width - 5, 5);
    const startY = rng.uint(height - 5, 5);

    const grid1: number[][] = [];
    const grid2: number[][] = [];

    for (y = 0; y < height; y++) {
        grid1[y] = [];
        grid2[y] = [];
        for (x = 0; x < width; x++) {
            grid1[y][x] =
                x < 5 || x >= width - 5 || y < 5 || y >= height - 5 ? 1
                : rng() > 0.1 ? 1
                : 0;
            grid2[y][x] = 0;
        }
    }
    for (y = -1; y < 2; y++) {
        for (x = -1; x < 2; x++) {
            grid1[startY + y][startX + x] = 0;
        }
    }
    // Iteration 1

    for (y = 0; y < height; y++) {
        for (x = 0; x < width; x++) {
            // Not including centre tile
            const count =
                !x || !y || x === width - 1 || y === height - 1 ?
                    8
                :   grid1[y - 1][x - 1] +
                    grid1[y - 1][x] +
                    grid1[y - 1][x + 1] +
                    grid1[y][x - 1] +
                    grid1[y][x + 1] +
                    grid1[y + 1][x - 1] +
                    grid1[y + 1][x] +
                    grid1[y + 1][x + 1];

            grid2[y][x] = count > 6 ? 1 : 0;
        }
    }

    console.log(
        "grid",
        startX,
        startY,
        grid2.map((row) => row.toString().replace(/[^10]/g, "")),
    );

    DigRooms(rng, width, height);

    return grid1;
};
