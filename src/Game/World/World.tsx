/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { useMemo, type FunctionComponent, type PropsWithChildren } from "react";
import { createNoise3D } from "simplex-noise";

import { Alea } from "../../common";
import { WorldMap } from "../../WorldMap";
import { getTerrain, Terrain } from "./useTerrain";
import { WorldContext, type Coordinate2D, type IWorld } from "./WorldContext";

let currentWorld: IWorld;

/**
 * Shortcut to get the World without having to use React.
 */
export const getWorld = () => currentWorld!;

export const World: FunctionComponent<PropsWithChildren<{ seed: number }>> = ({ children, seed }) => {
    const radius = Alea(seed, "radius").uint(6500000, 500); // Earth is 6378km
    const land3D = createNoise3D(Alea(seed, "land"));
    const circumference = Math.floor(2 * radius * Math.PI);
    const climate = Alea(seed, "climate").abs(1);
    const maxY = Math.floor(circumference / 5); // so we can't approach the poles
    const gates = useMemo(() => {
        const gateRng = Alea(seed, "gate");
        const arr: [Coordinate2D, Coordinate2D] = [] as any;

        for (let i = 0; i < 2; i++) {
            let x: number, y: number, terrain: Terrain;

            do {
                x = gateRng.uint(circumference);
                y = gateRng.int(maxY);
                terrain = getTerrain(x, y, 0, circumference, climate, land3D);
            } while (terrain !== Terrain.DIRT);

            arr.push([x, y]);
        }

        return arr;
    }, []);

    const world: IWorld = (currentWorld = {
        seed,
        land3D,
        feature3D: createNoise3D(Alea(seed, "feature")),
        weather3D: createNoise3D(Alea(seed, "weather")),
        radius,
        circumference,
        year: Alea(seed, "year").uint(265) + 100,
        density: Alea(seed, "density").abs(1.5, 0.5),
        tilt: Alea(seed, "tilt").abs(1),
        climate,
        gates,
    });

    return (
        <WorldContext.Provider value={world}>
            {children}
        </WorldContext.Provider>
    );
};
