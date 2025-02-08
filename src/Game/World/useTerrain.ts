/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { useMemo } from "react";
import type { NoiseFunction3D } from "simplex-noise";

import { useSharedState } from "../../hooks";
import { PlayerState } from "../../Types";
import { useWorld } from "./WorldContext";

const TAU = 2 * Math.PI; // Radius ratio to world size

export enum Terrain {
    WATER,
    SAND,
    DIRT,
    ROCK,
}

/**
 * Calculate terrain, internal before we get into React.
 */
export const getTerrain = (
    x: number,
    y: number,
    z: number,
    circumference: number,
    climate: number,
    land3D: NoiseFunction3D,
): Terrain => {
    const angle = (2 * Math.PI * x) / circumference;
    const distance = circumference / 1000 + Math.min(0, z / 10);
    const xPos = (Math.cos(angle) * distance) / TAU;
    const yPos = (Math.sin(angle) * distance) / TAU;
    const value = land3D(xPos, yPos, y / 1000) + 1 - z / 10;

    switch (true) {
        case value >= 1.6:
            return Terrain.ROCK;
        case value >= 1:
            return Terrain.DIRT;
        case value >= climate + 0.3:
            return Terrain.SAND;
        default:
            return Terrain.WATER;
    }
};

/**
 * Find the value at the specified position.
 *
 * @param x
 * @param y
 */
export function useTerrain(x: number, y: number) {
    const { circumference, climate, land3D } = useWorld();
    const [playerZ] = useSharedState<number>(PlayerState.Z);

    return useMemo(() => getTerrain(x, y, playerZ / 10, circumference, climate, land3D), [x, y, playerZ]);
}
