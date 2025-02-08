/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { type Coordinate2D, Grid, type Point, Tile } from "../../../../../Types";

export const Rooms = new Set<Room>()

export class Room extends Grid {
    constructor(rows: string[], map?: Record<string, Tile>) {
        super();

        const height = this.height = rows.length;
        const width = this.width = rows[0].length;
        const enter: Coordinate2D[] = this.enter = [];
        const points: Point[] = this.points = [];

        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                const tile = rows[y][x] as Tile;


                switch (tile) {
                    case Tile.UNUSED:
                        points.length++;
                        break;

                    case Tile.CLOSEDDOOR:
                    case Tile.WALL:
                        points.push({ tile: map?.[tile] ?? tile, solid: true });
                        break;

                    case Tile.OPENDOOR:
                    case Tile.FLOOR:
                    case Tile.CORRIDOOR:
                    case Tile.UPSTAIRS:
                    case Tile.DOWNSTAIRS:
                        points.push({ tile: map?.[tile] ?? tile, solid: false });
                        break;

                    case Tile.ENTRY:
                        enter.push([x, y]);
                        points.push({ tile: Tile.FLOOR, solid: false });
                        break;
                }
            }
        }

        Rooms.add(this);
    }
}

new Room([
    "#?#",
    "#,#",
    "#?#",
], {
    "?": Tile.CORRIDOOR,
});

new Room([
    "#???#",
    "?...?",
    "?...?",
    "?...?",
    "#???#",
], {
    "?": Tile.WALL,
});

new Room([
    "      ########      ",
    "   ####......####   ",
    "  ##............##  ",
    " ##..............## ",
    " #...##########...# ",
    "##..##...##...##..##",
    "#...#....##....#...#",
    "#...#....##....#...#",
    "#...#....##....#...#",
    "#...#...####...#...#",
    "#...##+######+##...#",
    "#.......####.......#",
    "##......####......##",
    " #.......##.......# ",
    " ##..............#  ",
    "  #..............#  ",
    "  #..............#  ",
    "  ##+##+##+##+##+#  ",
    "  #..#..#..#..#..#  ",
    "  #..#..#..#..#..#  ",
    "  #..#..#..#..#..#  ",
    "  ################  ",
], {
    "?": Tile.CLOSEDDOOR,
});
