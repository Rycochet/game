/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

/**
 * Tile types for easy display.
 *
 * TODO: This should be development only mapping.
 */
export enum Tile {
    UNUSED = " ",
    FLOOR = ".",
    ENTRY = "?", // Used for Rooms, this is otherwise a Floor
    CORRIDOOR = ",",
    WALL = "#",
    CLOSEDDOOR = "+",
    OPENDOOR = "-",
    UPSTAIRS = "<",
    DOWNSTAIRS = ">",
}
