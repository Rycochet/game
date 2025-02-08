/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import type { Coordinate2D } from "./Coordinates";
import type { Point } from "./Point";
import { Tile } from "./Tile";

/**
 * The entire Map is a single Grid. Additionally this can be used as individual
 * Rooms, for stamping onto the grid.
 */
export interface IGrid {
    /**
     * Width of the grid, used for Goint lookup.
     */
    width: number;

    /**
     * Height of the grid, used for Point lookup.
     */
    height: number;

    /**
     * Positions we can enter the grid. For a room this is used for placement,
     * for a Map this is used for where the player comes in.
     */
    enter: Coordinate2D[];

    /**
     * The Points for each cell, some may be empty.
     */
    points: Point[];
}

export class Grid implements IGrid {
    width: number = 0;
    height: number = 0;
    enter: Coordinate2D[] = [];
    points: Point[] = [];

    getTile(x: number, y: number) {
        if (x >= 0 && y >= 0 && x < this.width && y < this.height) {
            return this.points[x + y * this.width] ?? "?";
        }
    }

    setTile(x: number, y: number, tile: Tile) {
        this.points[x + y * this.width] = { tile };
    }
}
