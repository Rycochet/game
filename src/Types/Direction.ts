/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

/**
 * Cardinal directions, same layout as numeric keypad.
 */
export enum Direction {
    SOUTHWEST = 1,
    SOUTH = 2,
    SOUTHEAST = 3,
    WEST = 4,
    NONE = 5,
    EAST = 6,
    NORTHWEST = 7,
    NORTH = 8,
    NORTHEAST = 9,
    UP = 10,
    DOWN = 11,
}

export namespace Direction {
    export function Reverse(dir: Direction) {
        switch (dir) {
            case Direction.NORTH: return Direction.SOUTH;
            case Direction.NORTHEAST: return Direction.SOUTHWEST;
            case Direction.EAST: return Direction.WEST;
            case Direction.SOUTHEAST: return Direction.NORTHWEST;
            case Direction.SOUTH: return Direction.NORTH;
            case Direction.SOUTHWEST: return Direction.NORTHEAST;
            case Direction.WEST: return Direction.EAST;
            case Direction.NORTHWEST: return Direction.SOUTHEAST;

            case Direction.UP: return Direction.DOWN;
            case Direction.DOWN: return Direction.UP;

            case Direction.NONE: return Direction.NONE;
        }
    }
}
