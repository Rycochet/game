/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { getTerrain, Terrain } from "../World/useTerrain";
import { getWorld } from "../World/World";
import { type Coordinate3D } from "../World/WorldContext";
import { getSharedState, useWeakKey } from "../../hooks";

export interface MapGrid {
    x: number;
    y: number;
    z: number;
    background: Terrain;
    feature: number;
    rain: number;
}

const renderMapGrid = (key: Coordinate3D, x: number, y: number, z: number) => {
    if (cache.has(key)) {
        return cache.get(key)!;
    }

    const { circumference, climate, feature3D, land3D, weather3D, seed } = getWorld();
    const playerZ = getSharedState<number>("player.z", 0);
    const background = getTerrain(x, y, playerZ, circumference, climate, land3D);
    // const rng = Alea(seed, x, y, z);

    const data: MapGrid = {
        x,
        y,
        z,
        background,
        feature: background > Terrain.WATER ? feature3D(x, y, z * background * 100) : 0,
        rain: weather3D(x, y, 0),
    };

    cache.set(key, data);

    return data;
}

/**
 * Cache of coordinates to data, this allows us to do the expensive procedural
 * generation less often.
 */
const cache = new WeakMap<Coordinate3D, MapGrid>();

/**
 * Render (and cache) a grid location. If the x & y are both on a 100m boundary
 * then this is a special cell that contains extra information about the rest of
 * the grid area.
 */
export const useMapCoords = (x: number, y: number, z: number) =>
    renderMapGrid(useWeakKey<Coordinate3D>([x, y, z]), x, y, z);
