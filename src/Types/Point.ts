/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import type { Tile } from "./Tile";

export interface Point {
    tile: Tile;

    /**
     * If true then can not pass through this square - walls & closed doors etc.
     */
    solid?: boolean;

    /**
     * If true then can see through this square.
     */
    transparent?: boolean;

    /**
     * Used for identifying which Points are part of the same room. These may
     * not be contiguous but will always be within the same Grid.
     */
    roomId?: number;
}
